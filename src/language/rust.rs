use crate::language::{LanguageAnalyzer, LanguageConfig};
use tree_sitter::{Node, Parser};

pub struct RustAnalyzer;

const FUNCTION_KINDS: &[&str] = &["function_item", "closure_expression"];
const CLOSURE_KINDS: &[&str] = &["closure_expression"];
const DECISION_KINDS: &[&str] = &[
    "if_expression",
    "if_let_expression",
    "for_expression",
    "while_expression",
    "while_let_expression",
    "loop_expression",
    "try_expression",
    "match_arm",
];
const OPERATOR_KINDS: &[&str] = &[
    "+",
    "-",
    "*",
    "/",
    "%",
    "&&",
    "||",
    "!",
    "==",
    "!=",
    "<",
    ">",
    "<=",
    ">=",
    "=",
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "&",
    "|",
    "^",
    "<<",
    ">>",
    ".",
    "..",
    "...",
    "->",
    "=>",
    "return_expression",
    "break_expression",
    "continue_expression",
    "await_expression",
    "try_expression",
];
const OPERAND_KINDS: &[&str] = &[
    "identifier",
    "integer_literal",
    "float_literal",
    "string_literal",
    "char_literal",
    "bool_literal",
    "self",
];

impl LanguageAnalyzer for RustAnalyzer {
    fn can_analyze(&self, path: &std::path::Path) -> bool {
        path.extension().map_or(false, |e| e == "rs")
    }

    fn language_name(&self) -> &'static str {
        "Rust"
    }

    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_rust::LANGUAGE.into())
    }

    fn config(&self) -> LanguageConfig {
        LanguageConfig {
            function_kinds: FUNCTION_KINDS,
            closure_kinds: CLOSURE_KINDS,
            decision_kinds: DECISION_KINDS,
            operator_kinds: OPERATOR_KINDS,
            operand_kinds: OPERAND_KINDS,
            extract_name,
            match_case_kinds: &[],
            skip_childless_nodes: false,
        }
    }
}

fn extract_name(node: Node, source: &str) -> String {
    if node.kind() == "closure_expression" {
        return format!("<closure>@line {}", node.start_position().row + 1);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return source[child.start_byte()..child.end_byte()].to_string();
        }
    }
    format!("<anon>@line {}", node.start_position().row + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let source = "fn foo() { if true {} }";
        let analyzer = RustAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_if_else_if() {
        let source = r#"
fn bar() {
    if x {}
    else if y {}
    else {}
}
"#;
        let analyzer = RustAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + if 1 + else-if 1
    }

    #[test]
    fn test_match() {
        let source = r#"
fn baz() {
    match x {
        1 => {}
        2 => {}
        _ => {}
    }
}
"#;
        let analyzer = RustAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4); // base 1 + 3 arms
    }

    #[test]
    fn test_closure_included() {
        let source = "fn outer() { let f = |x| if x > 0 { 1 } else { 0 }; }";
        let analyzer = RustAnalyzer;
        let result = analyzer.analyze(source, true).unwrap();
        assert_eq!(result.len(), 2);
        let closure = result
            .iter()
            .find(|f| f.name.starts_with("<closure>"))
            .unwrap();
        assert_eq!(closure.complexity, 2);
    }

    #[test]
    fn test_closure_excluded_by_default() {
        let source = "fn outer() { let f = |x| if x > 0 { 1 } else { 0 }; }";
        let analyzer = RustAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert!(!result[0].name.starts_with("<closure>"));
    }

    #[test]
    fn test_boolean_ops() {
        let source = "fn b() { a && b || c; }";
        let analyzer = RustAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + && 1 + || 1
    }
}
