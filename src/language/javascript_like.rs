use crate::language::TokenRole;
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

/// Classifies each CST node for clone-detection token normalization:
/// call/method target names are preserved verbatim, local identifiers and
/// literal values are replaced with generic placeholders, everything else
/// contributes its node kind (structural). Shared by the JavaScript and
/// TypeScript analyzers, which use identical node kinds for these
/// constructs.
pub fn classify_token(node: Node, _source: &str) -> TokenRole {
    match node.kind() {
        "identifier" => {
            if is_call_target(node) {
                TokenRole::Preserve
            } else {
                TokenRole::Normalize("<ID>")
            }
        }
        "property_identifier" => {
            if is_method_call_target(node) {
                TokenRole::Preserve
            } else {
                TokenRole::Normalize("<ID>")
            }
        }
        "number" | "string" | "true" | "false" | "null" | "undefined" => {
            TokenRole::Normalize("<LIT>")
        }
        _ => TokenRole::Structural,
    }
}

/// True when `node` is the callee identifier of a direct function call, e.g.
/// `foo` in `foo(x)`.
fn is_call_target(node: Node) -> bool {
    is_function_of_call(node)
}

/// True when `node` is the method-name identifier of a method call, e.g.
/// `method` in `obj.method(x)`. Plain property access (`obj.field` with no
/// call) is not a call target and is normalized like any other identifier.
fn is_method_call_target(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != "member_expression" || parent.child_by_field_name("property") != Some(node)
    {
        return false;
    }
    is_function_of_call(parent)
}

/// True when `node` is (possibly through an `instantiation_expression`
/// generic-call wrapper, e.g. TypeScript's `foo<T>(x)`) the `function` field
/// of a `call_expression`.
fn is_function_of_call(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        "call_expression" => parent.child_by_field_name("function") == Some(node),
        "instantiation_expression" => {
            parent.child_by_field_name("function") == Some(node) && is_function_of_call(parent)
        }
        _ => false,
    }
}
