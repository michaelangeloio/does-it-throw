extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;

use std::collections::HashSet;

use self::swc_common::{sync::Lrc, SourceMap, Span};
use self::swc_ecma_ast::{
  CallExpr, EsVersion, Expr, Function, ImportDecl, ImportSpecifier, MemberProp,
  ModuleExportName, ThrowStmt,
};
use self::swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use self::swc_ecma_visit::Visit;

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
    import_sources: HashSet::new(),
    imported_identifiers: HashSet::new(),
  };
  collector.visit_module(&module);

  (collector.into(), cm)
}

struct CodeAnalyzer {
  functions_with_throws: Vec<ThrowMap>,
  json_parse_calls: Vec<String>,
  fs_access_calls: Vec<String>,
  import_sources: HashSet<String>,
  imported_identifiers: HashSet<String>,
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

  fn register_import(&mut self, import: &ImportDecl) {
    self.import_sources.insert(import.src.value.to_string());
    for specifier in &import.specifiers {
      match specifier {
        ImportSpecifier::Default(default_spec) => {
          self
            .imported_identifiers
            .insert(default_spec.local.sym.to_string());
        }
        ImportSpecifier::Named(named_spec) => {
          let imported_name = match &named_spec.imported {
            Some(imported) => match imported {
              ModuleExportName::Ident(ident) => ident.sym.to_string(),
              ModuleExportName::Str(str) => str.value.to_string(),
            },
            None => named_spec.local.sym.to_string(),
          };
          self.imported_identifiers.insert(imported_name);
        }
        ImportSpecifier::Namespace(namespace_spec) => {
          self
            .imported_identifiers
            .insert(namespace_spec.local.sym.to_string());
        }
      }
    }
  }

}

impl Visit for CodeAnalyzer {

  fn visit_import_decl(&mut self, import: &ImportDecl) {
    self.register_import(import);
    self.import_sources.insert(import.src.value.to_string());
    swc_ecma_visit::visit_import_decl(self, import);
  }

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
  pub import_sources: HashSet<String>,
  pub imported_identifiers: HashSet<String>,
}

impl From<CodeAnalyzer> for AnalysisResult {
  fn from(analyzer: CodeAnalyzer) -> Self {
    Self {
      functions_with_throws: analyzer.functions_with_throws,
      json_parse_calls: analyzer.json_parse_calls,
      fs_access_calls: analyzer.fs_access_calls,
      import_sources: analyzer.import_sources,
      imported_identifiers: analyzer.imported_identifiers,
    }
  }
}
