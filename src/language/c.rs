use crate::language::{LanguageAnalyzer, LanguageConfig};
use tree_sitter::{Node, Parser};

pub struct CAnalyzer;

const FUNCTION_KINDS: &[&str] = &["function_definition"];
const DECISION_KINDS: &[&str] = &[
    "if_statement",
    "for_statement",
    "while_statement",
    "do_statement",
    "case_statement",
    "conditional_expression",
];
const OPERATOR_KINDS: &[&str] = &[
    "+", "-", "*", "/", "%",
    "==", "!=", "<", ">", "<=", ">=",
    "&&", "||", "!",
    "=", "+=", "-=", "*=", "/=", "%=",
    "&", "|", "^", "<<", ">>", "~",
    ".", "->", ":",
    "return_statement", "break_statement", "continue_statement", "goto_statement",
];
const OPERAND_KINDS: &[&str] = &[
    "identifier", "number_literal", "string_literal", "char_literal",
];

impl LanguageAnalyzer for CAnalyzer {
    fn can_analyze(&self, path: &std::path::Path) -> bool {
        path.extension().map_or(false, |e| e == "c" || e == "h")
    }

    fn language_name(&self) -> &'static str {
        "C"
    }

    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_c::LANGUAGE.into())
    }

    fn config(&self) -> LanguageConfig {
        LanguageConfig {
            function_kinds: FUNCTION_KINDS,
            closure_kinds: &[],
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
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return source[child.start_byte()..child.end_byte()].to_string();
        }
        // The declarator may be wrapped in pointer_declarator or function_declarator
        let name = find_identifier_in_declarator(child, source);
        if !name.is_empty() {
            return name;
        }
    }
    format!("<anon>@line {}", node.start_position().row + 1)
}

fn find_identifier_in_declarator(node: Node, source: &str) -> String {
    if node.kind() == "identifier" {
        return source[node.start_byte()..node.end_byte()].to_string();
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let name = find_identifier_in_declarator(child, source);
        if !name.is_empty() {
            return name;
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let source = "int foo() { if (1) {} }";
        let analyzer = CAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_if_else() {
        let source = "int bar() { if (x) {} else {} }";
        let analyzer = CAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2); // base 1 + if 1
    }

    #[test]
    fn test_switch() {
        let source = r#"
int baz() {
    switch (x) {
        case 1: break;
        case 2: break;
        default: break;
    }
}
"#;
        let analyzer = CAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4); // base 1 + 2 cases + 1 default
    }

    #[test]
    fn test_for_while_do() {
        let source = r#"
int loop() {
    for (;;) {}
    while (1) {}
    do {} while (1);
}
"#;
        let analyzer = CAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4); // base 1 + for 1 + while 1 + do 1
    }

    #[test]
    fn test_ternary() {
        let source = "int t() { return x > 0 ? 1 : 0; }";
        let analyzer = CAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2); // base 1 + ternary 1
    }

    #[test]
    fn test_boolean_ops() {
        let source = "int b() { return a && b || c; }";
        let analyzer = CAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + && 1 + || 1
    }

    #[test]
    fn test_pointer_return_name() {
        let source = "int *foo() { return 0; }";
        let analyzer = CAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "foo");
    }

    #[test]
    fn test_parse_error() {
        let source = "int foo() {";
        let analyzer = CAnalyzer;
        assert!(analyzer.analyze(source, false).is_err());
    }

    #[test]
    fn test_can_analyze_header() {
        let analyzer = CAnalyzer;
        assert!(analyzer.can_analyze(std::path::Path::new("foo.h")));
    }
}
