extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::vec;

use swc_ecma_ast::{
  ArrowExpr, AssignExpr, BlockStmtOrExpr, ClassDecl, ClassMethod, Decl, ExportDecl, FnDecl,
  JSXAttr, JSXAttrOrSpread, JSXAttrValue, JSXExpr, JSXOpeningElement, MemberExpr, ObjectLit,
  PatOrExpr, Prop, PropName, PropOrSpread, Stmt, VarDeclarator,
};

use self::swc_common::{sync::Lrc, SourceMap, Span};
use self::swc_ecma_ast::{
  CallExpr, EsVersion, Expr, Function, ImportDecl, ImportSpecifier, MemberProp, ModuleExportName,
  ThrowStmt,
};
use self::swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use self::swc_ecma_visit::Visit;

pub fn analyze_code(content: &str, cm: Lrc<SourceMap>) -> (AnalysisResult, Lrc<SourceMap>) {
  let fm = cm.new_source_file(swc_common::FileName::Anon, content.into());
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
    None,
  );

  let mut parser = Parser::new_from(lexer);
  let module = parser.parse_module().expect("Failed to parse module");
  let mut collector = ThrowAnalyzer {
    functions_with_throws: HashSet::new(),
    json_parse_calls: vec![],
    fs_access_calls: vec![],
    import_sources: HashSet::new(),
    imported_identifiers: Vec::new(),
    function_name_stack: vec![],
    current_class_name: None,
    current_method_name: None,
    imported_identifier_usages: vec![],
  };
  collector.visit_module(&module);
  let mut call_collector = CallFinder {
    functions_with_throws: collector.functions_with_throws.clone(),
    calls: HashSet::new(),
    current_class_name: None,
    instantiations: HashSet::new(),
    function_name_stack: vec![],
    object_property_stack: vec![],
  };
  call_collector.visit_module(&module);
  let combined_analyzers = CombinedAnalyzers {
    throw_analyzer: collector,
    call_finder: call_collector,
  };

  (combined_analyzers.into(), cm)
}

fn prop_name_to_string(prop_name: &PropName) -> String {
  match prop_name {
    PropName::Ident(ident) => ident.sym.to_string(),
    PropName::Str(str_) => str_.value.to_string(),
    PropName::Num(num) => num.value.to_string(),
    _ => "anonymous".to_string(), // Fallback for unnamed functions
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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IdentifierUsage {
  pub usage_span: Span,
  pub identifier_name: String,
  pub usage_context: String,
  pub id: String,
}

impl IdentifierUsage {
  pub fn new(usage_span: Span, identifier_name: String, usage_context: String, id: String) -> Self {
    Self {
      usage_span,
      identifier_name,
      usage_context,
      id,
    }
  }
}

#[derive(Default)]
pub struct AnalysisResult {
  pub functions_with_throws: HashSet<ThrowMap>,
  pub calls_to_throws: HashSet<CallToThrowMap>,
  pub json_parse_calls: Vec<String>,
  pub fs_access_calls: Vec<String>,
  pub import_sources: HashSet<String>,
  pub imported_identifiers: Vec<String>,
  pub imported_identifier_usages: Vec<IdentifierUsage>,
}

struct CombinedAnalyzers {
  throw_analyzer: ThrowAnalyzer,
  call_finder: CallFinder,
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
      imported_identifier_usages: analyzers.throw_analyzer.imported_identifier_usages,
    }
  }
}

#[derive(Clone)]
pub struct ThrowMap {
  pub throw_spans: Vec<Span>,
  pub throw_statement: Span,
  pub function_or_method_name: String,
  pub class_name: Option<String>,
  pub id: String,
}

impl PartialEq for ThrowMap {
  fn eq(&self, other: &Self) -> bool {
    self.throw_statement == other.throw_statement
  }
}

impl Eq for ThrowMap {}

impl Hash for ThrowMap {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.throw_statement.lo.hash(state);
    self.throw_statement.hi.hash(state);
    self.throw_statement.ctxt.hash(state);
  }
}

