# Java Language Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Java language support to lede, enabling analysis of `.java` source files for cyclomatic complexity, nesting depth, and Halstead metrics.

**Architecture:** Create a `JavaAnalyzer` struct implementing the existing `LanguageAnalyzer` trait, following the exact pattern of `rust.rs`, `python.rs`, and `c.rs`. Register it in the analyzer dispatcher.

**Tech Stack:** Rust, tree-sitter-java 0.23, existing lede framework

---

### Task 1: Add tree-sitter-java dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add the dependency to Cargo.toml**

Add `tree-sitter-java = "0.23"` to the `[dependencies]` section, after the existing tree-sitter entries:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
comfy-table = "7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tree-sitter = "0.24"
tree-sitter-c = "0.23"
tree-sitter-java = "0.23"
tree-sitter-javascript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-rust = "0.23"
tree-sitter-typescript = "0.23"
walkdir = "2"
```

- [ ] **Step 2: Verify the dependency resolves**

Run: `cargo check`
Expected: Dependencies download/compile, no errors (compilation warnings about unused imports are fine at this stage)

---

### Task 2: Create the Java analyzer module

**Files:**
- Create: `src/language/java.rs`

- [ ] **Step 1: Create `src/language/java.rs` with the full analyzer implementation**

```rust
use crate::language::{LanguageAnalyzer, LanguageConfig};
use tree_sitter::{Node, Parser};

pub struct JavaAnalyzer;

const FUNCTION_KINDS: &[&str] = &[
    "method_declaration",
    "constructor_declaration",
    "compact_constructor_declaration",
    "lambda_expression",
];
const CLOSURE_KINDS: &[&str] = &["lambda_expression"];
const DECISION_KINDS: &[&str] = &[
    "if_statement",
    "for_statement",
    "enhanced_for_statement",
    "while_statement",
    "do_statement",
    "switch_label",
    "catch_clause",
    "ternary_expression",
];
const OPERATOR_KINDS: &[&str] = &[
    "+", "-", "*", "/", "%",
    "==", "!=", "<", ">", "<=", ">=",
    "&&", "||", "!",
    "=", "+=", "-=", "*=", "/=", "%=",
    "&", "|", "^", "<<", ">>", ">>>", "~",
    ".", "::",
    "return_statement", "break_statement", "continue_statement",
    "throw_statement", "yield_statement",
    "instanceof_expression",
];
const OPERAND_KINDS: &[&str] = &[
    "identifier",
    "decimal_integer_literal",
    "hex_integer_literal",
    "octal_integer_literal",
    "binary_integer_literal",
    "decimal_floating_point_literal",
    "hex_floating_point_literal",
    "string_literal",
    "character_literal",
    "true",
    "false",
    "null_literal",
    "this",
    "super",
];

impl LanguageAnalyzer for JavaAnalyzer {
    fn can_analyze(&self, path: &std::path::Path) -> bool {
        path.extension().map_or(false, |e| e == "java")
    }

    fn language_name(&self) -> &'static str {
        "Java"
    }

    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_java::LANGUAGE.into())
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

