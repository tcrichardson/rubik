use crate::language::{LanguageAnalyzer, LanguageConfig};
use tree_sitter::{Node, Parser};

pub struct JavaAnalyzer;

const FUNCTION_KINDS: &[&str] = &[
    "method_declaration",
    "constructor_declaration",
    "compact_constructor_declaration",
    "lambda_expression",
];
const CLOSURE_KINDS: &[&str] = &["lambda_expression"];
const DECISION_KINDS: &[&str] = &[
    "if_statement",
    "for_statement",
    "enhanced_for_statement",
    "while_statement",
    "do_statement",
    "switch_label",
    "catch_clause",
    "ternary_expression",
];
const OPERATOR_KINDS: &[&str] = &[
    "+", "-", "*", "/", "%",
    "==", "!=", "<", ">", "<=", ">=",
    "&&", "||", "!",
    "=", "+=", "-=", "*=", "/=", "%=",
    "&", "|", "^", "<<", ">>", ">>>", "~",
    ".", "::",
    "return_statement", "break_statement", "continue_statement",
    "throw_statement", "yield_statement",
    "instanceof_expression",
];
const OPERAND_KINDS: &[&str] = &[
    "identifier",
    "decimal_integer_literal",
    "hex_integer_literal",
    "octal_integer_literal",
    "binary_integer_literal",
    "decimal_floating_point_literal",
    "hex_floating_point_literal",
    "string_literal",
    "character_literal",
    "true",
    "false",
    "null_literal",
    "this",
    "super",
];

impl LanguageAnalyzer for JavaAnalyzer {
    fn can_analyze(&self, path: &std::path::Path) -> bool {
        path.extension().map_or(false, |e| e == "java")
    }

    fn language_name(&self) -> &'static str {
        "Java"
    }

    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_java::LANGUAGE.into())
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
    if node.kind() == "lambda_expression" {
        return format!("<lambda>@line {}", node.start_position().row + 1);
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
    fn test_simple_method() {
        let source = "void foo() { if (true) {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_if_else() {
        let source = "void bar() { if (x) {} else {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_for_loop() {
        let source = "void foo() { for (int i = 0; i < 10; i++) {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_enhanced_for_loop() {
        let source = "void foo() { for (String s : list) {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_while_and_do_while() {
        let source = "void foo() { while (true) {} do {} while (true); }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3);
    }

    #[test]
    fn test_switch_with_cases() {
        let source = "void foo() { switch (x) { case 1: break; case 2: break; default: break; } }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4);
    }

    #[test]
    fn test_try_catch() {
        let source = "void foo() { try {} catch (Exception e) {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_lambda_included() {
        let source = "void foo() { Runnable r = () -> { if (true) {} }; }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, true).unwrap();
        assert_eq!(result.len(), 2);
        let lambda = result.iter().find(|f| f.name.starts_with("<lambda>")).unwrap();
        assert_eq!(lambda.complexity, 2);
    }

    #[test]
    fn test_lambda_excluded_by_default() {
        let source = "void foo() { Runnable r = () -> { if (true) {} }; }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert!(!result[0].name.starts_with("<lambda>"));
    }

    #[test]
    fn test_ternary() {
        let source = "int t() { return x > 0 ? 1 : 0; }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_boolean_ops() {
        let source = "void b() { boolean r = a && b || c; }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3);
    }

    #[test]
    fn test_constructor_name() {
        let source = "class Foo { Foo() { if (true) {} } }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_compact_constructor() {
        let source = "record Point(int x, int y) { Point { if (x < 0) {} } }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        // Compact constructor name is extracted from the AST
        // The name should be "Point" if the identifier is found
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_parse_error() {
        let source = "void foo() {";
        let analyzer = JavaAnalyzer;
        assert!(analyzer.analyze(source, false).is_err());
    }

    #[test]
    fn test_can_analyze_java_extension() {
        let analyzer = JavaAnalyzer;
        assert!(analyzer.can_analyze(std::path::Path::new("Foo.java")));
        assert!(!analyzer.can_analyze(std::path::Path::new("Foo.rs")));
    }
}
