extern crate serde;
extern crate serde_json;
extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;
extern crate wasm_bindgen;

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use self::serde::{Deserialize, Serialize, Serializer};
use self::swc_common::{sync::Lrc, SourceMap, SourceMapper, Span};
use swc_common::BytePos;
use wasm_bindgen::prelude::*;

use does_it_throw::call_finder::CallToThrowMap;
use does_it_throw::throw_finder::{IdentifierUsage, ThrowMap};
use does_it_throw::{analyze_code, AnalysisResult, UserSettings};

// Define an extern block with the `console.log` function.
#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

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

#[derive(Deserialize, Debug)]
pub struct DiagnosticSeverityInput(String);

impl FromStr for DiagnosticSeverity {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Error" => Ok(DiagnosticSeverity::Error),
      "Warning" => Ok(DiagnosticSeverity::Warning),
      "Information" => Ok(DiagnosticSeverity::Information),
      "Hint" => Ok(DiagnosticSeverity::Hint),
      _ => Err(()),
    }
  }
}

impl From<DiagnosticSeverityInput> for DiagnosticSeverity {
  fn from(input: DiagnosticSeverityInput) -> Self {
    DiagnosticSeverity::from_str(&input.0).unwrap()
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

fn get_line_start_byte_pos(cm: &SourceMap, lo_byte_pos: BytePos, hi_byte_pos: BytePos) -> BytePos {
  let src = cm
    .span_to_snippet(Span::new(lo_byte_pos, hi_byte_pos, Default::default()))
    .unwrap_or_default();

  // Split the source into lines and reverse the list to find the newline character from the end (which would be the start of the line)
  let lines = src.lines().rev().collect::<Vec<&str>>();

  if let Some(last_line) = lines.first() {
    // Calculate the byte position of the start of the line of interest
    let start_pos = last_line.chars().position(|c| c != ' ' && c != '\t');

    if let Some(pos) = start_pos {
      hi_byte_pos - BytePos((last_line.len() - pos) as u32)
    } else {
      // If there's no content (only whitespace), then we are at the start of the line
      hi_byte_pos - BytePos(last_line.len() as u32)
    }
  } else {
    // If there's no newline character, then we are at the start of the file
    BytePos(0)
  }
}

fn get_relative_imports(import_sources: Vec<String>) -> Vec<String> {
  let mut relative_imports: Vec<String> = Vec::new();
  for import_source in import_sources {
    if import_source.starts_with("./") || import_source.starts_with("../") {
      relative_imports.push(import_source);
    }
  }
  relative_imports
}

#[derive(Serialize)]
pub struct ImportedIdentifiers {
  pub diagnostics: Vec<Diagnostic>,
  pub id: String,
}

pub fn add_diagnostics_for_functions_that_throw(
  diagnostics: &mut Vec<Diagnostic>,
  functions_with_throws: HashSet<ThrowMap>,
  cm: &SourceMap,
  debug: Option<bool>,
  throw_statement_severity: DiagnosticSeverity,
  function_throw_severity: DiagnosticSeverity,
) {
  for fun in &functions_with_throws {
    let function_start = cm.lookup_char_pos(fun.throw_statement.lo());
    let line_end_byte_pos =
      get_line_end_byte_pos(cm, fun.throw_statement.lo(), fun.throw_statement.hi());

    let function_end = cm.lookup_char_pos(line_end_byte_pos - BytePos(1));

    let start_character_byte_pos =
      get_line_start_byte_pos(cm, fun.throw_statement.lo(), fun.throw_statement.hi());
    let start_character = cm.lookup_char_pos(start_character_byte_pos);

    if debug == Some(true) {
      log(&format!("Function throws: {}", fun.function_or_method_name));
      log(&format!(
        "From line {} column {} to line {} column {}",
        function_start.line,
        function_start.col_display,
        function_end.line,
        function_end.col_display
      ));
    }

    diagnostics.push(Diagnostic {
      severity: function_throw_severity.to_int(),
      range: DiagnosticRange {
        start: DiagnosticPosition {
          line: function_start.line - 1,
          character: start_character.col_display,
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
        severity: throw_statement_severity.to_int(),
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
}

pub fn add_diagnostics_for_calls_to_throws(
  diagnostics: &mut Vec<Diagnostic>,
  calls_to_throws: HashSet<CallToThrowMap>,
  cm: &SourceMap,
  debug: Option<bool>,
  call_to_throw_severity: DiagnosticSeverity,
) {
  for call in &calls_to_throws {
    let call_start = cm.lookup_char_pos(call.call_span.lo());

    let line_end_byte_pos = get_line_end_byte_pos(cm, call.call_span.lo(), call.call_span.hi());

    let call_end = cm.lookup_char_pos(line_end_byte_pos - BytePos(1));

    if debug == Some(true) {
      log(&format!(
        "Function call that may throw: {}",
        call.call_function_or_method_name
      ));
      log(&format!(
        "From line {} column {} to line {} column {}",
        call_start.line, call_start.col_display, call_end.line, call_end.col_display
      ));
    }

    diagnostics.push(Diagnostic {
      severity: call_to_throw_severity.to_int(),
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
}

// Multiple calls to the same identifier can result in multiple diagnostics for the same identifier.
// We want to return a diagnostic for all calls to the same identifier, so we need to combine the diagnostics for each identifier.
pub fn identifier_usages_vec_to_combined_map(
  identifier_usages: HashSet<IdentifierUsage>,
  cm: &SourceMap,
  debug: Option<bool>,
  call_to_imported_throw_severity: DiagnosticSeverity,
) -> HashMap<String, ImportedIdentifiers> {
  let mut identifier_usages_map: HashMap<String, ImportedIdentifiers> = HashMap::new();
  for identifier_usage in identifier_usages {
    let identifier_name = identifier_usage.id.clone();
    let start = cm.lookup_char_pos(identifier_usage.usage_span.lo());
    let end = cm.lookup_char_pos(identifier_usage.usage_span.hi());

    if debug == Some(true) {
      log(&format!(
        "Identifier usage: {}",
        identifier_usage.id.clone()
      ));
      log(&format!(
        "From line {} column {} to line {} column {}",
        start.line, start.col_display, end.line, end.col_display
      ));
    }

    let identifier_diagnostics =
      identifier_usages_map
        .entry(identifier_name)
        .or_insert(ImportedIdentifiers {
          diagnostics: Vec::new(),
          id: identifier_usage.id,
        });

    identifier_diagnostics.diagnostics.push(Diagnostic {
      severity: call_to_imported_throw_severity.to_int(),
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
      message: "Function imported that may throw.".to_string(),
      source: "Does it Throw?".to_string(),
    });
  }
  identifier_usages_map
}

#[derive(Serialize)]
pub struct ParseResult {
  pub diagnostics: Vec<Diagnostic>,
  pub relative_imports: Vec<String>,
  pub throw_ids: Vec<String>,
  pub imported_identifiers_diagnostics: HashMap<String, ImportedIdentifiers>,
}

impl ParseResult {
  pub fn into(
    results: AnalysisResult,
    cm: &SourceMap,
    debug: Option<bool>,
    input_data: InputData,
  ) -> ParseResult {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    add_diagnostics_for_functions_that_throw(
      &mut diagnostics,
      results.functions_with_throws.clone(),
      cm,
      debug,
      DiagnosticSeverity::from(
        input_data
          .throw_statement_severity
          .unwrap_or(DiagnosticSeverityInput("Hint".to_string())),
      ),
      DiagnosticSeverity::from(
        input_data
          .function_throw_severity
          .unwrap_or(DiagnosticSeverityInput("Hint".to_string())),
      ),
    );
    add_diagnostics_for_calls_to_throws(
      &mut diagnostics,
      results.calls_to_throws,
      cm,
      debug,
      DiagnosticSeverity::from(
        input_data
          .call_to_throw_severity
          .unwrap_or(DiagnosticSeverityInput("Hint".to_string())),
      ),
    );

    ParseResult {
      diagnostics,
      throw_ids: results
        .functions_with_throws
        .into_iter()
        .map(|f| f.id)
        .collect(),
      relative_imports: get_relative_imports(results.import_sources.into_iter().collect()),
      imported_identifiers_diagnostics: identifier_usages_vec_to_combined_map(
        results.imported_identifier_usages,
        cm,
        debug,
        DiagnosticSeverity::from(
          input_data
            .call_to_imported_throw_severity
            .unwrap_or(DiagnosticSeverityInput("Hint".to_string())),
        ),
      ),
    }
  }
}

#[wasm_bindgen(typescript_custom_section)]
const TypeScriptSettings: &'static str = r#"
interface TypeScriptSettings {
	decorators?: boolean;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const DiagnosticSeverityInput: &'static str = r#"
type DiagnosticSeverityInput = "Error" | "Warning" | "Information" | "Hint";
"#;

#[wasm_bindgen(typescript_custom_section)]
const InputData: &'static str = r#"
interface InputData {
	uri: string;
	file_content: string;
	typescript_settings?: TypeScriptSettings;
	ids_to_check: string[];
	debug?: boolean;
  throw_statement_severity?: DiagnosticSeverityInput;
  function_throw_severity?: DiagnosticSeverityInput;
  call_to_throw_severity?: DiagnosticSeverityInput;
  call_to_imported_throw_severity?: DiagnosticSeverityInput;
  include_try_statement_throws?: boolean;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ImportedIdentifiers: &'static str = r#"
interface ImportedIdentifiers {
	diagnostics: any[];
	id: string;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ParseResult: &'static str = r#"
interface ParseResult {
	diagnostics: any[];
	relative_imports: string[];
	throw_ids: string[];
	imported_identifiers_diagnostics: Map<string, ImportedIdentifiers>;
}
"#;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(typescript_type = "ParseResult")]
  pub type ParseResultType;
}

#[derive(Serialize, Deserialize)]
pub struct TypeScriptSettings {
  decorators: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct InputData {
  // TODO - maybe use this in the future
  // uri: String,
  // typescript_settings: Option<TypeScriptSettings>,
  // ids_to_check: Vec<String>,
  pub file_content: String,
  pub debug: Option<bool>,
  pub throw_statement_severity: Option<DiagnosticSeverityInput>,
  pub function_throw_severity: Option<DiagnosticSeverityInput>,
  pub call_to_throw_severity: Option<DiagnosticSeverityInput>,
  pub call_to_imported_throw_severity: Option<DiagnosticSeverityInput>,
  pub include_try_statement_throws: Option<bool>,
}

#[wasm_bindgen]
pub fn parse_js(data: JsValue) -> JsValue {
  // Parse the input data into a Rust struct.
  let input_data: InputData = serde_wasm_bindgen::from_value(data).unwrap();

  let cm: Lrc<SourceMap> = Default::default();

  let user_settings = UserSettings {
    include_try_statement_throws: input_data.include_try_statement_throws.unwrap_or(false),
  };

  let (results, cm) = analyze_code(&input_data.file_content, cm, &user_settings);

  let parse_result = ParseResult::into(results, &cm, input_data.debug, input_data);

  // Convert the diagnostics to a JsValue and return it.
  serde_wasm_bindgen::to_value(&parse_result).unwrap()
}

#[cfg(test)]
mod tests {

  use super::*;
  use swc_common::FileName;

  #[test]
  fn test_get_line_end_byte_pos_with_newline() {
    let cm = Lrc::new(SourceMap::default());
    let source_file = cm.new_source_file(
      FileName::Custom("test_file".into()),
      "line 1\nline 2".into(),
    );

    let lo_byte_pos = source_file.start_pos;
    let hi_byte_pos = BytePos(source_file.end_pos.0 + 10);

    let result = get_line_end_byte_pos(&cm, lo_byte_pos, hi_byte_pos);
    assert_eq!(result, BytePos(24));
  }

  #[test]
  fn test_get_line_end_byte_pos_without_newline() {
    let cm = Lrc::new(SourceMap::default());
    let source_file = cm.new_source_file(FileName::Custom("test_file".into()), "no newline".into());

    let lo_byte_pos = source_file.start_pos;
    let hi_byte_pos = BytePos(source_file.end_pos.0 + 10);

    let result = get_line_end_byte_pos(&cm, lo_byte_pos, hi_byte_pos);
    assert_eq!(result, hi_byte_pos);
  }

  #[test]
  fn test_get_line_end_byte_pos_none_snippet() {
    let cm = Lrc::new(SourceMap::default());
    let source_file = cm.new_source_file(FileName::Custom("test_file".into()), "".into());

    let lo_byte_pos = source_file.start_pos;
    let hi_byte_pos = BytePos(source_file.end_pos.0 + 10);

    let result = get_line_end_byte_pos(&cm, lo_byte_pos, hi_byte_pos);
    assert_eq!(result, hi_byte_pos);
  }

  #[test]
  fn test_get_line_start_byte_pos_with_content() {
    let cm = Lrc::new(SourceMap::default());
    cm.new_source_file(
      FileName::Custom("test_file".into()),
      "line 1\n    line 2\nline 3".into(),
    );

    let lo_byte_pos = BytePos(19);
    let hi_byte_pos = BytePos(7);

    let result = get_line_start_byte_pos(&cm, lo_byte_pos, hi_byte_pos);
    assert_eq!(result, BytePos(1));
  }

  #[test]
  fn test_get_line_start_byte_pos_without_content() {
    let cm = Lrc::new(SourceMap::default());
    cm.new_source_file(
      FileName::Custom("test_file".into()),
      "line 1\n    \nline 3".into(),
    );

    let lo_byte_pos = BytePos(1);
    let hi_byte_pos = BytePos(11);

    let result = get_line_start_byte_pos(&cm, lo_byte_pos, hi_byte_pos);
    assert_eq!(result, BytePos(8));
  }

  #[test]
  fn test_get_line_start_byte_pos_at_file_start() {
    let cm = Lrc::new(SourceMap::default());
    cm.new_source_file(
      FileName::Custom("test_file".into()),
      "line 1\nline 2\nline 3".into(),
    );

    let lo_byte_pos = BytePos(0);
    let hi_byte_pos = BytePos(5);

    let result = get_line_start_byte_pos(&cm, lo_byte_pos, hi_byte_pos);
    assert_eq!(result, BytePos(0));
  }

  #[test]
  fn test_get_relative_imports() {
    let import_sources = vec![
      "./relative/path".to_string(),
      "../relative/path".to_string(),
      "/absolute/path".to_string(),
      "http://example.com".to_string(),
      "https://example.com".to_string(),
      "package".to_string(),
    ];

    let expected_relative_imports = vec![
      "./relative/path".to_string(),
      "../relative/path".to_string(),
    ];

    let relative_imports = get_relative_imports(import_sources);
    assert_eq!(relative_imports, expected_relative_imports);
  }

  #[test]
  fn test_add_diagnostics_for_functions_that_throw_single() {
    let cm = Lrc::new(SourceMap::default());
    let source_file = cm.new_source_file(
      FileName::Custom("test_file".into()),
      "function foo() {\n  throw new Error();\n}".into(),
    );

    let throw_span = Span::new(
      source_file.start_pos + BytePos(13),
      source_file.start_pos + BytePos(30),
      Default::default(),
    );

    let functions_with_throws = HashSet::from([ThrowMap {
      throw_statement: throw_span,
      throw_spans: vec![throw_span],
      function_or_method_name: "foo".to_string(),
      class_name: None,
      id: "foo".to_string(),
    }]);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    add_diagnostics_for_functions_that_throw(
      &mut diagnostics,
      functions_with_throws,
      &cm,
      None,
      DiagnosticSeverity::Hint,
      DiagnosticSeverity::Hint,
    );

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Hint.to_int());
    assert_eq!(diagnostics[0].message, "Function that may throw.");
  }

  #[test]
  fn test_add_diagnostics_for_functions_that_throw_multiple() {
    let cm = Lrc::new(SourceMap::default());
    let source_file = cm.new_source_file(
      FileName::Custom("test_file".into()),
      "function foo() {\n  throw new Error('First');\n  throw new Error('Second');\n}".into(),
    );

    let first_throw_span = Span::new(
      source_file.start_pos + BytePos(13),
      source_file.start_pos + BytePos(35),
      Default::default(),
    );

    let second_throw_span = Span::new(
      source_file.start_pos + BytePos(39),
      source_file.start_pos + BytePos(62),
      Default::default(),
    );

    let functions_with_throws = HashSet::from([ThrowMap {
      throw_statement: first_throw_span,
      throw_spans: vec![first_throw_span, second_throw_span],
      function_or_method_name: "foo".to_string(),
      class_name: None,
      id: "foo".to_string(),
    }]);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    add_diagnostics_for_functions_that_throw(
      &mut diagnostics,
      functions_with_throws,
      &cm,
      None,
      DiagnosticSeverity::Hint,
      DiagnosticSeverity::Hint,
    );

    assert_eq!(diagnostics.len(), 3);

    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Hint.to_int());
    assert_eq!(diagnostics[0].message, "Function that may throw.");

    assert_eq!(diagnostics[1].severity, DiagnosticSeverity::Hint.to_int());
    assert_eq!(diagnostics[1].message, "Throw statement.");

    assert_eq!(diagnostics[2].severity, DiagnosticSeverity::Hint.to_int());
    assert_eq!(diagnostics[2].message, "Throw statement.");
  }

  #[test]
  fn test_add_diagnostics_for_calls_to_throws() {
    let cm = Lrc::new(SourceMap::default());
    let source_file = cm.new_source_file(
      FileName::Custom("test_file".into()),
      "function foo() {\n  throw new Error();\n}".into(),
    );

    let call_span = Span::new(
      source_file.start_pos + BytePos(13),
      source_file.start_pos + BytePos(30),
      Default::default(),
    );

    let call_to_throws = HashSet::from([CallToThrowMap {
      call_span,
      call_function_or_method_name: "foo".to_string(),
      call_class_name: None,
      class_name: None,
      id: "foo".to_string(),
      throw_map: ThrowMap {
        throw_statement: Span::new(
          source_file.start_pos + BytePos(13),
          source_file.start_pos + BytePos(30),
          Default::default(),
        ),
        throw_spans: vec![],
        function_or_method_name: "foo".to_string(),
        class_name: None,
        id: "foo".to_string(),
      },
    }]);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    add_diagnostics_for_calls_to_throws(
      &mut diagnostics,
      call_to_throws,
      &cm,
      None,
      DiagnosticSeverity::Hint,
    );

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Hint.to_int());
    assert_eq!(diagnostics[0].message, "Function call that may throw.");
    assert_eq!(diagnostics[0].range.start.line, 0);
    assert_eq!(diagnostics[0].range.start.character, 13);
    assert_eq!(diagnostics[0].range.end.line, 0);
    assert_eq!(diagnostics[0].range.end.character, 15);
  }
  #[test]
  fn test_no_calls_to_throws() {
    let cm = Lrc::new(SourceMap::default());
    cm.new_source_file(
      FileName::Custom("test_file".into()),
      "function foo() {\n  console.log('No throw');\n}".into(),
    );

    let call_to_throws = HashSet::new();

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    add_diagnostics_for_calls_to_throws(
      &mut diagnostics,
      call_to_throws,
      &cm,
      None,
      DiagnosticSeverity::Hint,
    );

    assert!(diagnostics.is_empty());
  }

  #[test]
  fn test_multiple_calls_to_throws() {
    let cm = Lrc::new(SourceMap::default());
    let source_file = cm.new_source_file(
      FileName::Custom("test_file".into()),
      "function foo() {\n  throw new Error();\n}\nfunction bar() {\n  throw new Error();\n}".into(),
    );

    let call_span_foo = Span::new(
      source_file.start_pos + BytePos(13),
      source_file.start_pos + BytePos(30),
      Default::default(),
    );

    let call_span_bar = Span::new(
      source_file.start_pos + BytePos(52),
      source_file.start_pos + BytePos(69),
      Default::default(),
    );

    let call_to_throws = HashSet::from([
      CallToThrowMap {
        call_span: call_span_foo,
        call_function_or_method_name: "foo".to_string(),
        call_class_name: None,
        class_name: None,
        id: "foo".to_string(),
        throw_map: ThrowMap {
          throw_statement: Span::new(
            source_file.start_pos + BytePos(13),
            source_file.start_pos + BytePos(30),
            Default::default(),
          ),
          throw_spans: vec![],
          function_or_method_name: "foo".to_string(),
          class_name: None,
          id: "foo".to_string(),
        },
      },
      CallToThrowMap {
        call_span: call_span_bar,
        call_function_or_method_name: "bar".to_string(),
        call_class_name: None,
        class_name: None,
        id: "foo".to_string(),
        throw_map: ThrowMap {
          throw_statement: Span::new(
            source_file.start_pos + BytePos(13),
            source_file.start_pos + BytePos(30),
            Default::default(),
          ),
          throw_spans: vec![],
          function_or_method_name: "foo".to_string(),
          class_name: None,
          id: "foo".to_string(),
        },
      },
    ]);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    add_diagnostics_for_calls_to_throws(
      &mut diagnostics,
      call_to_throws,
      &cm,
      None,
      DiagnosticSeverity::Hint,
    );

    assert_eq!(diagnostics.len(), 2);
  }

  #[test]
  fn test_identifier_usages_vec_to_combined_map_multiple_usages_same_identifier() {
    let cm = Lrc::new(SourceMap::default());
    let source_file = cm.new_source_file(
      FileName::Custom("test_file".into()),
      "import {foo} from 'module'; foo(); foo();".into(),
    );

    let first_usage_span = Span::new(
      source_file.start_pos + BytePos(17),
      source_file.start_pos + BytePos(20),
      Default::default(),
    );

    let second_usage_span = Span::new(
      source_file.start_pos + BytePos(22),
      source_file.start_pos + BytePos(25),
      Default::default(),
    );

    let identifier_usages = HashSet::from([
      IdentifierUsage {
        id: "foo".to_string(),
        usage_span: first_usage_span,
        identifier_name: "foo".to_string(),
        usage_context: "import".to_string(),
      },
      IdentifierUsage {
        id: "foo".to_string(),
        usage_span: second_usage_span,
        identifier_name: "foo".to_string(),
        usage_context: "import".to_string(),
      },
    ]);

    let combined_map =
      identifier_usages_vec_to_combined_map(identifier_usages, &cm, None, DiagnosticSeverity::Hint);

    assert_eq!(combined_map.len(), 1);

    let foo_diagnostics = &combined_map.get("foo").unwrap().diagnostics;
    assert_eq!(foo_diagnostics.len(), 2);

    assert_eq!(
      foo_diagnostics[0].severity,
      DiagnosticSeverity::Hint.to_int()
    );
    assert_eq!(
      foo_diagnostics[0].message,
      "Function imported that may throw."
    );

    assert_eq!(
      foo_diagnostics[1].severity,
      DiagnosticSeverity::Hint.to_int()
    );
    assert_eq!(
      foo_diagnostics[1].message,
      "Function imported that may throw."
    );
  }

  #[test]
  fn test_should_include_throws_in_try_statement() {
    // smoke test to ensure backwards compatibility
    let cm = Lrc::new(SourceMap::default());
    let source_file = cm.new_source_file(
      FileName::Custom("test_file".into()),
      "function foo() {\n  try {\n    throw new Error();\n  } catch (e) {\n    throw e;\n  }\n}".into(),
    );

    let throw_span = Span::new(
      source_file.start_pos + BytePos(34),
      source_file.start_pos + BytePos(51),
      Default::default(),
    );

    let functions_with_throws = HashSet::from([ThrowMap {
      throw_statement: throw_span,
      throw_spans: vec![throw_span],
      function_or_method_name: "foo".to_string(),
      class_name: None,
      id: "foo".to_string(),
    }]);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    add_diagnostics_for_functions_that_throw(
      &mut diagnostics,
      functions_with_throws,
      &cm,
      None,
      DiagnosticSeverity::Hint,
      DiagnosticSeverity::Hint,
    );

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Hint.to_int());
    assert_eq!(diagnostics[0].message, "Function that may throw.");
  }
}