fn extract_name(node: Node, source: &str) -> String {
    if node.kind() == "lambda_expression" {
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
    fn test_simple_method() {
        let source = "void foo() { if (true) {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_if_else() {
        let source = "void bar() { if (x) {} else {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_for_loop() {
        let source = "void foo() { for (int i = 0; i < 10; i++) {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_enhanced_for_loop() {
        let source = "void foo() { for (String s : list) {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_while_and_do_while() {
        let source = "void foo() { while (true) {} do {} while (true); }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3);
    }

    #[test]
    fn test_switch_with_cases() {
        let source = "void foo() { switch (x) { case 1: break; case 2: break; default: break; } }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 4);
    }

    #[test]
    fn test_try_catch() {
        let source = "void foo() { try {} catch (Exception e) {} }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_lambda_included() {
        let source = "void foo() { Runnable r = () -> { if (true) {} }; }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, true).unwrap();
        assert_eq!(result.len(), 2);
        let lambda = result.iter().find(|f| f.name.starts_with("<lambda>")).unwrap();
        assert_eq!(lambda.complexity, 2);
    }

    #[test]
    fn test_lambda_excluded_by_default() {
        let source = "void foo() { Runnable r = () -> { if (true) {} }; }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert!(!result[0].name.starts_with("<lambda>"));
    }

    #[test]
    fn test_ternary() {
        let source = "int t() { return x > 0 ? 1 : 0; }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_boolean_ops() {
        let source = "void b() { boolean r = a && b || c; }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result[0].complexity, 3);
    }

    #[test]
    fn test_constructor_name() {
        let source = "class Foo { Foo() { if (true) {} } }";
        let analyzer = JavaAnalyzer;
        let result = analyzer.analyze(source, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Foo");
        assert_eq!(result[0].complexity, 2);
    }

    #[test]
    fn test_parse_error() {
        let source = "void foo() {";
        let analyzer = JavaAnalyzer;
        assert!(analyzer.analyze(source, false).is_err());
    }

    #[test]
    fn test_can_analyze_java_extension() {
        let analyzer = JavaAnalyzer;
        assert!(analyzer.can_analyze(std::path::Path::new("Foo.java")));
        assert!(!analyzer.can_analyze(std::path::Path::new("Foo.rs")));
    }
}
```

- [ ] **Step 2: Run the Java tests to verify they pass**

Run: `cargo test java:: -- --nocapture`
Expected: All 14 tests pass

---

### Task 3: Register the Java analyzer

**Files:**
- Modify: `src/language/mod.rs`
- Modify: `src/analyzer.rs`

- [ ] **Step 1: Add `pub mod java;` to `src/language/mod.rs`**

Add the module declaration after the existing module declarations. The file should end with:

```rust
pub mod c;
pub mod java;
pub mod javascript;
mod javascript_like;
pub mod python;
pub mod rust;
pub mod typescript;
```

- [ ] **Step 2: Import and register JavaAnalyzer in `src/analyzer.rs`**

Update the import line to include `JavaAnalyzer`:

```rust
use crate::{
    FileResult,
    language::{c::CAnalyzer, java::JavaAnalyzer, javascript::JavaScriptAnalyzer, python::PythonAnalyzer, rust::RustAnalyzer, typescript::TypeScriptAnalyzer, LanguageAnalyzer},
};
```

Add `&JavaAnalyzer` to the `ANALYZERS` array:

```rust
static ANALYZERS: &[&dyn LanguageAnalyzer] = &[
    &RustAnalyzer,
    &PythonAnalyzer,
    &JavaScriptAnalyzer,
    &TypeScriptAnalyzer,
    &CAnalyzer,
    &JavaAnalyzer,
];
```

- [ ] **Step 3: Run all tests to verify nothing is broken**

Run: `cargo test`
Expected: All tests pass (existing + 14 new Java tests)

---

### Task 4: Update README documentation

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update the Features list**

Change the first bullet from:
```
- **Multi-language support:** Rust, Python, JavaScript/JSX, TypeScript/TSX, and C
```
To:
```
- **Multi-language support:** Rust, Python, JavaScript/JSX, TypeScript/TSX, C, and Java
```

- [ ] **Step 2: Update the Supported File Extensions table**

Change the table from:
```
| Language | Extensions |
|---|---|
| Rust | `.rs` |
| Python | `.py` |
| JavaScript | `.js`, `.jsx` |
| TypeScript | `.ts`, `.tsx` |
| C | `.c`, `.h` |
```
To:
```
| Language | Extensions |
|---|---|
| Rust | `.rs` |
| Python | `.py` |
| JavaScript | `.js`, `.jsx` |
| TypeScript | `.ts`, `.tsx` |
| C | `.c`, `.h` |
| Java | `.java` |
```

- [ ] **Step 3: Update the Decision Point table**

Change the table header and rows to add a Java column:

```
| Decision Point | Rust | Python | JavaScript | TypeScript | C | Java |
|---|---|---|---|---|---|---|
| `if` / `elif` | `if_expression` | `if_statement`, `elif_clause` | `if_statement` | `if_statement` | `if_statement` | `if_statement` |
| `match` / `switch` / `case` | `match_expression` (per arm) | `match_statement` (per case) | `switch_statement` (per case) | `switch_statement` (per case) | `case_statement` | `switch_label` |
| `for` | `for_expression` | `for_statement` | `for_statement` | `for_statement` | `for_statement` | `for_statement`, `enhanced_for_statement` |
| `while` | `while_expression` | `while_statement` | `while_statement`, `do_statement` | `while_statement`, `do_statement` | `while_statement`, `do_statement` | `while_statement`, `do_statement` |
| `loop` | `loop_expression` | — | — | — | — | — |
| `try` / `except` / `catch` | `try_expression` | `except_clause` | `catch_clause` | `catch_clause` | — | `catch_clause` |
| `&&` / `\|\|` | binary operators | `and` / `or` | binary operators | binary operators | binary operators | binary operators |
| Ternary | — | `conditional_expression` | `ternary_expression` | `ternary_expression` | `conditional_expression` | `ternary_expression` |
| Lambda / Closure* | `closure_expression` | `lambda` | `arrow_function` | `arrow_function` | — | `lambda_expression` |
```

---

### Task 5: Final verification and commit

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 2: Run cargo clippy for lint checks**

Run: `cargo clippy`
Expected: No warnings or errors (fix any that appear)

- [ ] **Step 3: Build release binary**

Run: `cargo build --release`
Expected: Successful build

- [ ] **Step 4: Test with a real Java file (optional smoke test)**

Create a test file `Test.java`:
```java
public class Test {
    public void process(int x) {
        if (x > 0) {
            for (int i = 0; i < x; i++) {
                switch (i) {
                    case 0: break;
                    case 1: break;
                    default: break;
                }
            }
        } else {
            try {
                while (true) {}
            } catch (Exception e) {}
        }
    }
}
```

Run: `cargo run -- Test.java`
Expected: Output showing the `process` method with correct complexity metrics
