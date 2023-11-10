extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::vec;

use swc_ecma_ast::{
  ArrowExpr, AssignExpr, BlockStmtOrExpr, Callee, ClassDecl, ClassMethod, Constructor, Decl,
  ExportDecl, FnDecl, ObjectLit, PatOrExpr, Prop, PropName, PropOrSpread, Stmt, VarDeclarator,
};

use self::swc_common::Span;
use self::swc_ecma_ast::{
  CallExpr, Expr, Function, ImportDecl, ImportSpecifier, MemberProp, ModuleExportName, ThrowStmt,
};

use self::swc_ecma_visit::Visit;

fn prop_name_to_string(prop_name: &PropName) -> String {
  match prop_name {
    PropName::Ident(ident) => ident.sym.to_string(),
    PropName::Str(str_) => str_.value.to_string(),
    PropName::Num(num) => num.value.to_string(),
    _ => "anonymous".to_string(), // Fallback for unnamed functions
  }
}

pub struct ThrowFinder {
  throw_spans: Vec<Span>,
}

impl Visit for ThrowFinder {
  fn visit_throw_stmt(&mut self, node: &ThrowStmt) {
    self.throw_spans.push(node.span)
  }
}

#[derive(Clone)]
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

impl Eq for IdentifierUsage {}

impl PartialEq for IdentifierUsage {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Hash for IdentifierUsage {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.id.hash(state);
    self.usage_span.lo.hash(state);
    self.usage_span.hi.hash(state);
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

pub struct ThrowAnalyzer {
  pub functions_with_throws: HashSet<ThrowMap>,
  pub json_parse_calls: Vec<String>,
  pub fs_access_calls: Vec<String>,
  pub import_sources: HashSet<String>,
  pub imported_identifiers: Vec<String>,
  pub function_name_stack: Vec<String>,
  pub current_class_name: Option<String>,
  pub current_method_name: Option<String>,
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

  fn check_arrow_function_for_throws(&mut self, arrow_function: &ArrowExpr) {
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

  fn check_constructor_for_throws(&mut self, constructor: &Constructor) {
    let mut throw_finder = ThrowFinder {
      throw_spans: vec![],
    };
    throw_finder.visit_constructor(constructor);
    if !throw_finder.throw_spans.is_empty() {
      let throw_map = ThrowMap {
        throw_spans: throw_finder.throw_spans,
        throw_statement: constructor.span,
        function_or_method_name: self
          .current_method_name
          .clone()
          .unwrap_or_else(|| "<constructor>".to_string()),
        class_name: self.current_class_name.clone(),
        id: format!(
          "{}-{}",
          self
            .current_class_name
            .clone()
            .unwrap_or_else(|| "NOT_SET".to_string()),
          self
            .current_method_name
            .clone()
            .unwrap_or_else(|| "<constructor>".to_string())
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
    if let Callee::Expr(expr) = &call.callee {
      match &**expr {
        Expr::Member(member_expr) => {
          // if let Expr::Ident(object_ident) = &*member_expr.obj {
          //   self.current_class_name = Some(object_ident.sym.to_string());
          // }

          if let MemberProp::Ident(method_ident) = &member_expr.prop {
            self.current_method_name = Some(method_ident.sym.to_string());
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
              self.visit_arrow_expr(arrow_expr)
            }
            if let Expr::Fn(fn_expr) = &*arg.expr {
              self.check_function_for_throws(&fn_expr.function);
              self.visit_function(&fn_expr.function)
            }
            self.function_name_stack.pop();
          }
        }

        Expr::Ident(ident) => {
          let called_function_name = ident.sym.to_string();
          for arg in &call.args {
            self.function_name_stack.push(called_function_name.clone());
            if let Expr::Arrow(arrow_expr) = &*arg.expr {
              self.check_arrow_function_for_throws(arrow_expr);
              self.visit_arrow_expr(arrow_expr);
            }
            if let Expr::Fn(fn_expr) = &*arg.expr {
              self.check_function_for_throws(&fn_expr.function);
              self.visit_function(&fn_expr.function);
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

        if let Expr::Object(object_expr) = &**init {
          self.current_class_name = Some(function_name.clone());
          self.visit_object_lit(object_expr);
          self.current_class_name = None;
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
    if let Some(block_stmt) = &function.body {
      for stmt in &block_stmt.stmts {
        self.visit_stmt(stmt);
      }
    }
    self.check_function_for_throws(function);
    swc_ecma_visit::visit_function(self, function);
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
        self.visit_stmt(&if_stmt.cons);
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
    if let Expr::Call(call_expr) = expr {
      self.visit_call_expr(call_expr)
    }
    swc_ecma_visit::visit_expr(self, expr);
  }

  fn visit_constructor(&mut self, constructor: &Constructor) {
    self.current_method_name = Some("<constructor>".to_string());
    self.check_constructor_for_throws(constructor);
    if let Some(constructor) = &constructor.body {
      for stmt in &constructor.stmts {
        self.visit_stmt(stmt);
      }
    }
    swc_ecma_visit::visit_constructor(self, constructor);
    self.current_method_name = None;
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
    }
    // else if let Decl::Var(var_decl) = &export_decl.decl {
    //   for declar in &var_decl.decls {
    //     if let Some(ident) = &declar.name.as_ident() {
    //       self.current_class_name = Some(ident.sym.to_string());
    //     }
    //     self.visit_var_declarator(&declar);
    //     self.current_class_name = None;
    //   }

    // }
    else {
      swc_ecma_visit::visit_export_decl(self, export_decl);
    }
  }
}
