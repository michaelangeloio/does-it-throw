use std::collections::{BTreeMap, HashSet};

use lsp_types::{Position, PublishDiagnosticsParams, Range, TextDocumentItem, Url};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

#[cfg(not(test))]
pub(crate) fn log(s: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        console_log(&("[pls] ".to_owned() + s))
    }
}

#[cfg(test)]
pub(crate) fn log(_: &str) {}

pub(crate) type Documents = BTreeMap<Url, TextDocumentItem>;
pub(crate) type Diagnostics = BTreeMap<Url, PublishDiagnosticsParams>;

pub(crate) fn empty_diagnostics_for_doc(
    (uri, doc): (&Url, &TextDocumentItem),
) -> (Url, PublishDiagnosticsParams) {
    let params = PublishDiagnosticsParams::new(uri.clone(), vec![], Some(doc.version));
    (uri.clone(), params)
}

#[derive(Default, Serialize)]
pub(crate) struct LspEvent<'a> {
    pub(crate) lsp_method: &'a str,
    pub(crate) lsp_file_extensions: HashSet<String>,
}

pub(crate) fn unique_extensions(uris: &[&Url]) -> HashSet<String> {
    uris.iter()
        .filter_map(|uri| uri.as_str().rsplit_once('.'))
        .map(|(_, suffix)| suffix.into())
        .collect()
}
