import {
  DidChangeConfigurationNotification,
  InitializeParams,
  InitializeResult,
  ProposedFeatures,
  TextDocumentSyncKind,
  TextDocuments,
  createConnection
} from 'vscode-languageserver/node'

import { access, constants, readFile } from 'fs/promises'
import { TextDocument } from 'vscode-languageserver-textdocument'
import { InputData, ParseResult, parse_js } from './rust/does_it_throw_wasm'
import path = require('path')
import { inspect } from 'util'
import { getAnalysisResults } from './analysis'

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
      textDocumentSync: TextDocumentSyncKind.Incremental
    }
  }
  if (params?.workspaceFolders && params.workspaceFolders.length > 1) {
    throw new Error('This extension only supports one workspace folder at this time')
  }
  if (hasWorkspaceFolderCapability) {
    result.capabilities.workspace = {
      workspaceFolders: {
        supported: false
      }
    }
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

type DiagnosticSeverity = 'Error' | 'Warning' | 'Information' | 'Hint'

// The server settings
interface Settings {
  maxNumberOfProblems: number
  throwStatementSeverity: DiagnosticSeverity
  functionThrowSeverity: DiagnosticSeverity
  callToThrowSeverity: DiagnosticSeverity
  callToImportedThrowSeverity: DiagnosticSeverity
  includeTryStatementThrows: boolean
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
const defaultSettings: Settings = {
  maxNumberOfProblems: 1000000,
  throwStatementSeverity: 'Hint',
  functionThrowSeverity: 'Hint',
  callToThrowSeverity: 'Hint',
  callToImportedThrowSeverity: 'Hint',
  includeTryStatementThrows: false
}
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
  // biome-ignore lint/complexity/noForEach: original vscode-languageserver code
  documents.all().forEach(validateTextDocument)
})

function getDocumentSettings(resource: string): Thenable<Settings> {
  if (!hasConfigurationCapability) {
    connection.console.info(`does not have config capability, using global settings: ${JSON.stringify(globalSettings)}`)
    return Promise.resolve(globalSettings)
  }
  let result = documentSettings.get(resource)
  if (!result) {
    result = connection.workspace.getConfiguration({
      scopeUri: resource,
      section: 'doesItThrow'
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

async function validateTextDocument(textDocument: TextDocument): Promise<void> {
  let settings = await getDocumentSettings(textDocument.uri)
  if (!settings) {
    // this should never happen, but just in case
    connection.console.warn(`No settings found for ${textDocument.uri}, using defaults`)
    settings = defaultSettings
  }
  try {
    const opts = {
      uri: textDocument.uri,
      file_content: textDocument.getText(),
      ids_to_check: [],
      typescript_settings: {
        decorators: true
      },
      function_throw_severity: settings?.functionThrowSeverity ?? defaultSettings.functionThrowSeverity,
      throw_statement_severity: settings?.throwStatementSeverity ?? defaultSettings.throwStatementSeverity,
      call_to_imported_throw_severity:
        settings?.callToImportedThrowSeverity ?? defaultSettings.callToImportedThrowSeverity,
      call_to_throw_severity: settings?.callToThrowSeverity ?? defaultSettings.callToThrowSeverity,
      include_try_statement_throws: settings?.includeTryStatementThrows ?? defaultSettings.includeTryStatementThrows
    } satisfies InputData
    const analysis = await getAnalysisResults({
      errorLogCallback: (msg) => connection.console.error(msg),
      inputData: opts,
      initialUri: textDocument.uri
    })
    connection.sendDiagnostics({
      uri: textDocument.uri,
      diagnostics: analysis?.diagnostics ?? []
    })
  } catch (e) {
    console.log(e)
    connection.console.error(`Error parsing file ${textDocument.uri}`)
    connection.console.error(`settings are: ${JSON.stringify(settings)}`)
    connection.console.error(`Error: ${e instanceof Error ? e.message : JSON.stringify(e)} error`)
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
