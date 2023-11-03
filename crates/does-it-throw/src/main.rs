extern crate does_it_throw;
extern crate swc_common;
use std::fs;

use self::swc_common::{sync::Lrc, SourceMap, Span};
use does_it_throw::analyze_code;

pub fn main() {
  let sample_code = fs::read_to_string("crates/does-it-throw/src/fixtures/sample.ts")
    .expect("Something went wrong reading the file");
  let cm: Lrc<SourceMap> = Default::default();

  let (result, _cm) = analyze_code(&sample_code, cm);
  for import in result.import_sources.into_iter() {
    println!("Imported {}", import);
  }
  for fun in result.functions_with_throws.into_iter() {
    let start = _cm.lookup_char_pos(fun.throw_statement.lo());
    let end = _cm.lookup_char_pos(fun.throw_statement.hi());
    println!(
      "Function throws: {}, className {}",
      fun.function_or_method_name,
      fun.class_name.unwrap_or_else(|| "NOT_SET".to_string())
    );
    println!(
      "From line {} column {} to line {} column {}",
      start.line, start.col_display, end.line, end.col_display
    );
    for span in &fun.throw_spans {
      let start = _cm.lookup_char_pos(span.lo());
      let end = _cm.lookup_char_pos(span.hi());
      println!(
        "  Throw from line {} column {} to line {} column {}",
        start.line, start.col_display, end.line, end.col_display
      );
    }
  }
  println!("------- Calls to throws --------");
  for call in result.calls_to_throws.into_iter() {
    let start = _cm.lookup_char_pos(call.call_span.lo());
    let end = _cm.lookup_char_pos(call.call_span.hi());
    println!("Call throws: {}", call.id);
    println!(
      "From line {} column {} to line {} column {}",
      start.line, start.col_display, end.line, end.col_display
    );
  }

  println!("-------- Imported identifiers usages --------");
  for identifier_usage in result.imported_identifier_usages.into_iter() {
    let start = _cm.lookup_char_pos(identifier_usage.usage_span.lo());
    let end = _cm.lookup_char_pos(identifier_usage.usage_span.hi());
    let identifier_name = &identifier_usage.id;
    println!(
      "{} From line {} column {} to line {} column {}",
      identifier_name, start.line, start.col_display, end.line, end.col_display
    );
  }
}