struct ThrowAnalyzer {
  functions_with_throws: HashSet<ThrowMap>,
  json_parse_calls: Vec<String>,
  fs_access_calls: Vec<String>,
  import_sources: HashSet<String>,
  imported_identifiers: Vec<String>,
  function_name_stack: Vec<String>,
  current_class_name: Option<String>,
  current_method_name: Option<String>,
  imported_identifier_usages: Vec<IdentifierUsage>,
}

impl ThrowAnalyzer {
  fn check_function_for_throws(&mut self, function: &Function) {
    let mut throw_finder = ThrowFinder {
      throw_spans: vec![],
    };
    throw_finder.visit_function(function);
    if !throw_finder.throw_spans.is_empty() {
      let throw_map = ThrowMap {
        throw_spans: throw_finder.throw_spans,
        throw_statement: function.span,
        function_or_method_name: self
          .function_name_stack
          .last()
          .cloned()
          .unwrap_or_else(|| "<anonymous>".to_string()),
        class_name: None,
        id: format!(
          "{}-{}",
          self
            .current_class_name
            .clone()
            .unwrap_or_else(|| "NOT_SET".to_string()),
          self
            .function_name_stack
            .last()
            .cloned()
            .unwrap_or_else(|| "<anonymous>".to_string())
        ),
      };
      self.functions_with_throws.insert(throw_map);
    }
  }

  fn check_arrow_function_for_throws(&mut self, arrow_function: &swc_ecma_ast::ArrowExpr) {
    let mut throw_finder = ThrowFinder {
      throw_spans: vec![],
    };
    throw_finder.visit_arrow_expr(arrow_function);
    if !throw_finder.throw_spans.is_empty() {
      let throw_map = ThrowMap {
        throw_spans: throw_finder.throw_spans,
        throw_statement: arrow_function.span,
        function_or_method_name: self
          .function_name_stack
          .last()
          .cloned()
          .unwrap_or_else(|| "<anonymous>".to_string()),
        class_name: None,
        id: format!(
          "{}-{}",
          self
            .current_class_name
            .clone()
            .unwrap_or_else(|| "NOT_SET".to_string()),
          self
            .function_name_stack
            .last()
            .cloned()
            .unwrap_or_else(|| "<anonymous>".to_string())
        ),
      };
      self.functions_with_throws.insert(throw_map);
    }
  }

  fn register_import(&mut self, import: &ImportDecl) {
    self.import_sources.insert(import.src.value.to_string());
    for specifier in &import.specifiers {
      match specifier {
        ImportSpecifier::Default(default_spec) => {
          self
            .imported_identifiers
            .push(default_spec.local.sym.to_string());
        }
        ImportSpecifier::Named(named_spec) => {
          let imported_name = match &named_spec.imported {
            Some(imported) => match imported {
              ModuleExportName::Ident(ident) => ident.sym.to_string(),
              ModuleExportName::Str(str) => str.value.to_string(),
            },
            None => named_spec.local.sym.to_string(),
          };
          self.imported_identifiers.push(imported_name);
        }
        ImportSpecifier::Namespace(namespace_spec) => {
          self
            .imported_identifiers
            .push(namespace_spec.local.sym.to_string());
        }
      }
    }
  }
}

// --------- ThrowAnalyzer Visitor implementation ---------
// `ThrowAnalyzer` uses the Visitor pattern to traverse the AST of JavaScript or TypeScript code.
// Its primary goal is to identify functions that throw exceptions and record their context.
// It also records the usage of imported identifiers to help identify the context of function calls.

