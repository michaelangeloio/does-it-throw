import {
	DidChangeConfigurationNotification,
	InitializeParams,
	InitializeResult,
	ProposedFeatures,
	TextDocumentSyncKind,
	TextDocuments,
	createConnection
} from 'vscode-languageserver/node'

import {
	TextDocument
} from 'vscode-languageserver-textdocument'
import { parse_js } from './rust/does_it_throw_wasm'

const connection = createConnection(ProposedFeatures.all)

const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument)

let hasConfigurationCapability = false
let hasWorkspaceFolderCapability = false
// use if needed later
// let hasDiagnosticRelatedInformationCapability = false 

let rootUri: string | undefined | null

connection.onInitialize((params: InitializeParams) => {
	const capabilities = params.capabilities


	hasConfigurationCapability = !!(
		capabilities.workspace && !!capabilities.workspace.configuration
	)
	hasWorkspaceFolderCapability = !!(
		capabilities.workspace && !!capabilities.workspace.workspaceFolders
	)
	// use if needed later
	// hasDiagnosticRelatedInformationCapability = !!(
	// 	capabilities.textDocument &&
	// 	capabilities.textDocument.publishDiagnostics &&
	// 	capabilities.textDocument.publishDiagnostics.relatedInformation
	// ) 

	const result: InitializeResult = {
		capabilities: {
			textDocumentSync: TextDocumentSyncKind.Incremental,
		}
	}
	if (hasWorkspaceFolderCapability) {
		result.capabilities.workspace = {
			workspaceFolders: {
				supported: false
			}
		}
	}	
	if (params?.workspaceFolders && params.workspaceFolders.length > 1) {
		throw new Error('This extension only supports one workspace folder at this time')
	}
	if (!hasWorkspaceFolderCapability) {
		rootUri = params.rootUri
	}
	else {
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
		connection.workspace.onDidChangeWorkspaceFolders(_event => {
			connection.console.log(`Workspace folder change event received. ${
				JSON.stringify(_event)
			}`)
		})
	}
})

// The example settings
interface Settings {
	maxNumberOfProblems: number;
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
const defaultSettings: Settings = { maxNumberOfProblems: 1000000 } 
// 👆 very unlikely someone will have more than 1 million throw statements, lol point up
let globalSettings: Settings = defaultSettings

// Cache the settings of all open documents
const documentSettings: Map<string, Thenable<Settings>> = new Map()

connection.onDidChangeConfiguration(change => {
	if (hasConfigurationCapability) {
		// Reset all cached document settings
		documentSettings.clear()
	} else {
		globalSettings = <Settings>(
			(change.settings.doesItThrow || defaultSettings)
		)
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
			section: 'doesItThrow'
		})
		documentSettings.set(resource, result)
	}
	return result
}

// Only keep settings for open documents
documents.onDidClose(e => {
	documentSettings.delete(e.document.uri)
})

// The content of a text document has changed. This event is emitted
// when the text document first opened or when its content has changed.
documents.onDidChangeContent(change => {
	validateTextDocument(change.document)
})

async function validateTextDocument(textDocument: TextDocument): Promise<void> {
	try {
		const jsonString = parse_js(textDocument.getText())
		const diagnostics = JSON.parse(jsonString)
		const settings = await getDocumentSettings(textDocument.uri)
		if (settings.maxNumberOfProblems && diagnostics > settings.maxNumberOfProblems  ) {
			connection.sendDiagnostics({ uri: textDocument.uri, diagnostics: diagnostics.slice(0, settings.maxNumberOfProblems)})
		} else {
			connection.sendDiagnostics({ uri: textDocument.uri, diagnostics: JSON.parse(jsonString) }) 
		}
		
	}
	catch (e) {
		connection.console.log(`${e instanceof Error ? e.message: JSON.stringify(e)} error`)
		connection.sendDiagnostics({ uri: textDocument.uri, diagnostics: [] })
	}
	
}

connection.onDidChangeWatchedFiles(_change => {
	// Monitored files have change in VSCode
	connection.console.log(`We received an file change event ${_change}, not implemented yet`)
})


// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection)

// Listen on the connection
connection.listen()
