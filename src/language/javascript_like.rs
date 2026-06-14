use tree_sitter::Node;

pub const FUNCTION_KINDS: &[&str] = &[
    "function_declaration",
    "function_expression",
    "arrow_function",
    "method_definition",
];
pub const CLOSURE_KINDS: &[&str] = &["function_expression", "arrow_function"];
pub const DECISION_KINDS: &[&str] = &[
    "if_statement",
    "for_statement",
    "while_statement",
    "do_statement",
    "catch_clause",
    "ternary_expression",
    "switch_case",
    "switch_default",
];
pub const OPERATOR_KINDS: &[&str] = &[
    "+",
    "-",
    "*",
    "/",
    "%",
    "**",
    "==",
    "!=",
    "===",
    "!==",
    "<",
    ">",
    "<=",
    ">=",
    "&&",
    "||",
    "!",
    "??",
    "?.",
    "=",
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "**=",
    "&",
    "|",
    "^",
    "<<",
    ">>",
    ">>>",
    "~",
    "++",
    "--",
    ".",
    ":",
    "=>",
    "return_statement",
    "yield",
    "await",
];
pub const OPERAND_KINDS: &[&str] = &[
    "identifier",
    "number",
    "string",
    "true",
    "false",
    "null",
    "undefined",
];

pub fn extract_name(node: Node, source: &str) -> String {
    if node.kind() == "arrow_function" || node.kind() == "function_expression" {
        return format!("<closure>@line {}", node.start_position().row + 1);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "property_identifier" {
            return source[child.start_byte()..child.end_byte()].to_string();
        }
    }
    format!("<anon>@line {}", node.start_position().row + 1)
}