impl Visit for ThrowAnalyzer {
  fn visit_call_expr(&mut self, call: &CallExpr) {
    if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
      match &**expr {
        Expr::Member(member_expr) => {
          if let Expr::Ident(object_ident) = &*member_expr.obj {
            self.current_class_name = Some(object_ident.sym.to_string());
          }

          if let MemberProp::Ident(method_ident) = &member_expr.prop {
            self.current_method_name = Some(method_ident.sym.to_string());
          }

          if let (Some(current_class_name), Some(current_method_name)) = (
            self.current_class_name.clone(),
            self.current_method_name.clone(),
          ) {
            if self.imported_identifiers.contains(&current_class_name) {
              let usage_context = current_method_name.clone();
              let id = format!(
                "{}-{}",
                self
                  .current_class_name
                  .clone()
                  .unwrap_or_else(|| "NOT_SET".to_string()),
                usage_context
              );
              // Create and store the identifier usage information
              let usage_map = IdentifierUsage::new(
                call.span,
                current_class_name.clone(),
                usage_context.clone(),
                id.clone(),
              );
              self.imported_identifier_usages.push(usage_map);
            }
          }
          for arg in &call.args {
            self.function_name_stack.push(
              self
                .current_method_name
                .clone()
                .unwrap_or_else(|| "<anonymous>".to_string()),
            );
            if let Expr::Arrow(arrow_expr) = &*arg.expr {
              self.check_arrow_function_for_throws(arrow_expr);
							self.visit_arrow_expr(&arrow_expr)
            }
            if let Expr::Fn(fn_expr) = &*arg.expr {
              self.check_function_for_throws(&fn_expr.function);
            }
            self.function_name_stack.pop();
          }
          self.current_class_name = None;
          self.current_method_name = None;
        }

        Expr::Ident(ident) => {
          let called_function_name = ident.sym.to_string();
          if self.imported_identifiers.contains(&called_function_name) {
            let usage_context = self
              .function_name_stack
              .last()
              .cloned()
              .unwrap_or_else(|| "<anonymous>".to_string());
            let id = format!(
              "{}-{}",
              self
                .current_class_name
                .clone()
                .unwrap_or_else(|| "NOT_SET".to_string()),
              called_function_name
            );
            let usage_map = IdentifierUsage::new(
              call.span,
              called_function_name.clone(),
              usage_context.clone(),
              id.clone(),
            );
            self.imported_identifier_usages.push(usage_map);
          }
          for arg in &call.args {
            self.function_name_stack.push(called_function_name.clone());
            if let Expr::Arrow(arrow_expr) = &*arg.expr {
              self.check_arrow_function_for_throws(arrow_expr);
            }
            if let Expr::Fn(fn_expr) = &*arg.expr {
              self.check_function_for_throws(&fn_expr.function);
            }
            self.function_name_stack.pop();
          }
        }

        Expr::Arrow(arrow_expr) => {
          let mut throw_finder = ThrowFinder {
            throw_spans: vec![],
          };
          throw_finder.visit_arrow_expr(arrow_expr);
          if !throw_finder.throw_spans.is_empty() {
            let throw_map = ThrowMap {
              throw_spans: throw_finder.throw_spans,
              throw_statement: arrow_expr.span,
              function_or_method_name: self
                .function_name_stack
                .last()
                .cloned()
                .unwrap_or_else(|| "<anonymous>".to_string()),
              class_name: None,
              id: format!(
                "{}-{}",
                self
                  .current_class_name
                  .clone()
                  .unwrap_or_else(|| "NOT_SET".to_string()),
                self
                  .function_name_stack
                  .last()
                  .cloned()
                  .unwrap_or_else(|| "<anonymous>".to_string())
              ),
            };
            self.functions_with_throws.insert(throw_map);
          }
        }
        _ => {}
      }
    }
  }

  fn visit_fn_decl(&mut self, fn_decl: &FnDecl) {
    let function_name = fn_decl.ident.sym.to_string();
    self.function_name_stack.push(function_name);

    swc_ecma_visit::visit_fn_decl(self, fn_decl);

    self.function_name_stack.pop();
  }

  fn visit_object_lit(&mut self, object_lit: &ObjectLit) {
    // Iterate over the properties of the object literal
    for prop in &object_lit.props {
      match prop {
        // Check for method properties (e.g., someImportedThrow: () => { ... })
        PropOrSpread::Prop(prop) => {
          if let Prop::Method(method_prop) = &**prop {
            if let Some(method_name) = &method_prop.key.as_ident() {
              let method_name: String = method_name.sym.to_string();

              self.function_name_stack.push(method_name.clone());

              let mut throw_finder = ThrowFinder {
                throw_spans: vec![],
              };
              throw_finder.visit_function(&method_prop.function);

              if !throw_finder.throw_spans.is_empty() {
                let throw_map = ThrowMap {
                  throw_spans: throw_finder.throw_spans,
                  throw_statement: method_prop.function.span,
                  function_or_method_name: method_name.clone(),
                  class_name: self.current_class_name.clone(),
                  id: format!(
                    "{}-{}",
                    self
                      .current_class_name
                      .clone()
                      .unwrap_or_else(|| "NOT_SET".to_string()),
                    method_name
                  ),
                };
                self.functions_with_throws.insert(throw_map);
              }

              self.function_name_stack.pop();
            }
          }
          if let Prop::KeyValue(key_value_prop) = &**prop {
            match &*key_value_prop.value {
              Expr::Fn(fn_expr) => {
                let mut throw_finder = ThrowFinder {
                  throw_spans: vec![],
                };
                throw_finder.visit_function(&fn_expr.function);
                let function_name = prop_name_to_string(&key_value_prop.key);

                if !throw_finder.throw_spans.is_empty() {
                  let throw_map = ThrowMap {
                    throw_spans: throw_finder.throw_spans,
                    throw_statement: fn_expr.function.span,
                    function_or_method_name: function_name.clone(),
                    class_name: self.current_class_name.clone(),
                    id: format!(
                      "{}-{}",
                      self
                        .current_class_name
                        .clone()
                        .unwrap_or_else(|| "NOT_SET".to_string()),
                      function_name
                    ),
                  };
                  self.functions_with_throws.insert(throw_map);
                }
              }
              Expr::Arrow(arrow_expr) => {
                let mut throw_finder = ThrowFinder {
                  throw_spans: vec![],
                };
                throw_finder.visit_arrow_expr(arrow_expr);
                let function_name = prop_name_to_string(&key_value_prop.key);

                if !throw_finder.throw_spans.is_empty() {
                  let throw_map = ThrowMap {
                    throw_spans: throw_finder.throw_spans,
                    throw_statement: arrow_expr.span,
                    function_or_method_name: function_name.clone(),
                    class_name: self.current_class_name.clone(),
                    id: format!(
                      "{}-{}",
                      self
                        .current_class_name
                        .clone()
                        .unwrap_or_else(|| "NOT_SET".to_string()),
                      function_name
                    ),
                  };
                  self.functions_with_throws.insert(throw_map);
                }
              }
              _ => {}
            }
          }
        }
        _ => {}
      }
    }
    swc_ecma_visit::visit_object_lit(self, object_lit);
  }

  fn visit_var_declarator(&mut self, declarator: &VarDeclarator) {
    if let Some(ident) = &declarator.name.as_ident() {
      if let Some(init) = &declarator.init {
        let function_name = ident.sym.to_string();
        let mut throw_finder = ThrowFinder {
          throw_spans: vec![],
        };

        // Check if the init is a function expression or arrow function
        if let Expr::Fn(fn_expr) = &**init {
          self.function_name_stack.push(function_name.clone());
          throw_finder.visit_function(&fn_expr.function);
          self.function_name_stack.pop();
        } else if let Expr::Arrow(arrow_expr) = &**init {
          self.function_name_stack.push(function_name.clone());
          throw_finder.visit_arrow_expr(arrow_expr);
          self.function_name_stack.pop();
        }

        if !throw_finder.throw_spans.is_empty() {
          let throw_map = ThrowMap {
            throw_spans: throw_finder.throw_spans,
            throw_statement: declarator.span,
            function_or_method_name: function_name.clone(),
            class_name: self.current_class_name.clone(),
            id: format!(
              "{}-{}",
              self
                .current_class_name
                .clone()
                .unwrap_or_else(|| "NOT_SET".to_string()),
              function_name
            ),
          };
          self.functions_with_throws.insert(throw_map);
        }
      }
    }
    swc_ecma_visit::visit_var_declarator(self, declarator);
  }
  fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) {
    if let PatOrExpr::Expr(expr) = &assign_expr.left {
      if let Expr::Ident(ident) = &**expr {
        if matches!(&*assign_expr.right, Expr::Fn(_) | Expr::Arrow(_)) {
          let function_name = ident.sym.to_string();
          self.function_name_stack.push(function_name);
        }
      }
    }

    swc_ecma_visit::visit_assign_expr(self, assign_expr);

    if let PatOrExpr::Expr(expr) = &assign_expr.left {
      if let Expr::Ident(_) = &**expr {
        if matches!(&*assign_expr.right, Expr::Fn(_) | Expr::Arrow(_)) {
          self.function_name_stack.pop();
        }
      }
    }
  }

  fn visit_import_decl(&mut self, import: &ImportDecl) {
    self.register_import(import);
    self.import_sources.insert(import.src.value.to_string());
    swc_ecma_visit::visit_import_decl(self, import);
  }

  fn visit_function(&mut self, function: &Function) {
    self.check_function_for_throws(function);
    swc_ecma_visit::visit_function(self, function);
  }

  fn visit_arrow_expr(&mut self, arrow_expr: &swc_ecma_ast::ArrowExpr) {
			match &*arrow_expr.body {
				BlockStmtOrExpr::BlockStmt(block_stmt) => {
					for stmt in &block_stmt.stmts {
						self.visit_stmt(stmt);
					}
				}
				BlockStmtOrExpr::Expr(expr) => {
					if let Expr::Call(call_expr) = &**expr {
						self.visit_call_expr(call_expr);
					} else {
						// use default implementation for other kinds of expressions (for now)
						self.visit_expr(expr);
					}
				}
			}
    swc_ecma_visit::visit_arrow_expr(self, arrow_expr);
  }

	fn visit_stmt(&mut self, stmt: &Stmt) {
    match stmt {
      Stmt::Expr(expr_stmt) => {
        self.visit_expr(&expr_stmt.expr);
      }
      Stmt::Block(block_stmt) => {
        for stmt in &block_stmt.stmts {
          self.visit_stmt(stmt);
        }
      }
      Stmt::If(if_stmt) => {
        self.visit_expr(&if_stmt.test);
        self.visit_stmt(&*if_stmt.cons);
        if let Some(alt) = &if_stmt.alt {
          self.visit_stmt(alt);
        }
      }
      _ => {
        // For other kinds of statements, we continue with the default implementation (for now)
        swc_ecma_visit::visit_stmt(self, stmt);
      }
    }
  }

  fn visit_expr(&mut self, expr: &Expr) {
    if let Expr::Call(call_expr) = &*expr {
      self.visit_call_expr(call_expr)
    }
		swc_ecma_visit::visit_expr(self, expr);
	}
  

  fn visit_class_method(&mut self, class_method: &ClassMethod) {
    if let Some(method_name) = &class_method.key.as_ident() {
      let method_name = method_name.sym.to_string();

      self.function_name_stack.push(method_name.clone());

      let mut throw_finder = ThrowFinder {
        throw_spans: vec![],
      };
      throw_finder.visit_class_method(class_method);

      if !throw_finder.throw_spans.is_empty() {
        let throw_map = ThrowMap {
          throw_spans: throw_finder.throw_spans,
          throw_statement: class_method.span,
          function_or_method_name: method_name.clone(),
          class_name: self.current_class_name.clone(),
          id: format!(
            "{}-{}",
            self
              .current_class_name
              .clone()
              .unwrap_or_else(|| "NOT_SET".to_string()),
            method_name
          ),
        };
        self.functions_with_throws.insert(throw_map);
      }

      self.function_name_stack.pop();
    }

    self.function_name_stack.pop();

    swc_ecma_visit::visit_class_method(self, class_method);
  }

  fn visit_class_decl(&mut self, class_decl: &ClassDecl) {
    self.current_class_name = Some(class_decl.ident.sym.to_string());
    self.visit_class(&class_decl.class);
    self.current_class_name = None;
  }

  fn visit_export_decl(&mut self, export_decl: &ExportDecl) {
    if let Decl::Class(class_decl) = &export_decl.decl {
      self.current_class_name = Some(class_decl.ident.sym.to_string());
      self.visit_class(&class_decl.class);
      self.current_class_name = None;
    } else {
      swc_ecma_visit::visit_export_decl(self, export_decl);
    }
  }
}

