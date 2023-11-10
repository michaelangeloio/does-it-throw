extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;

use std::collections::HashSet;

use swc_ecma_ast::Callee;

use crate::throw_finder::IdentifierUsage;

use self::swc_ecma_ast::{CallExpr, Expr, MemberProp};

use self::swc_ecma_visit::Visit;

pub struct ImportUsageFinder {
  pub imported_identifiers: Vec<String>,
  pub imported_identifier_usages: HashSet<IdentifierUsage>,
  pub current_class_name: Option<String>,
  pub current_method_name: Option<String>,
  pub function_name_stack: Vec<String>,
}

impl Visit for ImportUsageFinder {
  fn visit_call_expr(&mut self, call: &CallExpr) {
    if let Callee::Expr(expr) = &call.callee {
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
              self.imported_identifier_usages.insert(usage_map);
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
              self.visit_arrow_expr(arrow_expr)
            }
            if let Expr::Fn(fn_expr) = &*arg.expr {
              self.visit_function(&fn_expr.function)
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
            self.imported_identifier_usages.insert(usage_map);
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
          self.current_class_name = None;
          self.current_method_name = None;
        }

        Expr::Arrow(_arrow_expr) => {}
        _ => {}
      }
    }
  }
}
