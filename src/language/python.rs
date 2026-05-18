use crate::language::{LanguageAnalyzer, LanguageConfig};
use tree_sitter::{Node, Parser};

pub struct PythonAnalyzer;

const FUNCTION_KINDS: &[&str] = &["function_definition", "lambda"];
const CLOSURE_KINDS: &[&str] = &["lambda"];
const DECISION_KINDS: &[&str] = &[
    "if_statement",
    "elif_clause",
    "for_statement",
    "while_statement",
    "except_clause",
    "conditional_expression",
];
const OPERATOR_KINDS: &[&str] = &[
    "+", "-", "*", "/", "%", "//", "**",
    "==", "!=", "<", ">", "<=", ">=",
    "and", "or", "not", "in", "is",
    "=", "+=", "-=", "*=", "/=", "%=", "//=", "**=",
    "&", "|", "^", "<<", ">>", "~",
    ".", ":", "->",
    "return_statement", "yield", "await",
];
const OPERAND_KINDS: &[&str] = &[
    "identifier", "integer", "float", "string", "true", "false", "none",
];

impl LanguageAnalyzer for PythonAnalyzer {
    fn can_analyze(&self, path: &std::path::Path) -> bool {
        path.extension().map_or(false, |e| e == "py")
    }

    fn language_name(&self) -> &'static str {
        "Python"
    }

    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_python::LANGUAGE.into())
    }

    fn config(&self) -> LanguageConfig {
        LanguageConfig {
            function_kinds: FUNCTION_KINDS,
            closure_kinds: CLOSURE_KINDS,
            decision_kinds: DECISION_KINDS,
            operator_kinds: OPERATOR_KINDS,
            operand_kinds: OPERAND_KINDS,
            extract_name,
            match_case_kinds: &[("match_statement", "case_clause")],
            skip_childless_nodes: true,
        }
    }
}

fn extract_name(node: Node, source: &str) -> String {
    if node.kind() == "lambda" {
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
    fn test_simple_function() {
        let source = "def foo():\n    if x:\n        pass\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_if_elif_else() {
        let source = "def bar():\n    if x:\n        pass\n    elif y:\n        pass\n    else:\n        pass\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + if 1 + elif 1
    }

    #[test]
    fn test_match() {
        let source = "def baz():\n    match x:\n        case 1:\n            pass\n        case 2:\n            pass\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + 2 cases
    }

    #[test]
    fn test_lambda_included() {
        let source = "f = lambda x: 1 if x > 0 else 0\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze(source, true).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].name.starts_with("<lambda>"));
        assert_eq!(result[0].complexity, 2); // base 1 + ternary 1
    }

    #[test]
    fn test_lambda_excluded_by_default() {
        let source = "f = lambda x: 1 if x > 0 else 0\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_try_except() {
        let source = "def err():\n    try:\n        pass\n    except A:\n        pass\n    except B:\n        pass\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + except A 1 + except B 1
    }
}