pub struct CallToThrowMap {
  pub call_span: Span,
  pub call_function_or_method_name: String,
  pub call_class_name: Option<String>,
  pub throw_map: ThrowMap,
  pub class_name: Option<String>,
  pub id: String,
}

impl PartialEq for CallToThrowMap {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for CallToThrowMap {}

impl Hash for CallToThrowMap {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.id.hash(state);
    self.call_span.lo.hash(state);
    self.call_span.hi.hash(state);
  }
}

struct InstantiationsMap {
  pub class_name: String,
  pub variable_name: String,
}

impl PartialEq for InstantiationsMap {
  fn eq(&self, other: &Self) -> bool {
    self.variable_name == other.variable_name
  }
}

impl Eq for InstantiationsMap {}

impl Hash for InstantiationsMap {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.variable_name.hash(state);
  }
}

struct CallFinder {
  calls: HashSet<CallToThrowMap>,
  functions_with_throws: HashSet<ThrowMap>,
  current_class_name: Option<String>,
  instantiations: HashSet<InstantiationsMap>,
  function_name_stack: Vec<String>,
  object_property_stack: Vec<String>,
}

// ----- CallFinder Visitor implementation -----
/// This module defines structures and implements functionality for identifying and mapping
/// function calls to their respective functions or methods that throw exceptions. It uses
/// SWC's visitor pattern to traverse the AST (Abstract Syntax Tree) of JavaScript or TypeScript code.
///
/// `CallToThrowMap` records the mapping of a function call to a function that throws.
/// It captures the span of the call, the name of the function/method being called,
/// the class name if the call is a method call, and the `ThrowMap` that provides details
/// about the throw statement in the called function/method.
///
/// `InstantiationsMap` keeps track of class instantiations by recording the class name
/// and the variable name that holds the instance.
///
/// `CallFinder` is the core structure that uses the Visitor pattern to traverse the AST nodes.
/// It maintains state as it goes through the code, keeping track of current class names,
/// function name stacks, and object property stacks. As it finds function calls, it tries
/// to match them with known functions or methods that throw exceptions using the data
/// accumulated in `functions_with_throws`. When a match is found, it records the mapping
/// in `calls`. It also tracks instantiations of classes to help resolve method calls.

