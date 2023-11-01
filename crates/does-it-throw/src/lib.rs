extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::vec;

use swc_ecma_ast::{
  AssignExpr, ClassDecl, ClassMethod, Decl, ExportDecl, FnDecl, MemberExpr, MethodProp, PatOrExpr,
  VarDeclarator,
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
  let mut collector = ThrowAnalyzer {
    functions_with_throws: HashSet::new(),
    json_parse_calls: vec![],
    fs_access_calls: vec![],
    import_sources: HashSet::new(),
    imported_identifiers: HashSet::new(),
    function_name_stack: vec![],
    current_class_name: None,
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

struct ThrowFinder {
  throw_spans: Vec<Span>,
}

impl Visit for ThrowFinder {
  fn visit_throw_stmt(&mut self, node: &ThrowStmt) {
    self.throw_spans.push(node.span)
  }
}

#[derive(Default)]

pub struct AnalysisResult {
  pub functions_with_throws: HashSet<ThrowMap>,
  pub calls_to_throws: HashSet<CallToThrowMap>,
  pub json_parse_calls: Vec<String>,
  pub fs_access_calls: Vec<String>,
  pub import_sources: HashSet<String>,
  pub imported_identifiers: HashSet<String>,
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
  imported_identifiers: HashSet<String>,
  function_name_stack: Vec<String>,
  current_class_name: Option<String>,
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

impl Visit for ThrowAnalyzer {
  fn visit_fn_decl(&mut self, fn_decl: &FnDecl) {
    let function_name = fn_decl.ident.sym.to_string();
    self.function_name_stack.push(function_name);

    swc_ecma_visit::visit_fn_decl(self, fn_decl);

    self.function_name_stack.pop();
  }

  fn visit_var_declarator(&mut self, declarator: &VarDeclarator) {
    if let Some(ident) = &declarator.name.as_ident() {
      if let Some(init) = &declarator.init {
        if matches!(&**init, Expr::Fn(_) | Expr::Arrow(_)) {
          let function_name = ident.sym.to_string();
          self.function_name_stack.push(function_name);
        }
      }
    }

    swc_ecma_visit::visit_var_declarator(self, declarator);

    if let Some(ident) = &declarator.name.as_ident() {
      if let Some(init) = &declarator.init {
        if matches!(&**init, Expr::Fn(_) | Expr::Arrow(_)) {
          self.function_name_stack.pop();
        }
      }
    }
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
                println!(
                  "object_property_stack: {:?}",
                  self.object_property_stack.last()
                );
                let call_to_throw_map = CallToThrowMap {
                  call_span: call.span,
                  throw_map: throw_map.clone(),
                  call_class_name: self.current_class_name.clone(),
                  call_function_or_method_name: call_function_or_method_name.clone(),
                  class_name: possible_class_name.clone(),
                  id: format!(
                    "{}-{}",
                    self
                      .current_class_name
                      .clone()
                      .unwrap_or_else(|| "NOT_SET".to_string()),
                    call_function_or_method_name.clone()
                  ),
                };
                self.calls.insert(call_to_throw_map);
                break;
              }
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
              // The function being called is known to throw
              let call_to_throw_map = CallToThrowMap {
                call_span: call.span,
                throw_map: throw_map.clone(),
                call_class_name: self.current_class_name.clone(),
                call_function_or_method_name: self
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
              self.calls.insert(call_to_throw_map);
              break;
            }
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
}
