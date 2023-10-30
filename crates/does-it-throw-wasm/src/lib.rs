

extern crate serde;
extern crate serde_json;
extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;
extern crate wasm_bindgen;

use self::serde::{Serialize, Serializer};
use self::swc_common::{sync::Lrc, SourceMap, SourceMapper, Span};
use self::swc_ecma_ast::*;
use self::swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use self::swc_ecma_visit::Visit;
use swc_common::BytePos;
use wasm_bindgen::prelude::*;

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
            DiagnosticSeverity::Hint => 3
        }
    }
}

fn get_line_end_byte_pos(cm: &SourceMap, lo_byte_pos: BytePos, hi_byte_pos: BytePos) -> BytePos {
	let src = cm.span_to_snippet(Span::new(lo_byte_pos, hi_byte_pos, Default::default())).unwrap_or_default();

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

	// TODO - Remove this or put behind debug flag
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
  format!("Functions that throw parsed");

	// Transform results into diagnostics
	let mut diagnostics: Vec<Diagnostic> = Vec::new();

	for fun in &results.functions_with_throws {
			let function_start = cm.lookup_char_pos(fun.throw_statement.lo());
			let line_end_byte_pos = get_line_end_byte_pos(&cm, fun.throw_statement.lo(), fun.throw_statement.hi());

			let function_end = cm.lookup_char_pos(line_end_byte_pos - BytePos(1));

			diagnostics.push(Diagnostic {
					severity: DiagnosticSeverity::Information.to_int(),
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
											line: end.line -1 ,
											character: end.col_display,
									},
							},
							message: "Throw statement.".to_string(),
							source: "Does it Throw?".to_string(),
					});
			}
	}

	// Convert the diagnostics to a JSON string
	serde_json::to_string(&diagnostics).unwrap()
}

pub fn analyze_code(content: &str, cm: Lrc<SourceMap>) -> (AnalysisResult, Lrc<SourceMap>) {
  let fm = cm.new_source_file(swc_common::FileName::Anon, content.into());
  let lexer = Lexer::new(
    Syntax::Typescript(swc_ecma_parser::TsConfig {
      tsx: false,
      decorators: false,
      dts: false,
      no_early_errors: false,
      disallow_ambiguous_jsx_like: true,
    }),
    EsVersion::latest(),
    StringInput::from(&*fm),
    None,
  );

  let mut parser = Parser::new_from(lexer);
  let module = parser.parse_module().expect("Failed to parse module");

  let mut collector = CodeAnalyzer {
    functions_with_throws: vec![],
    json_parse_calls: vec![],
    fs_access_calls: vec![],
  };
  collector.visit_module(&module);

  (collector.into(), cm)
}

struct CodeAnalyzer {
  functions_with_throws: Vec<ThrowMap>,
  json_parse_calls: Vec<String>,
  fs_access_calls: Vec<String>,
}

impl CodeAnalyzer {
  fn check_function_for_throws(&mut self, function: &Function) {
    let mut throw_finder = ThrowFinder {
      throw_spans: vec![],
    };
    throw_finder.visit_function(function);

    if !throw_finder.throw_spans.is_empty() {
      let throw_map = ThrowMap {
        throw_spans: throw_finder.throw_spans,
        throw_statement: function.span,
      };
      self.functions_with_throws.push(throw_map);
    }
  }
}

impl Visit for CodeAnalyzer {
  fn visit_function(&mut self, function: &Function) {
    self.check_function_for_throws(function);
    swc_ecma_visit::visit_function(self, function);
  }

  fn visit_call_expr(&mut self, call: &CallExpr) {
    if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
      if let Expr::Member(member) = &**expr {
        if let Expr::Ident(object) = &*member.obj {
          match object.sym.as_ref() {
            "JSON" => {
              if let MemberProp::Ident(property) = &member.prop {
                if property.sym == "parse" {
                  let call_info = format!("{:?}", call.span);
                  self.json_parse_calls.push(call_info);
                }
              }
            }
            "fs" => {
              if let MemberProp::Ident(property) = &member.prop {
                if property.sym == "access" {
                  let call_info = format!("{:?}", call.span);
                  self.fs_access_calls.push(call_info);
                }
              }
            }

            _ => {}
          }
        }
      }
    }
  }
}

struct ThrowFinder {
  throw_spans: Vec<Span>,
}

impl Visit for ThrowFinder {
  fn visit_throw_stmt(&mut self, node: &ThrowStmt) {
    self.throw_spans.push(node.span)
  }
}

#[derive(Default)]

pub struct ThrowMap {
  pub throw_spans: Vec<Span>,
  pub throw_statement: Span,
}

pub struct AnalysisResult {
  pub functions_with_throws: Vec<ThrowMap>,
  pub json_parse_calls: Vec<String>,
  pub fs_access_calls: Vec<String>,
}

impl From<CodeAnalyzer> for AnalysisResult {
  fn from(analyzer: CodeAnalyzer) -> Self {
    Self {
      functions_with_throws: analyzer.functions_with_throws,
      json_parse_calls: analyzer.json_parse_calls,
      fs_access_calls: analyzer.fs_access_calls,
    }
  }
}