impl Visit for CallFinder {
  fn visit_class_decl(&mut self, class_decl: &ClassDecl) {
    self.current_class_name = Some(class_decl.ident.sym.to_string());
    self.visit_class(&class_decl.class);
    self.current_class_name = None;
  }

  fn visit_fn_decl(&mut self, fn_decl: &FnDecl) {
    let function_name = fn_decl.ident.sym.to_string();
    self.function_name_stack.push(function_name);

    swc_ecma_visit::visit_fn_decl(self, fn_decl);

    self.function_name_stack.pop();
  }

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    if let MemberProp::Ident(ident) = &member_expr.prop {
      self.object_property_stack.push(ident.sym.to_string());
    }
    swc_ecma_visit::visit_member_expr(self, member_expr);
    self.object_property_stack.pop();
  }

  fn visit_class_method(&mut self, method: &ClassMethod) {
    if let Some(method_ident) = method.key.as_ident() {
      self
        .object_property_stack
        .push(method_ident.sym.to_string());
    }

    swc_ecma_visit::visit_class_method(self, method);

    self.object_property_stack.pop();
  }

  fn visit_jsx_opening_element(&mut self, jsx_opening_element: &JSXOpeningElement) {
    for attr in &jsx_opening_element.attrs {
      if let JSXAttrOrSpread::JSXAttr(attr) = attr {
        self.visit_jsx_attr(attr);
      }
    }
  }

  fn visit_jsx_attr(&mut self, jsx_attr: &JSXAttr) {
    if let Some(JSXAttrValue::JSXExprContainer(expr_container)) = &jsx_attr.value {
      if let JSXExpr::Expr(expr) = &expr_container.expr {
        // Check if the expression is a function call
        if let Expr::Call(call_expr) = &**expr {
          self.visit_call_expr(call_expr)
        }
      }
    }
  }

  fn visit_call_expr(&mut self, call: &CallExpr) {
    if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
      match &**expr {
        Expr::Member(member_expr) => {
          let mut possible_class_name = None;
          if let Expr::Ident(object_ident) = &*member_expr.obj {
            possible_class_name = Some(object_ident.sym.to_string());
          } else if let Expr::This(_) = &*member_expr.obj {
            possible_class_name = self.current_class_name.clone();
          }
          if let Some(ref obj_name) = possible_class_name {
            let mut new_class_name = None;
            for instantiation in self.instantiations.iter() {
              if &instantiation.variable_name == obj_name {
                new_class_name = Some(instantiation.class_name.clone());
              }
            }
            if let Some(class_name) = new_class_name {
              possible_class_name = Some(class_name);
            }
          }

          if let MemberProp::Ident(method_ident) = &member_expr.prop {
            let called_method_name = method_ident.sym.to_string();
            for throw_map in self.functions_with_throws.iter() {
              let call_function_or_method_name =
                if let Some(function_name) = self.function_name_stack.last() {
                  function_name.clone()
                } else if let Some(property_name) = self.object_property_stack.last() {
                  property_name.clone()
                } else {
                  "<anonymous>".to_string()
                };
              if throw_map.function_or_method_name == called_method_name {
                let class_name_or_not_set = self
                  .current_class_name
                  .clone()
                  .or(possible_class_name.clone())
                  .unwrap_or_else(|| "NOT_SET".to_string());
                let call_to_throw_map = CallToThrowMap {
                  call_span: call.span,
                  throw_map: throw_map.clone(),
                  call_class_name: Some(class_name_or_not_set.clone()),
                  call_function_or_method_name: call_function_or_method_name.clone(),
                  class_name: possible_class_name.clone(),
                  id: format!(
                    "{}-{}",
                    class_name_or_not_set,
                    call_function_or_method_name.clone()
                  ),
                };
                self.calls.insert(call_to_throw_map);
                break;
              }
            }
            for arg in &call.args {
              self.function_name_stack.push(method_ident.sym.to_string());
              self.current_class_name = possible_class_name.clone();
              if let Expr::Arrow(arrow_expr) = &*arg.expr {
                self.visit_arrow_expr(arrow_expr);
              }
              if let Expr::Fn(fn_expr) = &*arg.expr {
                self.visit_function(&fn_expr.function);
              }
              self.function_name_stack.pop();
              self.current_class_name = None;
            }
          }
        }
        Expr::Ident(ident) => {
          let called_function_name = ident.sym.to_string();
          for throw_map in self.functions_with_throws.iter() {
            let potential_throw_id = format!(
              "{}-{}",
              self
                .current_class_name
                .clone()
                .unwrap_or_else(|| "NOT_SET".to_string()),
              called_function_name
            );
            if throw_map.id == potential_throw_id {
              let call_function_or_method_name = self
                .function_name_stack
                .last()
                .cloned()
                .unwrap_or_else(|| "<anonymous>".to_string());
              // The function being called is known to throw
              let call_to_throw_map = CallToThrowMap {
                call_span: call.span,
                throw_map: throw_map.clone(),
                call_class_name: self.current_class_name.clone(),
                call_function_or_method_name: call_function_or_method_name.clone(),
                class_name: None,
                id: format!(
                  "{}-{}",
                  self
                    .current_class_name
                    .clone()
                    .unwrap_or_else(|| "NOT_SET".to_string()),
                  call_function_or_method_name
                ),
              };
              self.calls.insert(call_to_throw_map);
              break;
            }
          }
          for arg in &call.args {
            self.function_name_stack.push(called_function_name.clone());
            if let Expr::Arrow(arrow_expr) = &*arg.expr {
              self.visit_arrow_expr(arrow_expr);
            }
            if let Expr::Fn(fn_expr) = &*arg.expr {
              self.visit_function(&fn_expr.function);
            }
            self.function_name_stack.pop();
          }
        }
        _ => {}
      }
    }
  }

  fn visit_var_declarator(&mut self, var_declarator: &VarDeclarator) {
    if let Some(init_expr) = &var_declarator.init {
      match &**init_expr {
        Expr::New(new_expr) => {
          if let Expr::Ident(expr) = &*new_expr.callee {
            let class_name = expr.sym.to_string();
            if let Some(var_ident) = &var_declarator.name.as_ident() {
              let var_name = var_ident.sym.to_string();
              self.instantiations.insert(InstantiationsMap {
                class_name: class_name,
                variable_name: var_name,
              });
            }
          }
        }
        _ => {}
      }
    }
    if let Some(ident) = &var_declarator.name.as_ident() {
      if let Some(init) = &var_declarator.init {
        if matches!(&**init, Expr::Fn(_) | Expr::Arrow(_)) {
          let function_name = ident.sym.to_string();
          self.function_name_stack.push(function_name);
        }
      }
    }

    // swc_ecma_visit::visit_var_declarator(self, declarator);

    if let Some(ident) = &var_declarator.name.as_ident() {
      if let Some(init) = &var_declarator.init {
        if matches!(&**init, Expr::Fn(_) | Expr::Arrow(_)) {
          self.function_name_stack.pop();
        }
      }
    }
    swc_ecma_visit::visit_var_declarator(self, var_declarator);
  }

  fn visit_arrow_expr(&mut self, arrow_expr: &ArrowExpr) {
    match &*arrow_expr.body {
      BlockStmtOrExpr::BlockStmt(block_stmt) => {
        for stmt in &block_stmt.stmts {
          self.visit_stmt(stmt);
        }
      }
      BlockStmtOrExpr::Expr(expr) => {
        if let Expr::Call(call_expr) = &**expr {
          self.visit_call_expr(call_expr);
        } else {
          // use default implementation for other kinds of expressions (for now)
          self.visit_expr(expr);
        }
      }
    }
  }

  fn visit_function(&mut self, function: &Function) {
    if let Some(block_stmt) = &function.body {
      for stmt in &block_stmt.stmts {
        self.visit_stmt(stmt);
      }
    }
  }

  fn visit_stmt(&mut self, stmt: &Stmt) {
    match stmt {
      Stmt::Expr(expr_stmt) => {
        self.visit_expr(&expr_stmt.expr);
      }
      Stmt::Block(block_stmt) => {
        for stmt in &block_stmt.stmts {
          self.visit_stmt(stmt);
        }
      }
      Stmt::If(if_stmt) => {
        self.visit_expr(&if_stmt.test);
        self.visit_stmt(&*if_stmt.cons);
        if let Some(alt) = &if_stmt.alt {
          self.visit_stmt(alt);
        }
      }
      _ => {
        // For other kinds of statements, we continue with the default implementation (for now)
        swc_ecma_visit::visit_stmt(self, stmt);
      }
    }
  }

  fn visit_expr(&mut self, expr: &Expr) {
    if let Expr::Call(call_expr) = &*expr {
      self.visit_call_expr(call_expr)
    }
  }
}
