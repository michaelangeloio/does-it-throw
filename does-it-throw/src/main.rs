mod shared;

use shared::{analyze_code, AnalysisResult,};
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

