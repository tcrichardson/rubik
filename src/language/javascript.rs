use crate::language::javascript_like::{
    CLOSURE_KINDS, DECISION_KINDS, FUNCTION_KINDS, OPERAND_KINDS, OPERATOR_KINDS, classify_token,
    extract_name,
};
use crate::language::{LanguageAnalyzer, LanguageConfig};
use tree_sitter::Parser;

pub struct JavaScriptAnalyzer;

impl LanguageAnalyzer for JavaScriptAnalyzer {
    fn can_analyze(&self, path: &std::path::Path) -> bool {
        path.extension().map_or(false, |e| e == "js" || e == "jsx")
    }

    fn language_name(&self) -> &'static str {
        "JS"
    }

    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_javascript::LANGUAGE.into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let source = "function foo() { if (x) {} }";
        let analyzer = JavaScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_if_else() {
        let source = "function bar() { if (x) {} else if (y) {} else {} }";
        let analyzer = JavaScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + if 1 + else-if 1
    }

    #[test]
    fn test_switch() {
        let source =
            "function baz() { switch(x) { case 1: break; case 2: break; default: break; } }";
        let analyzer = JavaScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4); // base 1 + 3 cases
    }

    #[test]
    fn test_arrow_function_included() {
        let source = "const f = (x) => x > 0 ? 1 : 0;";
        let analyzer = JavaScriptAnalyzer;
        let result = analyzer.analyze(source, true).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].name.starts_with("<closure>"));
        assert_eq!(result[0].complexity, 2); // base 1 + ternary 1
    }

    #[test]
    fn test_arrow_function_excluded_by_default() {
        let source = "const f = (x) => x > 0 ? 1 : 0;";
        let analyzer = JavaScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_try_catch() {
        let source = "function err() { try {} catch (a) { if (b) {} } }";
        let analyzer = JavaScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + catch 1 + if 1
    }

    #[test]
    fn test_boolean_ops() {
        let source = "function b() { return a && b || c; }";
        let analyzer = JavaScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + && 1 + || 1
    }

    #[test]
    fn test_clone_tokens_not_computed_without_flag() {
        let source = "function foo() { bar(1); }";
        let analyzer = JavaScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert!(result[0].clone_tokens.is_empty());
    }

    #[test]
    fn test_clone_tokens_preserve_call_target() {
        let source = "function foo(a) { bar(a); }";
        let analyzer = JavaScriptAnalyzer;
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
        let source = "function foo(obj) { obj.method(1); }";
        let analyzer = JavaScriptAnalyzer;
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
        let source = "function foo() { let x = 5; bar(x); }";
        let analyzer = JavaScriptAnalyzer;
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
    fn test_clone_tokens_preserve_chained_method_call_target() {
        let source = "function foo(items) { items.filter(x).map(y); }";
        let analyzer = JavaScriptAnalyzer;
        let result = analyzer.analyze_full(source, false, true).unwrap();
        let tokens = &result[0].clone_tokens;
        assert!(
            tokens.contains(&"filter".to_string()),
            "expected 'filter' preserved verbatim in {:?}",
            tokens
        );
        assert!(
            tokens.contains(&"map".to_string()),
            "expected chained method call target 'map' preserved verbatim in {:?}",
            tokens
        );
    }

    #[test]
    fn test_clone_tokens_two_renamed_functions_produce_equal_sequences() {
        let a = "function foo() { let x = 5; bar(x); }";
        let b = "function renamed() { let y = 5; bar(y); }";
        let analyzer = JavaScriptAnalyzer;
        let ra = analyzer.analyze_full(a, false, true).unwrap();
        let rb = analyzer.analyze_full(b, false, true).unwrap();
        assert_eq!(ra[0].clone_tokens, rb[0].clone_tokens);
    }
}
