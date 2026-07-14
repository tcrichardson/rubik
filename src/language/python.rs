use crate::language::{LanguageAnalyzer, LanguageConfig, TokenRole};
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
    "+",
    "-",
    "*",
    "/",
    "%",
    "//",
    "**",
    "==",
    "!=",
    "<",
    ">",
    "<=",
    ">=",
    "and",
    "or",
    "not",
    "in",
    "is",
    "=",
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "//=",
    "**=",
    "&",
    "|",
    "^",
    "<<",
    ">>",
    "~",
    ".",
    ":",
    "->",
    "return_statement",
    "yield",
    "await",
];
const OPERAND_KINDS: &[&str] = &[
    "identifier",
    "integer",
    "float",
    "string",
    "true",
    "false",
    "none",
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
            token_classifier: Some(classify_token),
        }
    }
}

/// Classifies each CST node for clone-detection token normalization:
/// call/method target names are preserved verbatim, local identifiers and
/// literal values are replaced with generic placeholders, everything else
/// contributes its node kind (structural).
fn classify_token(node: Node, _source: &str) -> TokenRole {
    match node.kind() {
        "identifier" => {
            if is_call_target(node) || is_method_call_target(node) {
                TokenRole::Preserve
            } else {
                TokenRole::Normalize("<ID>")
            }
        }
        "integer" | "float" | "string" | "true" | "false" | "none" => TokenRole::Normalize("<LIT>"),
        _ => TokenRole::Structural,
    }
}

/// True when `node` is the callee identifier of a direct function call, e.g.
/// `foo` in `foo(x)`.
fn is_call_target(node: Node) -> bool {
    node.parent().is_some_and(|parent| {
        parent.kind() == "call" && parent.child_by_field_name("function") == Some(node)
    })
}

/// True when `node` is the method-name identifier of a method call, e.g.
/// `method` in `obj.method(x)`. Plain attribute access (`obj.field` with no
/// call) is not a call target and is normalized like any other identifier.
fn is_method_call_target(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != "attribute" || parent.child_by_field_name("attribute") != Some(node) {
        return false;
    }
    parent.parent().is_some_and(|grandparent| {
        grandparent.kind() == "call" && grandparent.child_by_field_name("function") == Some(parent)
    })
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

    #[test]
    fn test_clone_tokens_not_computed_without_flag() {
        let source = "def foo():\n    bar(1)\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert!(result[0].clone_tokens.is_empty());
    }

    #[test]
    fn test_clone_tokens_preserve_call_target() {
        let source = "def foo(a):\n    bar(a)\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze_full(source, false, true).unwrap();
        let tokens = &result[0].clone_tokens;
        assert!(
            tokens.contains(&"bar".to_string()),
            "expected call target 'bar' preserved verbatim in {:?}",
            tokens
        );
    }

    #[test]
    fn test_clone_tokens_preserve_method_call_target() {
        let source = "def foo(obj):\n    obj.method(1)\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze_full(source, false, true).unwrap();
        let tokens = &result[0].clone_tokens;
        assert!(
            tokens.contains(&"method".to_string()),
            "expected method call target 'method' preserved verbatim in {:?}",
            tokens
        );
    }

    #[test]
    fn test_clone_tokens_normalize_local_identifiers_and_literals() {
        let source = "def foo():\n    x = 5\n    bar(x)\n";
        let analyzer = PythonAnalyzer;
        let result = analyzer.analyze_full(source, false, true).unwrap();
        let tokens = &result[0].clone_tokens;
        assert!(
            !tokens.contains(&"x".to_string()),
            "local identifier 'x' must be normalized, got {:?}",
            tokens
        );
        assert!(tokens.contains(&"<ID>".to_string()));
        assert!(tokens.contains(&"<LIT>".to_string()));
    }

    #[test]
    fn test_clone_tokens_two_renamed_functions_produce_equal_sequences() {
        let a = "def foo():\n    x = 5\n    bar(x)\n";
        let b = "def renamed():\n    y = 5\n    bar(y)\n";
        let analyzer = PythonAnalyzer;
        let ra = analyzer.analyze_full(a, false, true).unwrap();
        let rb = analyzer.analyze_full(b, false, true).unwrap();
        assert_eq!(ra[0].clone_tokens, rb[0].clone_tokens);
    }
}
