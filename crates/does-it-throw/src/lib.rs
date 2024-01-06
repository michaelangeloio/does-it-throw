pub mod call_finder;
pub mod import_usage_finder;
pub mod throw_finder;
use call_finder::{CallFinder, CallToThrowMap};
use import_usage_finder::ImportUsageFinder;
use swc_common::comments::SingleThreadedComments;
use throw_finder::{IdentifierUsage, ThrowAnalyzer, ThrowMap};
extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;

use std::collections::HashSet;

use std::vec;

use self::swc_common::{sync::Lrc, SourceMap};
use self::swc_ecma_ast::EsVersion;
use self::swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use self::swc_ecma_visit::Visit;

#[derive(Default)]
pub struct AnalysisResult {
  pub functions_with_throws: HashSet<ThrowMap>,
  pub calls_to_throws: HashSet<CallToThrowMap>,
  pub json_parse_calls: Vec<String>,
  pub fs_access_calls: Vec<String>,
  pub import_sources: HashSet<String>,
  pub imported_identifiers: Vec<String>,
  pub imported_identifier_usages: HashSet<IdentifierUsage>,
}

struct CombinedAnalyzers {
  throw_analyzer: ThrowAnalyzer,
  call_finder: CallFinder,
  import_usage_finder: ImportUsageFinder,
}

impl From<CombinedAnalyzers> for AnalysisResult {
  fn from(analyzers: CombinedAnalyzers) -> Self {
    Self {
      functions_with_throws: analyzers.throw_analyzer.functions_with_throws,
      calls_to_throws: analyzers.call_finder.calls,
      json_parse_calls: analyzers.throw_analyzer.json_parse_calls,
      fs_access_calls: analyzers.throw_analyzer.fs_access_calls,
      import_sources: analyzers.throw_analyzer.import_sources,
      imported_identifiers: analyzers.throw_analyzer.imported_identifiers,
      imported_identifier_usages: analyzers.import_usage_finder.imported_identifier_usages,
    }
  }
}

pub struct UserSettings {
  pub include_try_statement_throws: bool,
}

pub fn analyze_code(
  content: &str,
  cm: Lrc<SourceMap>,
  user_settings: &UserSettings,
) -> (AnalysisResult, Lrc<SourceMap>) {
  let fm = cm.new_source_file(swc_common::FileName::Anon, content.into());
  let comments = Lrc::new(SingleThreadedComments::default());
  let lexer = Lexer::new(
    Syntax::Typescript(swc_ecma_parser::TsConfig {
      tsx: true,
      decorators: true,
      dts: false,
      no_early_errors: false,
      disallow_ambiguous_jsx_like: false,
    }),
    EsVersion::latest(),
    StringInput::from(&*fm),
    Some(&comments),
  );

  let mut parser = Parser::new_from(lexer);
  let module = parser.parse_module().expect("Failed to parse module");
  let mut throw_collector = ThrowAnalyzer {
    comments: comments.clone(),
    functions_with_throws: HashSet::new(),
    json_parse_calls: vec![],
    fs_access_calls: vec![],
    import_sources: HashSet::new(),
    imported_identifiers: Vec::new(),
    function_name_stack: vec![],
    current_class_name: None,
    current_method_name: None,
    include_try_statement: user_settings.include_try_statement_throws,
  };
  throw_collector.visit_module(&module);
  let mut call_collector = CallFinder {
    functions_with_throws: throw_collector.functions_with_throws.clone(),
    calls: HashSet::new(),
    current_class_name: None,
    instantiations: HashSet::new(),
    function_name_stack: vec![],
    object_property_stack: vec![],
  };
  call_collector.visit_module(&module);

  let mut import_usages_collector = ImportUsageFinder {
    imported_identifiers: throw_collector.imported_identifiers.clone(),
    imported_identifier_usages: HashSet::new(),
    current_class_name: None,
    current_method_name: None,
    function_name_stack: vec![],
  };
  import_usages_collector.visit_module(&module);

  let combined_analyzers = CombinedAnalyzers {
    throw_analyzer: throw_collector,
    call_finder: call_collector,
    import_usage_finder: import_usages_collector,
  };

  (combined_analyzers.into(), cm)
}
