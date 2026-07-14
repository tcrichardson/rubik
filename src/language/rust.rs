use crate::language::{LanguageAnalyzer, LanguageConfig, TokenRole};
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
            if is_call_target(node) {
                TokenRole::Preserve
            } else {
                TokenRole::Normalize("<ID>")
            }
        }
        "field_identifier" => {
            if is_method_call_target(node) {
                TokenRole::Preserve
            } else {
                TokenRole::Normalize("<ID>")
            }
        }
        "integer_literal" | "float_literal" | "string_literal" | "char_literal"
        | "boolean_literal" => TokenRole::Normalize("<LIT>"),
        _ => TokenRole::Structural,
    }
}

/// True when `node` is the callee identifier of a direct or path-qualified
/// function call, e.g. `foo` in `foo(x)` or `from` in `String::from(x)`.
fn is_call_target(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() == "call_expression" && parent.child_by_field_name("function") == Some(node) {
        return true;
    }
    // Path-qualified call, e.g. `String::from(x)`: `node` is the `name` field
    // of a `scoped_identifier` that is itself (possibly via turbofish) the
    // callee of a call_expression.
    if parent.kind() == "scoped_identifier" && parent.child_by_field_name("name") == Some(node) {
        return is_function_of_call(parent);
    }
    false
}

/// True when `node` is the method-name identifier of a method call, e.g.
/// `method` in `obj.method(x)` or `collect` in `iter.collect::<Vec<_>>()`.
/// Plain field access (`obj.field` with no call) is not a call target and is
/// normalized like any other identifier.
fn is_method_call_target(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != "field_expression" || parent.child_by_field_name("field") != Some(node) {
        return false;
    }
    is_function_of_call(parent)
}

/// True when `node` is (possibly through a `generic_function` turbofish
/// wrapper, e.g. `.collect::<Vec<_>>()`) the `function` field of a
/// `call_expression`.
fn is_function_of_call(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        "call_expression" => parent.child_by_field_name("function") == Some(node),
        "generic_function" => {
            parent.child_by_field_name("function") == Some(node) && is_function_of_call(parent)
        }
        _ => false,
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

    #[test]
    fn test_clone_tokens_not_computed_without_flag() {
        let source = "fn foo() { bar(1); }";
        let analyzer = RustAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert!(result[0].clone_tokens.is_empty());
    }

    #[test]
    fn test_clone_tokens_preserve_call_target() {
        let source = "fn foo(a: i32) { bar(a); }";
        let analyzer = RustAnalyzer;
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
        let source = "fn foo(obj: Thing) { obj.method(1); }";
        let analyzer = RustAnalyzer;
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
        let source = "fn foo() { let x = 5; bar(x); }";
        let analyzer = RustAnalyzer;
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
    fn test_clone_tokens_preserve_path_qualified_call_target() {
        let source = r#"fn foo() { String::from("x"); }"#;
        let analyzer = RustAnalyzer;
        let result = analyzer.analyze_full(source, false, true).unwrap();
        let tokens = &result[0].clone_tokens;
        assert!(
            tokens.contains(&"from".to_string()),
            "expected path-qualified call target 'from' preserved verbatim in {:?}",
            tokens
        );
        assert!(
            !tokens.contains(&"String".to_string()),
            "the path segment 'String' is not itself the call target and should be normalized in {:?}",
            tokens
        );
    }

    #[test]
    fn test_clone_tokens_preserve_turbofish_method_call_target() {
        let source = "fn foo(v: Vec<i32>) { v.iter().collect::<Vec<i32>>(); }";
        let analyzer = RustAnalyzer;
        let result = analyzer.analyze_full(source, false, true).unwrap();
        let tokens = &result[0].clone_tokens;
        assert!(
            tokens.contains(&"iter".to_string()),
            "expected 'iter' preserved verbatim in {:?}",
            tokens
        );
        assert!(
            tokens.contains(&"collect".to_string()),
            "expected turbofish method call target 'collect' preserved verbatim in {:?}",
            tokens
        );
    }

    #[test]
    fn test_clone_tokens_two_renamed_functions_produce_equal_sequences() {
        let a = "fn foo() { let x = 5; bar(x); }";
        let b = "fn renamed() { let y = 5; bar(y); }";
        let analyzer = RustAnalyzer;
        let ra = analyzer.analyze_full(a, false, true).unwrap();
        let rb = analyzer.analyze_full(b, false, true).unwrap();
        assert_eq!(ra[0].clone_tokens, rb[0].clone_tokens);
    }
}
