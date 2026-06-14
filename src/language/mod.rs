use crate::FunctionComplexity;
use std::path::Path;
use tree_sitter::{Node, Parser};

/// Stroud's number: estimated elementary mental discriminations per second.
const STROUDS_NUMBER: f64 = 18.0;

pub struct LanguageConfig {
    pub function_kinds: &'static [&'static str],
    pub closure_kinds: &'static [&'static str],
    pub decision_kinds: &'static [&'static str],
    pub operator_kinds: &'static [&'static str],
    pub operand_kinds: &'static [&'static str],
    pub extract_name: fn(Node, &str) -> String,
    /// Pairs of (container_kind, child_kind) where each matching child
    /// counts as one decision (e.g., Python match/case).
    pub match_case_kinds: &'static [(&'static str, &'static str)],
    /// If true, nodes with no children are skipped (e.g., bare lambda tokens in Python).
    pub skip_childless_nodes: bool,
}

pub trait LanguageAnalyzer: Send + Sync {
    fn can_analyze(&self, path: &Path) -> bool;
    fn config(&self) -> LanguageConfig;
    fn parser(&self) -> Result<Parser, String>;
    fn language_name(&self) -> &'static str {
        "source"
    }
    fn analyze(
        &self,
        source: &str,
        include_closures: bool,
    ) -> Result<Vec<FunctionComplexity>, String> {
        let mut parser = self.parser()?;
        let config = self.config();
        let msg = format!("Failed to parse {} source", self.language_name());
        let tree = parser.parse(source, None).ok_or_else(|| msg.clone())?;
        if tree.root_node().has_error() {
            return Err(msg);
        }
        let mut functions = Vec::new();
        collect_functions(
            tree.root_node(),
            source,
            &mut functions,
            &config,
            include_closures,
        );
        Ok(functions)
    }
}

/// Generic function collector. Each language analyzer calls this with its own configuration.
pub fn collect_functions(
    node: Node,
    source: &str,
    functions: &mut Vec<FunctionComplexity>,
    config: &LanguageConfig,
    include_closures: bool,
) {
    if is_target_function(node, config, include_closures) {
        functions.push(build_function_complexity(node, source, config));
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_functions(child, source, functions, config, include_closures);
    }
}

fn is_target_function(node: Node, config: &LanguageConfig, include_closures: bool) -> bool {
    if !config.function_kinds.contains(&node.kind()) {
        return false;
    }
    if config.skip_childless_nodes && node.child_count() == 0 {
        return false;
    }
    include_closures || !config.closure_kinds.contains(&node.kind())
}

fn build_function_complexity(
    node: Node,
    source: &str,
    config: &LanguageConfig,
) -> FunctionComplexity {
    let name = (config.extract_name)(node, source);
    let complexity = 1 + count_decisions(
        node,
        source,
        config.decision_kinds,
        config.function_kinds,
        config.match_case_kinds,
    );
    let nesting_depth =
        crate::cognitive::max_nesting_depth(node, config.decision_kinds, config.function_kinds);
    let (halstead_volume, halstead_difficulty) = crate::cognitive::halstead_metrics(
        node,
        source,
        config.operator_kinds,
        config.operand_kinds,
        config.function_kinds,
    );
    let halstead_effort = halstead_volume * halstead_difficulty;
    let halstead_time = halstead_effort / STROUDS_NUMBER;
    FunctionComplexity {
        name,
        line_start: node.start_position().row + 1,
        line_end: node.end_position().row + 1,
        lines: node.end_position().row - node.start_position().row + 1,
        complexity,
        nesting_depth,
        halstead_volume,
        halstead_difficulty,
        halstead_effort,
        halstead_time,
    }
}

pub fn count_decisions(
    node: Node,
    source: &str,
    decision_kinds: &[&str],
    function_kinds: &[&str],
    match_case_kinds: &[(&str, &str)],
) -> u32 {
    let mut count = 0;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if function_kinds.contains(&child.kind()) {
            continue;
        }
        if decision_kinds.contains(&child.kind()) {
            count += 1;
        }
        if crate::complexity::is_boolean_operator(child, source) {
            count += 1;
        }
        for (container, case_kind) in match_case_kinds {
            if child.kind() == *container {
                count += crate::complexity::count_descendants_of_kind(
                    child,
                    &[case_kind],
                    function_kinds,
                );
            }
        }
        count += count_decisions(
            child,
            source,
            decision_kinds,
            function_kinds,
            match_case_kinds,
        );
    }
    count
}

/// Creates a tree-sitter Parser configured for the given language grammar.
/// Shared by all language analyzers to avoid duplicating error-handling boilerplate.
pub fn make_parser(language: tree_sitter::Language) -> Result<Parser, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| format!("{e:?}"))?;
    Ok(parser)
}

pub mod c;
pub mod java;
pub mod javascript;
mod javascript_like;
pub mod python;
pub mod rust;
pub mod typescript;
