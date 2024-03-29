extern crate does_it_throw;
extern crate swc_common;
use std::fs;

use self::swc_common::{sync::Lrc, SourceMap};
use does_it_throw::{analyze_code, UserSettings};

pub fn main() {
  let sample_code = fs::read_to_string("crates/does-it-throw/src/fixtures/sample.ts")
    .expect("Something went wrong reading the file");
  let cm: Lrc<SourceMap> = Default::default();
  let user_settings = UserSettings {
    include_try_statement_throws: false,
    ignore_statements: vec!["@it-throws".to_string()],
  };
  let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);
  for import in result.import_sources.into_iter() {
    println!("Imported {}", import);
  }
  for fun in result.functions_with_throws.clone().into_iter() {
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

  for throw_id in result.functions_with_throws.into_iter() {
    println!("throw id: {}", throw_id.id);
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

#[cfg(test)]
mod integration_tests {
  use std::env;

  use super::*;
  #[test]
  fn test_ts_class() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/class.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 4);
    assert_eq!(result.calls_to_throws.len(), 5);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }
    assert!(function_names_contains(
      &function_names,
      "someMethodThatThrows"
    ));
    assert!(function_names_contains(
      &function_names,
      "someMethodThatThrows2"
    ));
    assert!(function_names_contains(&function_names, "nestedThrow"));
    assert!(function_names_contains(&function_names, "<constructor>"));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();
    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    assert!(calls_to_throws_contains(
      &calls_to_throws,
      "NOT_SET-callNestedThrow"
    ));
    assert!(calls_to_throws_contains(
      &calls_to_throws,
      "Something-_somethingCall"
    ));
    assert!(calls_to_throws_contains(
      &calls_to_throws,
      "Something-_somethingCall2"
    ));
  }

  #[test]
  fn test_js_class() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/class.js", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 4);
    assert_eq!(result.calls_to_throws.len(), 5);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }
    assert!(function_names_contains(
      &function_names,
      "someMethodThatThrows"
    ));
    assert!(function_names_contains(
      &function_names,
      "someMethodThatThrows2"
    ));
    assert!(function_names_contains(&function_names, "nestedThrow"));
    assert!(function_names_contains(&function_names, "<constructor>"));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();
    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    assert!(calls_to_throws_contains(
      &calls_to_throws,
      "NOT_SET-callNestedThrow"
    ));
    assert!(calls_to_throws_contains(
      &calls_to_throws,
      "Something-_somethingCall"
    ));
    assert!(calls_to_throws_contains(
      &calls_to_throws,
      "Something-_somethingCall2"
    ));
  }

  #[test]
  fn test_exports() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/exports.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 4);
    assert_eq!(result.calls_to_throws.len(), 4);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }
    [
      "hiKhue",
      "someConstThatThrows2",
      "someConstThatThrows",
      "_ConstThatThrows",
    ]
    .iter()
    .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    [
      "NOT_SET-callToConstThatThrows2",
      "NOT_SET-callToConstThatThrows3",
      "NOT_SET-callToConstThatThrows",
      "NOT_SET-callToConstThatThrows4",
    ]
    .iter()
    .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }

  #[test]
  fn test_js_exports() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/exports.js", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 4);
    assert_eq!(result.calls_to_throws.len(), 4);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }
    [
      "hiKhue",
      "someConstThatThrows2",
      "someConstThatThrows",
      "_ConstThatThrows",
    ]
    .iter()
    .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    [
      "NOT_SET-callToConstThatThrows2",
      "NOT_SET-callToConstThatThrows3",
      "NOT_SET-callToConstThatThrows",
      "NOT_SET-callToConstThatThrows4",
    ]
    .iter()
    .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }

  #[test]

  fn test_object_literal() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/objectLiteral.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 3);
    assert_eq!(result.calls_to_throws.len(), 4);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }
    [
      "someExampleThrow",
      "objectLiteralThrow",
      "nestedObjectLiteralThrow",
    ]
    .iter()
    .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    [
      "someObjectLiteral-callToLiteral",
      "NOT_SET-callToLiteral3",
      "someObjectLiteral-callToLiteral2",
      "SomeObject-callToLiteral3",
    ]
    .iter()
    .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }

  #[test]

  fn test_js_object_literal() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/objectLiteral.js", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 3);
    assert_eq!(result.calls_to_throws.len(), 4);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }
    [
      "someExampleThrow",
      "objectLiteralThrow",
      "nestedObjectLiteralThrow",
    ]
    .iter()
    .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    [
      "someObjectLiteral-callToLiteral",
      "NOT_SET-callToLiteral3",
      "someObjectLiteral-callToLiteral2",
      "SomeObject-callToLiteral3",
    ]
    .iter()
    .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }

  #[test]
  fn test_call_expr() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/callExpr.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 5);
    assert_eq!(result.calls_to_throws.len(), 10);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }
    [
      "onInitialized2",
      "SomeThrow2",
      "SomeThrow",
      "SomeRandomCall",
      "oneWithASecondArg",
    ]
    .iter()
    .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();
    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    [
      "connection-<anonymous>",
      "NOT_SET-<anonymous>",
      "NOT_SET-onInitialized",
      "NOT_SET-<anonymous>",
      "NOT_SET-SomeRandomCall2",
      "NOT_SET-SomeRandomCall2",
      "NOT_SET-onInitialized",
      "NOT_SET-<anonymous>",
      "connection-<anonymous>",
      "NOT_SET-<anonymous>",
    ]
    .iter()
    .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }

  #[test]
  fn test_tsx() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/tsx.tsx", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 4);
    assert_eq!(result.calls_to_throws.len(), 4);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }

    ["someTsx", "someThrow2", "someThrow", "someAsyncTsx"]
      .iter()
      .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    [
      "NOT_SET-callToThrow",
      "NOT_SET-someTsxWithJsx",
      "NOT_SET-callToThrow",
      "NOT_SET-someTsxWithJsx",
    ]
    .iter()
    .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }

  #[test]
  fn test_jsx() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/jsx.jsx", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 4);
    assert_eq!(result.calls_to_throws.len(), 4);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }

    ["someTsx", "someThrow2", "someThrow", "someAsyncTsx"]
      .iter()
      .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    [
      "NOT_SET-callToThrow",
      "NOT_SET-someTsxWithJsx",
      "NOT_SET-callToThrow",
      "NOT_SET-someTsxWithJsx",
    ]
    .iter()
    .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }

  #[test]
  fn imported_identifiers() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/importIdentifiers.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 0);
    assert_eq!(result.calls_to_throws.len(), 0);
    assert_eq!(result.imported_identifier_usages.len(), 5);
    assert_eq!(result.import_sources.len(), 5);

    let imported_identifier_usages = result
      .imported_identifier_usages
      .into_iter()
      .map(|i| i.id)
      .collect::<Vec<String>>();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }
    [
      "NOT_SET-Something",
      "NOT_SET-Testing",
      "NOT_SET-SomethingElse",
      "NOT_SET-resolve",
      "NOT_SET-SomeThrow",
    ]
    .iter()
    .for_each(|f| assert!(function_names_contains(&imported_identifier_usages, f)));

    let import_sources = result.import_sources.into_iter().collect::<Vec<String>>();
    fn import_sources_contains(import_sources: &Vec<String>, import_source: &str) -> bool {
      import_sources.iter().any(|f| f == import_source)
    }
    [
      "./something3",
      "path",
      "./somethingElse2",
      "./something",
      "./somethingElse",
    ]
    .iter()
    .for_each(|f| assert!(import_sources_contains(&import_sources, f)));
  }

  #[test]
  fn test_spread_expr() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/spreadExpr.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 2);
    assert_eq!(result.calls_to_throws.len(), 2);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }
    ["_contextFromWorkflow", "_contextFromWorkflow"]
      .iter()
      .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }
    ["SomeClass-someCallToThrow", "SomeClass-someCallToThrow"]
      .iter()
      .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }

  #[test]
  fn test_switch_statement() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/switchStatement.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 2);
    assert_eq!(result.calls_to_throws.len(), 2);
    assert_eq!(result.imported_identifier_usages.len(), 2);
    assert_eq!(result.import_sources.len(), 1);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .iter()
      .map(|f| f.function_or_method_name.clone())
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }

    ["someRandomThrow", "createServer"]
      .iter()
      .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result
      .calls_to_throws
      .iter()
      .map(|c| c.id.clone())
      .collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }

    ["NOT_SET-createServer", "http-<anonymous>"]
      .iter()
      .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));

    let import_sources = result.import_sources.into_iter().collect::<Vec<String>>();
    fn import_sources_contains(import_sources: &Vec<String>, import_source: &str) -> bool {
      import_sources.iter().any(|f| f == import_source)
    }
    ["./something"]
      .iter()
      .for_each(|f| assert!(import_sources_contains(&import_sources, f)));

    let import_identifiers = result
      .imported_identifier_usages
      .into_iter()
      .map(|i| i.id)
      .collect::<Vec<String>>();
    fn import_identifiers_contains(
      import_identifiers: &Vec<String>,
      import_identifier: &str,
    ) -> bool {
      import_identifiers.iter().any(|f| f == import_identifier)
    }
    ["someObjectLiteral-objectLiteralThrow", "NOT_SET-SomeThrow"]
      .iter()
      .for_each(|f| assert!(import_identifiers_contains(&import_identifiers, f)));
  }

  #[test]
  fn test_try_statement_including_all_throws() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/tryStatement.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: true,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 8);
    assert_eq!(result.calls_to_throws.len(), 9);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);

    // function names
    let function_names: Vec<String> = result
      .functions_with_throws
      .clone()
      .into_iter()
      .map(|f| f.function_or_method_name)
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }

    [
      "someMethodThatThrows",
      "_contextFromWorkflow",
      "createServer",
      "<constructor>",
      "someConstThatThrows",
      "nestedThrow",
      "_contextFromWorkflow",
      "someMethodThatThrows2",
    ]
    .iter()
    .for_each(|f| assert!(function_names_contains(&function_names, f)));

    // calls to throws
    let calls_to_throws: Vec<String> = result.calls_to_throws.into_iter().map(|c| c.id).collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }

    [
      "Something-somethingCall",
      "Something-_somethingCall2",
      "NOT_SET-someCallToThrow",
      "Something-somethingCall2",
      "http-<anonymous>",
      "Something-_somethingCall",
      "NOT_SET-callNestedThrow",
      "NOT_SET-callToConstThatThrows4",
      "NOT_SET-someCallToThrow",
    ]
    .iter()
    .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }

  #[test]
  fn test_try_statement_does_not_include_all_throws() {
    // This test is the same as test_try_statement but with include_try_statement_throws set to false
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/tryStatement.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    assert_eq!(result.functions_with_throws.len(), 1);
    assert_eq!(result.calls_to_throws.len(), 0);
  }
  #[test]
  fn test_try_statement_nested_including_all_throws() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/tryStatementNested.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: true,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    // general result assertions
    assert_eq!(result.functions_with_throws.len(), 4);
    assert_eq!(result.calls_to_throws.len(), 0);
    assert_eq!(result.imported_identifier_usages.len(), 0);
    assert_eq!(result.import_sources.len(), 0);
  }

  #[test]
  fn test_try_statement_nested_does_not_include_throws() {
    // We need to test the following conditions:
    // 1. include_try_statement_throws = false
    // 2. a nested try statement that throws that ISNT caught by the parent try statement
    // 3. a nested try statement that throws that IS caught by the parent try statement
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/tryStatementNested.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: false,
      ignore_statements: vec!["@it-throws".to_string()],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    assert_eq!(result.functions_with_throws.len(), 2);
    assert_eq!(result.calls_to_throws.len(), 0);

    let function_names: Vec<String> = result
      .functions_with_throws
      .clone()
      .into_iter()
      .map(|f| f.function_or_method_name)
      .collect();
    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }

    ["parentCatchThatisNotCaught", "throwInsideCatch"]
      .iter()
      .for_each(|f| assert!(function_names_contains(&function_names, f)));
  }

  #[test]
  fn test_should_include_throws_for_no_ignore_statements() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/ignoreStatements.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: true,
      ignore_statements: vec![],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    assert_eq!(result.functions_with_throws.len(), 11);
    assert_eq!(result.calls_to_throws.len(), 15);
  }

  #[test]
  fn test_should_include_throws_for_no_ignore_statements_js() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/ignoreStatements.js", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: true,
      ignore_statements: vec![],
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    assert_eq!(result.functions_with_throws.len(), 6);
    assert_eq!(result.calls_to_throws.len(), 7);
  }

  #[test]
  fn test_should_not_include_throws_for_ignore_statements() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/ignoreStatements.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let ignore_statements = vec![
      "@it-throws".to_string(),
      "@it-throws-ignore".to_string(),
      "@some-random-ignore".to_string(),
    ];
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: true,
      ignore_statements,
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    assert_eq!(result.functions_with_throws.len(), 0);
    assert_eq!(result.calls_to_throws.len(), 0);
  }

  #[test]
  fn test_should_not_include_throws_for_ignore_statements_js() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/ignoreStatements.js", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let ignore_statements = vec![
      "@it-throws".to_string(),
      "@it-throws-ignore".to_string(),
      "@some-random-ignore".to_string(),
    ];
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: true,
      ignore_statements,
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    assert_eq!(result.functions_with_throws.len(), 0);
    assert_eq!(result.calls_to_throws.len(), 0);
  }

  #[test]
  fn test_should_handle_return_statements() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = format!("{}/src/fixtures/returnStatement.ts", manifest_dir);
    // Read sample code from file
    let sample_code = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let ignore_statements = vec![
      "@it-throws".to_string(),
      "@it-throws-ignore".to_string(),
      "@some-random-ignore".to_string(),
    ];
    let cm: Lrc<SourceMap> = Default::default();
    let user_settings = UserSettings {
      include_try_statement_throws: true,
      ignore_statements,
    };
    let (result, _cm) = analyze_code(&sample_code, cm, &user_settings);

    assert_eq!(result.functions_with_throws.len(), 2);
    assert_eq!(result.calls_to_throws.len(), 7);

    let function_names: Vec<String> = result
      .functions_with_throws
      .clone()
      .into_iter()
      .map(|f| f.function_or_method_name)
      .collect();

    fn function_names_contains(function_names: &Vec<String>, function_name: &str) -> bool {
      function_names.iter().any(|f| f == function_name)
    }

    ["someThrow", "badMethod"]
      .iter()
      .for_each(|f| assert!(function_names_contains(&function_names, f)));

    let calls_to_throws: Vec<String> = result.calls_to_throws.into_iter().map(|c| c.id).collect();

    fn calls_to_throws_contains(calls_to_throws: &Vec<String>, call_to_throw: &str) -> bool {
      calls_to_throws.iter().any(|c| c == call_to_throw)
    }

    [
      "NOT_SET-callToSomeThrow",
      "NOT_SET-callToSomeThrow",
      "NOT_SET-callToSomeThrow",
      "NOT_SET-test",
      "NOT_SET-callToSomeThrow",
      "NOT_SET-callToSomeThrow",
      "Test-callToSomeThrow",
    ]
    .iter()
    .for_each(|f| assert!(calls_to_throws_contains(&calls_to_throws, f)));
  }
}
