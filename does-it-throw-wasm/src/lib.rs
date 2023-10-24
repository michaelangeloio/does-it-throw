use std::{collections::BTreeMap, str::Split};

use lsp_types::{
    notification::{
        DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidDeleteFiles,
        DidOpenTextDocument, DidSaveTextDocument, Initialized, Notification,
    },
    DeleteFilesParams, Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidChangeWatchedFilesParams, DidOpenTextDocumentParams, FileChangeType, FileDelete, FileEvent,
    NumberOrString, PublishDiagnosticsParams, TextDocumentItem, Url,
    VersionedTextDocumentIdentifier,
};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

mod helpers;
use helpers::{
    empty_diagnostics_for_doc, log, unique_extensions,
     Diagnostics, Documents, LspEvent,
};

#[wasm_bindgen]
pub struct PolarLanguageServer {
    documents: Documents,
    send_diagnostics_callback: js_sys::Function,
    telemetry_callback: js_sys::Function,
}

/// Public API exposed via WASM.
#[wasm_bindgen]
impl PolarLanguageServer {
    #[wasm_bindgen(constructor)]
    pub fn new(
        send_diagnostics_callback: &js_sys::Function,
        telemetry_callback: &js_sys::Function,
    ) -> Self {
        console_error_panic_hook::set_once();

        Self {
            documents: BTreeMap::new(),
            send_diagnostics_callback: send_diagnostics_callback.clone(),
            telemetry_callback: telemetry_callback.clone(),
        }
    }

    /// Catch-all handler for notifications sent by the LSP client.
    ///
    /// This function receives a notification's `method` and `params` and dispatches to the
    /// appropriate handler function based on `method`.
    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = PolarLanguageServer, js_name = onNotification)]
    pub fn on_notification(&mut self, method: &str, params: JsValue) {
        log(method);

        match method {
            DidOpenTextDocument::METHOD => {
                let DidOpenTextDocumentParams { text_document } = from_value(params).unwrap();

                let event = LspEvent {
                    lsp_method: method,
                    lsp_file_extensions: unique_extensions(&[&text_document.uri]),
                };

                let diagnostics = self.on_did_open_text_document(text_document);
                self.send_diagnostics(&diagnostics);

            }

            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = from_value(params).unwrap();
                println!("params: {:?}", params);

                // Ensure we receive full -- not incremental -- updates.
                assert_eq!(params.content_changes.len(), 1);
                let change = params.content_changes.into_iter().next().unwrap();
                assert!(change.range.is_none());

                let VersionedTextDocumentIdentifier { uri, version } = params.text_document;

                let event = LspEvent {
                    lsp_method: method,
                    lsp_file_extensions: unique_extensions(&[&uri]),
                };

                let updated_doc = TextDocumentItem::new(uri, "polar".into(), version, change.text);
                let diagnostics = self.on_did_change_text_document(updated_doc);
                self.send_diagnostics(&diagnostics);

            }

            // This is the type of event we'll receive when a Polar file is deleted, either via the
            // VS Code UI (right-click delete) or otherwise (e.g., `rm blah.polar` in a terminal).
            // The event comes from the `deleteWatcher` file watcher in the extension client.
            DidChangeWatchedFiles::METHOD => {
                let DidChangeWatchedFilesParams { changes } = from_value(params).unwrap();
                let uris: Vec<_> = changes
                    .into_iter()
                    .map(|FileEvent { uri, typ }| {
                        assert_eq!(typ, FileChangeType::DELETED); // We only watch for `Deleted` events.
                        uri
                    })
                    .collect();

                let event = LspEvent {
                    lsp_method: method,
                    lsp_file_extensions: unique_extensions(&uris.iter().collect::<Vec<_>>()),
                };

                let diagnostics = self.on_did_change_watched_files(uris);
                self.send_diagnostics(&diagnostics);

            }

            // This is the type of event we'll receive when *any* file or folder is deleted via the
            // VS Code UI (right-click delete). These events are triggered by the
            // `workspace.fileOperations.didDelete.filters[0].glob = '**'` capability we send from
            // the TS server -> client, which then sends us `didDelete` events for *all files and
            // folders within the current workspace*. This is how we are notified of directory
            // deletions that might contain Polar files, since they won't get picked up by the
            // `deleteWatcher` created in the client for reasons elaborated below.
            //
            // We can ignore any Polar file URIs received via this handler since they'll already be
            // covered by a corresponding `DidChangeWatchedFiles` event emitted by the
            // `deleteWatcher` file watcher in the extension client that watches for any
            // `**/*.polar` files deleted in the current workspace.
            //
            // In this handler we only care about *non-Polar* URIs, which we treat as potential
            // deletions of directories containing Polar files since those won't get picked up by
            // the `deleteWatcher` due to [a limitation of VS Code's file watching
            // capabilities][0].
            //
            // [0]: https://github.com/microsoft/vscode/issues/60813
            DidDeleteFiles::METHOD => {
                let DeleteFilesParams { files } = from_value(params).unwrap();
                let mut uris = vec![];
                for FileDelete { uri } in files {
                    match Url::parse(&uri) {
                        Ok(uri) => uris.push(uri),
                        Err(e) => log(&format!("\tfailed to parse URI: {}", e)),
                    }
                }

                let event = LspEvent {
                    lsp_method: method,
                    lsp_file_extensions: unique_extensions(&uris.iter().collect::<Vec<_>>()),
                };

                if let Some(diagnostics) = self.on_did_delete_files(uris) {
                    self.send_diagnostics(&diagnostics);
                }
            }

            // We don't care when a document is saved -- we already have the updated state thanks
            // to `DidChangeTextDocument`.
            DidSaveTextDocument::METHOD => (),
            // We don't care when a document is closed -- we care about all Polar files in a
            // workspace folder regardless of which ones remain open.
            DidCloseTextDocument::METHOD => (),
            // Nothing to do when we receive the `Initialized` notification.
            Initialized::METHOD => (),

            _ => log("unexpected notification"),
        }
    }
}

