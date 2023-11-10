extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::vec;

use swc_ecma_ast::{
  ArrowExpr, AssignExpr, AwaitExpr, BinExpr, BlockStmtOrExpr, Callee, ClassDecl, ClassMethod,
  Constructor, Decl, ExportDecl, FnDecl, JSXAttr, JSXAttrOrSpread, JSXAttrValue, JSXExpr,
  JSXOpeningElement, MemberExpr, ObjectLit, OptChainBase, OptChainExpr, ParenExpr, PatOrExpr, Prop,
  PropName, PropOrSpread, Stmt, VarDeclarator,
};

use crate::throw_finder::ThrowMap;

use self::swc_common::{sync::Lrc, SourceMap, Span};
use self::swc_ecma_ast::{
  CallExpr, EsVersion, Expr, Function, ImportDecl, ImportSpecifier, MemberProp, ModuleExportName,
  ThrowStmt,
};
use self::swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use self::swc_ecma_visit::Visit;

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

pub struct InstantiationsMap {
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

// ----- CallFinder Visitor implementation -----
// This module defines structures and implements functionality for identifying and mapping
// function calls to their respective functions or methods that throw exceptions. It uses
// SWC's visitor pattern to traverse the AST (Abstract Syntax Tree) of JavaScript or TypeScript code.
///
// `CallToThrowMap` records the mapping of a function call to a function that throws.
// It captures the span of the call, the name of the function/method being called,
// the class name if the call is a method call, and the `ThrowMap` that provides details
// about the throw statement in the called function/method.
///
// `InstantiationsMap` keeps track of class instantiations by recording the class name
// and the variable name that holds the instance.
///
// `CallFinder` is the core structure that uses the Visitor pattern to traverse the AST nodes.
// It maintains state as it goes through the code, keeping track of current class names,
// function name stacks, and object property stacks. As it finds function calls, it tries
// to match them with known functions or methods that throw exceptions using the data
// accumulated in `functions_with_throws`. When a match is found, it records the mapping
// in `calls`. It also tracks instantiations of classes to help resolve method calls.

pub struct CallFinder {
  pub calls: HashSet<CallToThrowMap>,
  pub functions_with_throws: HashSet<ThrowMap>,
  pub current_class_name: Option<String>,
  pub instantiations: HashSet<InstantiationsMap>,
  pub function_name_stack: Vec<String>,
  pub object_property_stack: Vec<String>,
}

impl CallFinder {
  fn handle_bin_expr(&mut self, bin_expr: &BinExpr) {
    if let Expr::Call(call_expr) = &*bin_expr.left {
      self.visit_call_expr(call_expr);
    }
    if let Expr::Call(call_expr) = &*bin_expr.right {
      self.visit_call_expr(call_expr);
    }
    if let Expr::Await(await_expr) = &*bin_expr.left {
      self.handle_await_expr(await_expr);
    }
    if let Expr::Await(await_expr) = &*bin_expr.right {
      self.handle_await_expr(await_expr);
    }
    if let Expr::OptChain(opt_chain_expr) = &*bin_expr.left {
      self.handle_opt_chain_expr(opt_chain_expr);
    }
    if let Expr::OptChain(opt_chain_expr) = &*bin_expr.right {
      self.handle_opt_chain_expr(opt_chain_expr);
    }
    if let Expr::Paren(paren_expr) = &*bin_expr.left {
      self.handle_paren_expr(paren_expr);
    }
    if let Expr::Paren(paren_expr) = &*bin_expr.right {
      self.handle_paren_expr(paren_expr);
    }
  }

  fn handle_paren_expr(&mut self, paren_expr: &ParenExpr) {
    if let Expr::Call(call_expr) = &*paren_expr.expr {
      self.visit_call_expr(call_expr);
    }
    if let Expr::Await(await_expr) = &*paren_expr.expr {
      self.handle_await_expr(await_expr);
    }
    if let Expr::OptChain(opt_chain_expr) = &*paren_expr.expr {
      self.handle_opt_chain_expr(opt_chain_expr);
    }
  }

  fn handle_opt_chain_expr(&mut self, opt_chain_expr: &OptChainExpr) {
    if let OptChainBase::Member(expr) = &*opt_chain_expr.base {
      if let Expr::Call(call_expr) = &*expr.obj {
        self.visit_call_expr(call_expr);
      }
    }
  }

  fn handle_await_expr(&mut self, await_expr: &AwaitExpr) {
    if let Expr::Call(call_expr) = &*await_expr.arg {
      self.visit_call_expr(call_expr);
    }
  }
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
    if let Callee::Expr(expr) = &call.callee {
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
      if let Expr::Bin(bin_expr) = &**init_expr {
        self.handle_bin_expr(bin_expr)
      }
    }
    if let Some(ident) = &var_declarator.name.as_ident() {
      if let Some(init) = &var_declarator.init {
        if let Expr::Await(await_expr) = &**init {
          if let Expr::Call(call_expr) = &*await_expr.arg {
            self.visit_call_expr(call_expr);
          }
        }
        if let Expr::Bin(bin_expr) = &**init {
          if let Expr::Call(call_expr) = &*bin_expr.left {
            self.visit_call_expr(call_expr);
          }
          if let Expr::Call(call_expr) = &*bin_expr.right {
            self.visit_call_expr(call_expr);
          }
        }
        if let Expr::OptChain(opt_chain_expr) = &**init {
          if let OptChainBase::Member(expr) = &*opt_chain_expr.base {
            if let Expr::Call(call_expr) = &*expr.obj {
              self.visit_call_expr(call_expr);
            }
          }
        }
        if let Expr::Arrow(arrow_expr) = &**init {
          self.function_name_stack.push(ident.sym.to_string());
          self.visit_arrow_expr(arrow_expr);
          self.function_name_stack.pop();
        }
        if let Expr::Fn(fn_expr) = &**init {
          self.function_name_stack.push(ident.sym.to_string());
          self.visit_function(&fn_expr.function);
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
      Stmt::Decl(decl) => {
        if let Decl::Var(var_decl) = decl {
          for decl in &var_decl.decls {
            self.visit_var_declarator(decl);
          }
        }
        if let Decl::Fn(fn_decl) = decl {
          self.visit_fn_decl(fn_decl);
        }
        if let Decl::Class(class_decl) = decl {
          self.visit_class_decl(class_decl);
        }
      }
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
