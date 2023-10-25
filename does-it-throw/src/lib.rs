pub mod shared;

extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;

use self::swc_common::{sync::Lrc, SourceMap};
use shared::{analyze_code, AnalysisResult};
pub fn get_analysis_results (content: String, path: String, cm: Lrc<SourceMap>) -> (AnalysisResult, Lrc<SourceMap>) {
  let (results, cm) = analyze_code(&content, cm, &path);
  (results, cm)
}