/// Individual LSP notification handlers.
impl PolarLanguageServer {
    fn on_did_open_text_document(&mut self, doc: TextDocumentItem) -> Diagnostics {
        log(&format!("\topening: {}", doc.uri));
        if self.upsert_document(doc).is_some() {
            log("\t\treopened tracked doc");
        }
        self.reload_kb()
    }

    fn on_did_change_text_document(&mut self, doc: TextDocumentItem) -> Diagnostics {
        let uri = doc.uri.clone();
        if self.upsert_document(doc).is_none() {
            log(&format!("\tupdated untracked doc: {}", uri));
        }
        self.reload_kb()
    }

    // This is (currently) only used to handle deletions of Polar *files*. `DidChangeWatchedFiles`
    // events come from the `deleteWatcher` filesystem watcher in the extension client. Due to [a
    // limitation of VS Code's filesystem watcher][0], we don't receive deletion events for Polar
    // files nested inside of a deleted directory. See corresponding comments on `DidDeleteFiles`
    // and `DidChangeWatchedFiles` in `PolarLanguageServer::on_notification`.
    //
    // [0]: https://github.com/microsoft/vscode/issues/60813
    fn on_did_change_watched_files(&mut self, uris: Vec<Url>) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();

        for uri in uris {
            log(&format!("\tdeleting: {}", uri));

            // If this returns `None`, `uri` was already removed from the local set of tracked
            // documents. An easy way to encounter this is to right-click delete a Polar file via
            // the VS Code UI, which races the `DidDeleteFiles` and `DidChangeWatchedFiles` events.
            if let Some(removed) = self.remove_document(&uri) {
                let (_, empty_diagnostics) = empty_diagnostics_for_doc((&uri, &removed));
                if diagnostics.insert(uri, empty_diagnostics).is_some() {
                    log("\t\tduplicate URIs in event payload");
                }
            } else {
                log("\t\tcannot delete untracked doc");
            }
        }

