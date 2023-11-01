extern crate serde;
extern crate serde_json;
extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;
extern crate wasm_bindgen;

use self::serde::{Serialize, Serializer};
use self::swc_common::{sync::Lrc, SourceMap, SourceMapper, Span};
use swc_common::BytePos;
use wasm_bindgen::prelude::*;

use does_it_throw::analyze_code;

// Define an extern block with the `console.log` function.
#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

// TODO - Maybe don't serialize the whole diagnostic struct, just the fields we need, or not at all

#[derive(Serialize)]
pub struct Diagnostic {
  severity: i32,
  range: DiagnosticRange,
  message: String,
  source: String,
}

#[derive(Serialize)]
pub struct DiagnosticRange {
  start: DiagnosticPosition,
  end: DiagnosticPosition,
}

#[derive(Serialize)]
pub struct DiagnosticPosition {
  line: usize,
  character: usize,
}

#[derive(Copy, Clone)]
pub enum DiagnosticSeverity {
  Error = 0,
  Warning = 1,
  Information = 2,
  Hint = 3,
}

impl Serialize for DiagnosticSeverity {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_i32(*self as i32)
  }
}

impl DiagnosticSeverity {
  fn to_int(&self) -> i32 {
    match *self {
      DiagnosticSeverity::Error => 0,
      DiagnosticSeverity::Warning => 1,
      DiagnosticSeverity::Information => 2,
      DiagnosticSeverity::Hint => 3,
    }
  }
}

fn get_line_end_byte_pos(cm: &SourceMap, lo_byte_pos: BytePos, hi_byte_pos: BytePos) -> BytePos {
  let src = cm
    .span_to_snippet(Span::new(lo_byte_pos, hi_byte_pos, Default::default()))
    .unwrap_or_default();

  if let Some(newline_pos) = src.find('\n') {
    lo_byte_pos + BytePos(newline_pos as u32)
  } else {
    // should never be true
    hi_byte_pos
  }
}

#[wasm_bindgen]
pub fn parse_js(js_code: &str) -> String {
  let cm: Lrc<SourceMap> = Default::default();

  let (results, cm) = analyze_code(js_code, cm);
  for import in results.import_sources.into_iter() {
    log(&format!("Import source: {}", import));
  }
  for identifier in results.imported_identifiers.into_iter() {
    log(&format!("Imported identifier: {}", identifier));
  }

  // TODO - Remove this or put behind debug flag
  // println!("Functions that throw:");
  // for fun in &results.functions_with_throws {
  //   let start = cm.lookup_char_pos(fun.throw_statement.lo());
  //   let end = cm.lookup_char_pos(fun.throw_statement.hi());
  // 	log(&format!(
  // 		"Function throws: {}", fun.function_or_method_name
  // 	));
  //   log(&format!(
  //     "From line {} column {} to line {} column {}",
  //     start.line, start.col_display, end.line, end.col_display
  //   ));
  // }
  //   for span in &fun.throw_spans {
  //     let start = cm.lookup_char_pos(span.lo());
  //     let end = cm.lookup_char_pos(span.hi());
  //     log(&format!(
  //       "  Throw from line {} column {} to line {} column {}",
  //       start.line, start.col_display, end.line, end.col_display
  //     ));
  //   }
  // }
  // format!("Functions that throw parsed");

  // Transform results into diagnostics
  let mut diagnostics: Vec<Diagnostic> = Vec::new();

  for fun in &results.functions_with_throws {
    let function_start = cm.lookup_char_pos(fun.throw_statement.lo());
    let line_end_byte_pos =
      get_line_end_byte_pos(&cm, fun.throw_statement.lo(), fun.throw_statement.hi());

    let function_end = cm.lookup_char_pos(line_end_byte_pos - BytePos(1));

    diagnostics.push(Diagnostic {
      severity: DiagnosticSeverity::Hint.to_int(),
      range: DiagnosticRange {
        start: DiagnosticPosition {
          line: function_start.line - 1,
          character: function_start.col_display,
        },
        end: DiagnosticPosition {
          line: function_end.line - 1,
          character: function_end.col_display,
        },
      },
      message: "Function that may throw.".to_string(),
      source: "Does it Throw?".to_string(),
    });

    for span in &fun.throw_spans {
      let start = cm.lookup_char_pos(span.lo());
      let end = cm.lookup_char_pos(span.hi());

      diagnostics.push(Diagnostic {
        severity: DiagnosticSeverity::Information.to_int(),
        range: DiagnosticRange {
          start: DiagnosticPosition {
            line: start.line - 1,
            character: start.col_display,
          },
          end: DiagnosticPosition {
            line: end.line - 1,
            character: end.col_display,
          },
        },
        message: "Throw statement.".to_string(),
        source: "Does it Throw?".to_string(),
      });
    }
  }

  for call in &results.calls_to_throws {
    let call_start = cm.lookup_char_pos(call.call_span.lo());

    let call_end = cm.lookup_char_pos(call.call_span.hi());

    diagnostics.push(Diagnostic {
      severity: DiagnosticSeverity::Hint.to_int(),
      range: DiagnosticRange {
        start: DiagnosticPosition {
          line: call_start.line - 1,
          character: call_start.col_display,
        },
        end: DiagnosticPosition {
          line: call_end.line - 1,
          character: call_end.col_display,
        },
      },
      message: "Function call that may throw.".to_string(),
      source: "Does it Throw?".to_string(),
    });
  }

  // Convert the diagnostics to a JSON string
  serde_json::to_string(&diagnostics).unwrap()
}
