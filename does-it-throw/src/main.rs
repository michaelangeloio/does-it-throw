extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_visit;

use std::fs;
use swc_common::{sync::Lrc, SourceMap, SourceMapper, Span};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Visit;

fn main() {
    let path = String::from("./testing/some-file.ts");
    let content = fs::read_to_string(path.clone()).expect("Could not read the file");
    let cm: Lrc<SourceMap> = Default::default();
    let (results, cm) = analyze_code(&content, cm, &path);

    println!("Functions that throw:");
    for fun in &results.functions_with_throws {
        let start = cm.lookup_char_pos(fun.throw_statement.lo());
        let end = cm.lookup_char_pos(fun.throw_statement.hi());
        println!(
            "From line {} column {} to line {} column {}",
            start.line, start.col_display, end.line, end.col_display
        );
        for span in &fun.throw_spans {
            let start = cm.lookup_char_pos(span.lo());
            let end = cm.lookup_char_pos(span.hi());
            println!(
                "  Throw from line {} column {} to line {} column {}",
                start.line, start.col_display, end.line, end.col_display
            );
        }
    }

    println!("\nOccurrences of JSON.parse:");
    for occurrence in &results.json_parse_calls {
        println!("{}", occurrence);
    }

    println!("\nOccurrences of fs.access:");
    for occurrence in &results.fs_access_calls {
        println!("{}", occurrence);
    }
}

fn analyze_code(content: &str, cm: Lrc<SourceMap>, path: &str) -> (AnalysisResult, Lrc<SourceMap>) {
    let fm = cm.new_source_file(swc_common::FileName::Anon, content.into());
    let lexer = Lexer::new(
        Syntax::Typescript(swc_ecma_parser::TsConfig {
            tsx: path.ends_with(".tsx"),
            decorators: true,
            dts: false,
            no_early_errors: false,
            disallow_ambiguous_jsx_like: false,
        }),
        EsVersion::Es2020,
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

struct ThrowMap {
    throw_spans: Vec<Span>,
    throw_statement: Span,
}

struct AnalysisResult {
    functions_with_throws: Vec<ThrowMap>,
    json_parse_calls: Vec<String>,
    fs_access_calls: Vec<String>,
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