        diagnostics.append(&mut self.reload_kb());
        diagnostics
    }

    // Returns `None` if no Polar files were deleted.
    fn on_did_delete_files(&mut self, uris: Vec<Url>) -> Option<Diagnostics> {
        let mut diagnostics = Diagnostics::new();

        for uri in uris {
            // If `removed` is empty, `uri` wasn't a directory containing tracked Polar files or
            // `uri` itself was a Polar file that was already removed via `DidChangeWatchedFiles`.
            let removed = self.remove_documents_in_dir(&uri);
            if !removed.is_empty() {
                log(&format!("\tdeleting: {}", uri));

                for (uri, params) in removed {
                    log(&format!("\t\tdeleted: {}", uri));

                    // NOTE(gj): fairly sure this will never be true.
                    if diagnostics.insert(uri, params).is_some() {
                        log("\t\t\tmultiple deletions of same doc");
                    }
                }
            }
        }

        if diagnostics.is_empty() {
            None
        } else {
            diagnostics.append(&mut self.reload_kb());
            Some(diagnostics)
        }
    }
}

/// Helper methods.
impl PolarLanguageServer {
    fn upsert_document(&mut self, doc: TextDocumentItem) -> Option<TextDocumentItem> {
        self.documents.insert(doc.uri.clone(), doc)
    }

    fn remove_document(&mut self, uri: &Url) -> Option<TextDocumentItem> {
        self.documents.remove(uri)
    }

    /// Remove tracked docs inside `dir`.
    fn remove_documents_in_dir(&mut self, dir: &Url) -> Diagnostics {
        let (in_dir, not_in_dir): (Documents, Documents) =
            self.documents.clone().into_iter().partition(|(uri, _)| {
                // Zip pair of `Option<Split<char>>`s into `Option<(Split<char>, Split<char>)>`.
                let maybe_segments = dir.path_segments().zip(uri.path_segments());
                // Compare paths (`Split<char>`) by zipping them together and comparing pairwise.
                let compare_paths = |(l, r): (Split<_>, Split<_>)| l.zip(r).all(|(l, r)| l == r);
                // If all path segments match b/w dir & uri, uri is in dir and should be removed.
                maybe_segments.map_or(false, compare_paths)
            });
        // Replace tracked docs w/ docs that aren't in the removed dir.
        self.documents = not_in_dir;
        in_dir.iter().map(empty_diagnostics_for_doc).collect()
    }

    fn send_diagnostics(&self, diagnostics: &Diagnostics) {
        let this = &JsValue::null();
        for params in diagnostics.values() {
            let params = &to_value(&params).unwrap();
            if let Err(e) = self.send_diagnostics_callback.call1(this, params) {
                log(&format!(
                    "send_diagnostics params:\n\t{:?}\n\tJS error: {:?}",
                    params, e
                ));
            }
        }
    }


    fn empty_diagnostics_for_all_documents(&self) -> Diagnostics {
        self.documents
            .iter()
            .map(empty_diagnostics_for_doc)
            .collect()
    }


  

    fn get_diagnostics(&self) -> Diagnostics {
       return Diagnostics::new(); 
    }

    /// Reloads tracked documents into the `KnowledgeBase`, translates `polar-core` diagnostics
    /// into `polar-language-server` diagnostics, and returns a set of diagnostics for publishing.
    ///
    /// NOTE(gj): we republish 'empty' diagnostics for all documents in order to purge stale
    /// diagnostics.
    fn reload_kb(&self) -> Diagnostics {
        let mut diagnostics = self.empty_diagnostics_for_all_documents();
        diagnostics.extend(self.get_diagnostics());
        diagnostics
    }
}
