var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);
var extension_exports = {};
__export(extension_exports, {
  activate: () => activate,
  deactivate: () => deactivate
});
module.exports = __toCommonJS(extension_exports);
var path = __toESM(require("path"));
var import_vscode = require("vscode");
var import_node = require("vscode-languageclient/node");
let client;
function activate(context) {
  const serverModule = context.asAbsolutePath(
    path.join("server", "out", "server.js")
  );
  const serverOptions = {
    run: { module: serverModule, transport: import_node.TransportKind.ipc },
    debug: {
      module: serverModule,
      transport: import_node.TransportKind.ipc
    }
  };
  const clientOptions = {
    // Register the server for plain text documents
    documentSelector: [
      {
        scheme: "file",
        language: "typescript"
      },
      {
        scheme: "file",
        language: "javascript"
      },
      {
        scheme: "file",
        language: "javascriptreact"
      },
      {
        scheme: "file",
        language: "typescriptreact"
      }
    ],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: import_vscode.workspace.createFileSystemWatcher("**/.clientrc")
    }
  };
  client = new import_node.LanguageClient(
    "doesItThrow",
    "Does it Throw Server",
    serverOptions,
    clientOptions
  );
  client.start();
}
function deactivate() {
  if (!client) {
    return void 0;
  }
  return client.stop();
}
