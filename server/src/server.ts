import {
	DidChangeConfigurationNotification,
	InitializeParams,
	InitializeResult,
	ProposedFeatures,
	TextDocumentSyncKind,
	TextDocuments,
	createConnection,
} from 'vscode-languageserver/node'

import { access, constants, readFile } from 'fs/promises'
import { TextDocument } from 'vscode-languageserver-textdocument'
import { InputData, ParseResult, parse_js } from './rust/does_it_throw_wasm'
import path = require('path')

const connection = createConnection(ProposedFeatures.all)

const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument)

let hasConfigurationCapability = false
let hasWorkspaceFolderCapability = false
// use if needed later
// let hasDiagnosticRelatedInformationCapability = false

let rootUri: string | undefined | null

connection.onInitialize((params: InitializeParams) => {
  const capabilities = params.capabilities

  hasConfigurationCapability = !!(capabilities.workspace && !!capabilities.workspace.configuration)
  hasWorkspaceFolderCapability = !!(capabilities.workspace && !!capabilities.workspace.workspaceFolders)
  // use if needed later
  // hasDiagnosticRelatedInformationCapability = !!(
  // 	capabilities.textDocument &&
  // 	capabilities.textDocument.publishDiagnostics &&
  // 	capabilities.textDocument.publishDiagnostics.relatedInformation
  // )

  const result: InitializeResult = {
    capabilities: {
      textDocumentSync: TextDocumentSyncKind.Incremental,
    },
  }
  if (hasWorkspaceFolderCapability) {
    result.capabilities.workspace = {
      workspaceFolders: {
        supported: false,
      },
    }
  }
  if (params?.workspaceFolders && params.workspaceFolders.length > 1) {
    throw new Error('This extension only supports one workspace folder at this time')
  }
  if (!hasWorkspaceFolderCapability) {
    rootUri = params.rootUri
  } else {
    rootUri = params?.workspaceFolders?.[0]?.uri
  }

  return result
})

connection.onInitialized(() => {
  if (hasConfigurationCapability) {
    // Register for all configuration changes.
    connection.client.register(DidChangeConfigurationNotification.type, undefined)
  }
  if (hasWorkspaceFolderCapability) {
    connection.workspace.onDidChangeWorkspaceFolders((_event) => {
      connection.console.log(`Workspace folder change event received. ${JSON.stringify(_event)}`)
    })
  }
})

// The example settings
interface Settings {
  maxNumberOfProblems: number
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
const defaultSettings: Settings = { maxNumberOfProblems: 1000000 }
// 👆 very unlikely someone will have more than 1 million throw statements, lol
// if you do, might want to rethink your code?
let globalSettings: Settings = defaultSettings

// Cache the settings of all open documents
const documentSettings: Map<string, Thenable<Settings>> = new Map()

connection.onDidChangeConfiguration((change) => {
  if (hasConfigurationCapability) {
    // Reset all cached document settings
    documentSettings.clear()
  } else {
    globalSettings = <Settings>(change.settings.doesItThrow || defaultSettings)
  }

  // Revalidate all open text documents
  documents.all().forEach(validateTextDocument)
})

// TODO - use this later if needed
function getDocumentSettings(resource: string): Thenable<Settings> {
  if (!hasConfigurationCapability) {
    return Promise.resolve(globalSettings)
  }
  let result = documentSettings.get(resource)
  if (!result) {
    result = connection.workspace.getConfiguration({
      scopeUri: resource,
      section: 'doesItThrow',
    })
    documentSettings.set(resource, result)
  }
  return result
}

// Only keep settings for open documents
documents.onDidClose((e) => {
  documentSettings.delete(e.document.uri)
})

// The content of a text document has changed. This event is emitted
// when the text document first opened or when its content has changed.
documents.onDidChangeContent(async (change) => {
  validateTextDocument(change.document)
})

documents.onDidSave((change) => {
  validateTextDocument(change.document)
})

const _checkAccessOnFile = async (file: string) => {
  try {
    await access(file, constants.R_OK)
    return Promise.resolve(file)
  } catch (e) {
    return Promise.reject(e)
  }
}

const findFirstFileThatExists = async (uri: string, relative_import: string) => {
  const isTs = uri.endsWith('.ts') || uri.endsWith('.tsx')
  const baseUri = `${path.resolve(path.dirname(uri.replace('file://', '')), relative_import)}`
  let files = Array(4)
  if (isTs) {
    files = [`${baseUri}.ts`, `${baseUri}.tsx`, `${baseUri}.js`, `${baseUri}.jsx`]
  } else {
    files = [`${baseUri}.js`, `${baseUri}.jsx`, `${baseUri}.ts`, `${baseUri}.tsx`]
  }
  return Promise.race(files.map(_checkAccessOnFile))
}

async function validateTextDocument(textDocument: TextDocument): Promise<void> {
  try {
    const opts = {
      uri: textDocument.uri,
      file_content: textDocument.getText(),
      ids_to_check: [],
      typescript_settings: {
        decorators: true,
      },
    } satisfies InputData
    const analysis = parse_js(opts) as ParseResult

    if (analysis.relative_imports.length > 0) {
      const resolvedImports = analysis.relative_imports.map((relative_import) => {
        return findFirstFileThatExists(textDocument.uri, relative_import)
      })
      const files = await Promise.all(
        resolvedImports.map(async (file) => {
          try {
            return readFile(await file, 'utf-8')
          } catch (e) {
            connection.console.log(`Error reading file ${e}`)
            return undefined
          }
        }),
      )
      const analysisArr = files.map((file) => {
        if (!file) {
          return undefined
        }
        const opts = {
          uri: textDocument.uri,
          file_content: file,
          ids_to_check: [],
          typescript_settings: {
            decorators: true,
          },
        } satisfies InputData
        return parse_js(opts) as ParseResult
      })
      // TODO - this is a bit of a mess, but it works for now.
      // The original analysis is the one that has the throw statements Map()
      // We get the get the throw_ids from the imported analysis and then
      // check the original analysis for existing throw_ids
      // This allows to to get the diagnostics from the imported analysis (one level deep for now)
      analysisArr.forEach((import_analysis) => {
        if (!import_analysis) {
          return
        }
        if (import_analysis.throw_ids.length) {
          import_analysis.throw_ids.forEach((throw_id) => {
            const newDiagnostics = analysis.imported_identifiers_diagnostics.get(throw_id)
            if (newDiagnostics && newDiagnostics?.diagnostics?.length) {
              analysis.diagnostics.push(...newDiagnostics.diagnostics)
            }
          })
        }
      })
    }

    connection.sendDiagnostics({ uri: textDocument.uri, diagnostics: analysis.diagnostics })
  } catch (e) {
    connection.console.log(`${e instanceof Error ? e.message : JSON.stringify(e)} error`)
    connection.sendDiagnostics({ uri: textDocument.uri, diagnostics: [] })
  }
}

connection.onDidChangeWatchedFiles((_change) => {
  // Monitored files have change in VSCode
  connection.console.log(`We received an file change event ${_change}, not implemented yet`)
})

// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection)

// Listen on the connection
connection.listen()
