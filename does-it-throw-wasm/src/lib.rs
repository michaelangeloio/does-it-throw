extern crate swc_common;
extern crate swc_core;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;
use does_it_throw::get_analysis_results;
use std::{collections::BTreeMap, fmt::format, str::Split};


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
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

mod helpers;
use helpers::{
    empty_diagnostics_for_doc, log, unique_extensions, Diagnostics, Documents, LspEvent,
};

use self::swc_common::{sync::Lrc, SourceMap};

#[wasm_bindgen]
pub struct DITLanguageServer {
    documents: Documents,
    send_diagnostics_callback: js_sys::Function,
    telemetry_callback: js_sys::Function,
}

/// Public API exposed via WASM.
#[wasm_bindgen]
impl DITLanguageServer {
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
    #[wasm_bindgen(js_class = DITLanguageServer, js_name = onNotification)]
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

                let updated_doc = TextDocumentItem::new(uri, "ts".into(), version, change.text);
                let diagnostics = self.on_did_change_text_document(updated_doc);
                self.send_diagnostics(&diagnostics);
            }

            // This is the type of event we'll receive when a ts file is deleted, either via the
            // VS Code UI (right-click delete) or otherwise (e.g., `rm blah.ts` in a terminal).
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
            // deletions that might contain ts files, since they won't get picked up by the
            // `deleteWatcher` created in the client for reasons elaborated below.
            //
            // We can ignore any ts file URIs received via this handler since they'll already be
            // covered by a corresponding `DidChangeWatchedFiles` event emitted by the
            // `deleteWatcher` file watcher in the extension client that watches for any
            // `**/*.ts` files deleted in the current workspace.
            //
            // In this handler we only care about *non-ts* URIs, which we treat as potential
            // deletions of directories containing ts files since those won't get picked up by
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
            // We don't care when a document is closed -- we care about all ts files in a
            // workspace folder regardless of which ones remain open.
            DidCloseTextDocument::METHOD => (),
            // Nothing to do when we receive the `Initialized` notification.
            Initialized::METHOD => (),

            _ => log("unexpected notification"),
        }
    }
}

/// Individual LSP notification handlers.
impl DITLanguageServer {
    fn on_did_open_text_document(&mut self, doc: TextDocumentItem) -> Diagnostics {
        let uri = doc.uri.clone();
        log(&format!("\topening: {}", doc.uri));
        if self.upsert_document(doc).is_some() {
            log("\t\treopened tracked doc");
        }
        let uris_vec = vec![uri.clone()];
        self.reload_kb(uris_vec)
    }

    fn on_did_change_text_document(&mut self, doc: TextDocumentItem) -> Diagnostics {
        let uri = doc.uri.clone();
        if self.upsert_document(doc).is_none() {
            log(&format!("\tupdated untracked doc: {}", uri));
        }
        let uris_vec = vec![uri.clone()];
        self.reload_kb(uris_vec)
    }

    // This is (currently) only used to handle deletions of ts *files*. `DidChangeWatchedFiles`
    // events come from the `deleteWatcher` filesystem watcher in the extension client. Due to [a
    // limitation of VS Code's filesystem watcher][0], we don't receive deletion events for ts
    // files nested inside of a deleted directory. See corresponding comments on `DidDeleteFiles`
    // and `DidChangeWatchedFiles` in `DITLanguageServer::on_notification`.
    //
    // [0]: https://github.com/microsoft/vscode/issues/60813
    fn on_did_change_watched_files(&mut self, uris: Vec<Url>) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();

        for uri in &uris {
            log(&format!("\tdeleting: {}", uri));

            // If this returns `None`, `uri` was already removed from the local set of tracked
            // documents. An easy way to encounter this is to right-click delete a ts file via
            // the VS Code UI, which races the `DidDeleteFiles` and `DidChangeWatchedFiles` events.
            if let Some(removed) = self.remove_document(&uri) {
                let (_, empty_diagnostics) = empty_diagnostics_for_doc((&uri, &removed));
                if diagnostics.insert(uri.clone(), empty_diagnostics).is_some() {
                    log("\t\tduplicate URIs in event payload");
                }
            } else {
                log("\t\tcannot delete untracked doc");
            }
        }

        diagnostics.append(&mut self.reload_kb(uris));
        diagnostics
    }

    // Returns `None` if no ts files were deleted.
    fn on_did_delete_files(&mut self, uris: Vec<Url>) -> Option<Diagnostics> {
        let mut diagnostics = Diagnostics::new();

        for uri in &uris {
            // If `removed` is empty, `uri` wasn't a directory containing tracked ts files or
            // `uri` itself was a ts file that was already removed via `DidChangeWatchedFiles`.
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
            diagnostics.append(&mut self.reload_kb(uris));
            Some(diagnostics)
        }
    }
}

/// Helper methods.
impl DITLanguageServer {
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

    fn get_diagnostics(&self, uris: Vec<Url>) -> Diagnostics {
        for uri in &uris {
            log(&format!("\tgetting diagnostics for: {}", &uri));
        }
        for doc in self.documents.values() {
            let cm: Lrc<SourceMap> = Default::default();
            let fm = cm.new_source_file(swc_common::FileName::Anon, doc.text.clone());
            let lines = fm.count_lines();
            log(&format!("tracked lines {}", &lines.to_string()));
            let (results, cm) = get_analysis_results(doc.text.to_string().clone(), doc.uri.to_string().clone(), cm);

            println!("Functions that throw:");
            for fun in &results.functions_with_throws {
                let start = cm.lookup_char_pos(fun.throw_statement.lo());
                let end = cm.lookup_char_pos(fun.throw_statement.hi());
                log(&format!(
                    "From line {} column {} to line {} column {}",
                    start.line, start.col_display, end.line, end.col_display
                ));
                for span in &fun.throw_spans {
                    let start = cm.lookup_char_pos(span.lo());
                    let end = cm.lookup_char_pos(span.hi());
                    log(&format!(
                        "  Throw from line {} column {} to line {} column {}",
                        start.line, start.col_display, end.line, end.col_display
                    ));
                }
            }
            log(&format!("parsed module "));
        }
        return Diagnostics::new();
    }

    fn reload_kb(&self, uris: Vec<Url>) -> Diagnostics {
        let mut diagnostics = self.empty_diagnostics_for_all_documents();
        diagnostics.extend(self.get_diagnostics(uris));
        diagnostics
    }
}
