use crate::language::{LanguageAnalyzer, LanguageConfig};
use tree_sitter::{Node, Parser};

pub struct GoAnalyzer;

const FUNCTION_KINDS: &[&str] = &["function_declaration", "method_declaration", "func_literal"];
const CLOSURE_KINDS: &[&str] = &["func_literal"];
const DECISION_KINDS: &[&str] = &[
    "if_statement",
    "for_statement",
    "expression_case",
    "type_case",
    "communication_case",
    "default_case",
];
const OPERATOR_KINDS: &[&str] = &[
    "+",
    "-",
    "*",
    "/",
    "%",
    "==",
    "!=",
    "<",
    ">",
    "<=",
    ">=",
    "&&",
    "||",
    "!",
    "&",
    "&^",
    "^",
    "<<",
    ">>",
    "|",
    "=",
    ":=",
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "&=",
    "&^=",
    "|=",
    "^=",
    "<<=",
    ">>=",
    "++",
    "--",
    ".",
    "<-",
    "return_statement",
    "break_statement",
    "continue_statement",
    "goto_statement",
    "defer_statement",
    "go_statement",
];
const OPERAND_KINDS: &[&str] = &[
    "identifier",
    "int_literal",
    "float_literal",
    "interpreted_string_literal",
    "raw_string_literal",
    "rune_literal",
    "true",
    "false",
    "nil",
    "iota",
];

impl LanguageAnalyzer for GoAnalyzer {
    fn can_analyze(&self, path: &std::path::Path) -> bool {
        path.extension().map_or(false, |e| e == "go")
    }

    fn language_name(&self) -> &'static str {
        "Go"
    }

    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_go::LANGUAGE.into())
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
            token_classifier: None,
        }
    }
}

fn extract_name(node: Node, source: &str) -> String {
    if node.kind() == "func_literal" {
        return format!("<closure>@line {}", node.start_position().row + 1);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "field_identifier" {
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
        let source = "func foo() { if true {} }";
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_method() {
        let source = r#"
func (r *Receiver) Bar() {
    if x {}
}
"#;
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Bar");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_if_else() {
        let source = "func bar() { if x {} else {} }";
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2); // base 1 + if 1
    }

    #[test]
    fn test_switch() {
        let source = r#"
func baz() {
    switch x {
    case 1:
    case 2:
    default:
    }
}
"#;
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4); // base 1 + 2 cases + 1 default
    }

    #[test]
    fn test_type_switch() {
        let source = r#"
func qux() {
    switch x := y.(type) {
    case int:
    case string:
    default:
    }
}
"#;
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4); // base 1 + 2 cases + 1 default
    }

    #[test]
    fn test_select() {
        let source = r#"
func sel() {
    select {
    case <-ch1:
    case ch2 <- 1:
    default:
    }
}
"#;
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4); // base 1 + 2 cases + 1 default
    }

    #[test]
    fn test_for_loop() {
        let source = "func loop() { for i := 0; i < 10; i++ {} }";
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2); // base 1 + for 1
    }

    #[test]
    fn test_range() {
        let source = "func r() { for _, v := range m {} }";
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2); // base 1 + for 1
    }

    #[test]
    fn test_closure_included() {
        let source = "func outer() { f := func(x int) { if x > 0 {} } }";
        let analyzer = GoAnalyzer;
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
        let source = "func outer() { f := func(x int) { if x > 0 {} } }";
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert!(!result[0].name.starts_with("<closure>"));
    }

    #[test]
    fn test_boolean_ops() {
        let source = "func b() { a && b || c }";
        let analyzer = GoAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + && 1 + || 1
    }

    #[test]
    fn test_parse_error() {
        let source = "func foo() {";
        let analyzer = GoAnalyzer;
        assert!(analyzer.analyze(source, false).is_err());
    }

    #[test]
    fn test_can_analyze_go() {
        let analyzer = GoAnalyzer;
        assert!(analyzer.can_analyze(std::path::Path::new("foo.go")));
        assert!(!analyzer.can_analyze(std::path::Path::new("foo.rs")));
    }
}
