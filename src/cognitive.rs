use std::collections::HashSet;
use tree_sitter::Node;

pub fn max_nesting_depth(node: Node, decision_kinds: &[&str], function_kinds: &[&str]) -> u32 {
    compute_nesting_depth(node, decision_kinds, function_kinds, 0)
}

fn compute_nesting_depth(
    node: Node,
    decision_kinds: &[&str],
    function_kinds: &[&str],
    current_depth: u32,
) -> u32 {
    let mut max_depth = current_depth;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if function_kinds.contains(&child.kind()) {
            continue;
        }
        let is_decision = decision_kinds.contains(&child.kind());
        let next_depth = if is_decision {
            current_depth + 1
        } else {
            current_depth
        };
        let child_max = compute_nesting_depth(child, decision_kinds, function_kinds, next_depth);
        if child_max > max_depth {
            max_depth = child_max;
        }
    }
    max_depth
}

struct HalsteadState {
    operators_distinct: HashSet<String>,
    operators_total: u32,
    operands_distinct: HashSet<String>,
    operands_total: u32,
}

impl HalsteadState {
    fn new() -> Self {
        Self {
            operators_distinct: HashSet::new(),
            operators_total: 0,
            operands_distinct: HashSet::new(),
            operands_total: 0,
        }
    }

    fn add_operator(&mut self, kind: &str) {
        self.operators_distinct.insert(kind.to_string());
        self.operators_total += 1;
    }

    fn add_operand(&mut self, text: &str) {
        self.operands_distinct.insert(text.to_string());
        self.operands_total += 1;
    }

    fn n1(&self) -> usize {
        self.operators_distinct.len()
    }

    fn n2(&self) -> usize {
        self.operands_distinct.len()
    }

    fn n(&self) -> usize {
        self.n1() + self.n2()
    }

    fn total(&self) -> u32 {
        self.operators_total + self.operands_total
    }
}

pub fn halstead_metrics(
    node: Node,
    source: &str,
    operator_kinds: &[&str],
    operand_kinds: &[&str],
    function_kinds: &[&str],
) -> (f64, f64) {
    let mut state = HalsteadState::new();
    collect_halstead(
        node,
        source,
        operator_kinds,
        operand_kinds,
        function_kinds,
        &mut state,
    );

    let n = state.n();
    let n1 = state.n1();
    let n2 = state.n2();
    let n_total = state.total();

    let volume = if n == 0 {
        0.0
    } else {
        (n_total as f64) * ((n as f64).log2())
    };

    let difficulty = if n2 == 0 {
        0.0
    } else {
        ((n1 as f64) / 2.0) * ((state.operands_total as f64) / (n2 as f64))
    };

    (volume, difficulty)
}

fn collect_halstead(
    node: Node,
    source: &str,
    operator_kinds: &[&str],
    operand_kinds: &[&str],
    function_kinds: &[&str],
    state: &mut HalsteadState,
) {
    let kind = node.kind();
    if operator_kinds.contains(&kind) {
        state.add_operator(kind);
    } else if operand_kinds.contains(&kind) {
        let text = &source[node.start_byte()..node.end_byte()];
        state.add_operand(text);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if function_kinds.contains(&child.kind()) {
            continue;
        }
        collect_halstead(
            child,
            source,
            operator_kinds,
            operand_kinds,
            function_kinds,
            state,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_rust(source: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
        parser.set_language(&language).unwrap();
        parser.parse(source, None).unwrap()
    }

    fn find_function<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        if node.kind() == kind {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_function(child, kind) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_nesting_depth_simple() {
        let source = "fn f() { if true {} }";
        let tree = parse_rust(source);
        let root = tree.root_node();
        let func = find_function(root, "function_item").expect("function not found");
        let depth = max_nesting_depth(func, &["if_expression"], &["function_item"]);
        assert_eq!(depth, 1);
    }

    #[test]
    fn test_nesting_depth_nested() {
        let source = "fn f() { if true { for x in y {} } }";
        let tree = parse_rust(source);
        let root = tree.root_node();
        let func = find_function(root, "function_item").expect("function not found");
        let depth = max_nesting_depth(
            func,
            &["if_expression", "for_expression"],
            &["function_item"],
        );
        assert_eq!(depth, 2);
    }

    #[test]
    fn test_nesting_depth_skips_inner_function() {
        let source = "fn f() { if true { fn g() { if true {} } } }";
        let tree = parse_rust(source);
        let root = tree.root_node();
        let func = find_function(root, "function_item").expect("function not found");
        let depth = max_nesting_depth(func, &["if_expression"], &["function_item"]);
        assert_eq!(depth, 1);
    }

    #[test]
    fn test_halstead_basic() {
        let source = "fn f() { let x = 1 + 2; }";
        let tree = parse_rust(source);
        let root = tree.root_node();
        let func = find_function(root, "function_item").expect("function not found");
        let (volume, difficulty) = halstead_metrics(
            func,
            source,
            &["+", "let_declaration"],
            &["identifier", "integer_literal"],
            &["function_item"],
        );
        assert!(volume > 0.0);
        assert!(difficulty >= 0.0);
    }
}
