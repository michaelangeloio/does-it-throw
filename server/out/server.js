"use strict";
var import_node = require("vscode-languageserver/node");
var import_vscode_languageserver_textdocument = require("vscode-languageserver-textdocument");
var import_does_it_throw = require("./rust/does_it_throw");
const connection = (0, import_node.createConnection)(import_node.ProposedFeatures.all);
const documents = new import_node.TextDocuments(import_vscode_languageserver_textdocument.TextDocument);
let hasConfigurationCapability = false;
let hasWorkspaceFolderCapability = false;
let hasDiagnosticRelatedInformationCapability = false;
connection.onInitialize((params) => {
  const capabilities = params.capabilities;
  hasConfigurationCapability = !!(capabilities.workspace && !!capabilities.workspace.configuration);
  hasWorkspaceFolderCapability = !!(capabilities.workspace && !!capabilities.workspace.workspaceFolders);
  hasDiagnosticRelatedInformationCapability = !!(capabilities.textDocument && capabilities.textDocument.publishDiagnostics && capabilities.textDocument.publishDiagnostics.relatedInformation);
  const result = {
    capabilities: {
      textDocumentSync: import_node.TextDocumentSyncKind.Incremental,
      // Tell the client that this server supports code completion.
      completionProvider: {
        resolveProvider: true
      }
    }
  };
  if (hasWorkspaceFolderCapability) {
    result.capabilities.workspace = {
      workspaceFolders: {
        supported: true
      }
    };
  }
  return result;
});
connection.onInitialized(() => {
  if (hasConfigurationCapability) {
    connection.client.register(import_node.DidChangeConfigurationNotification.type, void 0);
  }
  if (hasWorkspaceFolderCapability) {
    connection.workspace.onDidChangeWorkspaceFolders((_event) => {
      connection.console.log(`Workspace folder change event received. ${JSON.stringify(_event)}`);
    });
  }
});
const defaultSettings = { maxNumberOfProblems: 1e3 };
let globalSettings = defaultSettings;
const documentSettings = /* @__PURE__ */ new Map();
connection.onDidChangeConfiguration((change) => {
  if (hasConfigurationCapability) {
    documentSettings.clear();
  } else {
    globalSettings = change.settings.doesItThrow || defaultSettings;
  }
  documents.all().forEach(validateTextDocument);
});
function getDocumentSettings(resource) {
  if (!hasConfigurationCapability) {
    return Promise.resolve(globalSettings);
  }
  let result = documentSettings.get(resource);
  if (!result) {
    result = connection.workspace.getConfiguration({
      scopeUri: resource,
      section: "doesItThrow"
    });
    documentSettings.set(resource, result);
  }
  return result;
}
documents.onDidClose((e) => {
  documentSettings.delete(e.document.uri);
});
documents.onDidChangeContent((change) => {
  validateTextDocument(change.document);
});
async function validateTextDocument(textDocument) {
  console.log("\x1B[36m%s\x1B[0m", hasDiagnosticRelatedInformationCapability);
  try {
    const jsonString = (0, import_does_it_throw.parse_js)(textDocument.getText());
    connection.console.log(`jsonString: ${jsonString}`);
    connection.sendDiagnostics({ uri: textDocument.uri, diagnostics: JSON.parse(jsonString) });
  } catch (e) {
    connection.console.log(`${e instanceof Error ? e.message : JSON.stringify(e)} error`);
    connection.sendDiagnostics({ uri: textDocument.uri, diagnostics: [] });
  }
  const settings = await getDocumentSettings(textDocument.uri);
  if (!settings) {
    console.log("\x1B[36m%s\x1B[0m", "no settings");
  }
}
connection.onDidChangeWatchedFiles((_change) => {
  connection.console.log(`We received an file change event ${_change}`);
});
connection.onCompletion(
  (_textDocumentPosition) => {
    connection.console.log(`onCompletion ${_textDocumentPosition.position.line}}`);
    return [
      {
        label: "TypeScript",
        kind: import_node.CompletionItemKind.Text,
        data: 1
      },
      {
        label: "JavaScript",
        kind: import_node.CompletionItemKind.Text,
        data: 2
      }
    ];
  }
);
connection.onCompletionResolve(
  (item) => {
    if (item.data === 1) {
      item.detail = "TypeScript details";
      item.documentation = "TypeScript documentation";
    } else if (item.data === 2) {
      item.detail = "JavaScript details";
      item.documentation = "JavaScript documentation";
    }
    return item;
  }
);
documents.listen(connection);
connection.listen();
