use crate::language::javascript_like::{
    extract_name, CLOSURE_KINDS, DECISION_KINDS, FUNCTION_KINDS, OPERAND_KINDS, OPERATOR_KINDS,
};
use crate::language::{LanguageAnalyzer, LanguageConfig};
use std::path::Path;
use tree_sitter::Parser;

pub struct TypeScriptAnalyzer;

impl LanguageAnalyzer for TypeScriptAnalyzer {
    fn can_analyze(&self, path: &Path) -> bool {
        path.extension().map_or(false, |e| e == "ts" || e == "tsx")
    }

    fn language_name(&self) -> &'static str {
        "TS"
    }

    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_typescript::LANGUAGE_TSX.into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let source = "function foo() { if (x) {} }";
        let analyzer = TypeScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_if_else() {
        let source = "function bar() { if (x) {} else if (y) {} else {} }";
        let analyzer = TypeScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + if 1 + else-if 1
    }

    #[test]
    fn test_switch() {
        let source = "function baz() { switch(x) { case 1: break; case 2: break; default: break; } }";
        let analyzer = TypeScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4); // base 1 + 3 cases
    }

    #[test]
    fn test_arrow_function_included() {
        let source = "const f = (x) => x > 0 ? 1 : 0;";
        let analyzer = TypeScriptAnalyzer;
        let result = analyzer.analyze(source, true).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].name.starts_with("<closure>"));
        assert_eq!(result[0].complexity, 2); // base 1 + ternary 1
    }

    #[test]
    fn test_arrow_function_excluded_by_default() {
        let source = "const f = (x) => x > 0 ? 1 : 0;";
        let analyzer = TypeScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_try_catch() {
        let source = "function err() { try {} catch (a) { if (b) {} } }";
        let analyzer = TypeScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + catch 1 + if 1
    }

    #[test]
    fn test_boolean_ops() {
        let source = "function b() { return a && b || c; }";
        let analyzer = TypeScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3); // base 1 + && 1 + || 1
    }

    #[test]
    fn test_type_annotations_ignored() {
        let source = r#"function greet(name: string): string {
            if (name) {
                return "hello " + name;
            }
            return "hello";
        }"#;
        let analyzer = TypeScriptAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "greet");
        assert_eq!(result[0].complexity, 2); // base 1 + if 1
    }
}
