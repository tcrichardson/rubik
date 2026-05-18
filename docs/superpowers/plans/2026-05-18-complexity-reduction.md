# Complexity Reduction — High-Priority Refactors Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce cyclomatic complexity of two match-heavy dispatch functions in `src/output/markdown.rs` and eliminate the duplicated `parser()` body across five language files.

**Architecture:** Two independent refactors. (1) Replace `file_summary_row` and `project_summary_row` match arms with static function-pointer lookup tables in `markdown.rs`. (2) Extract a `make_parser` free function in `language/mod.rs` and delegate to it from each language's `parser()` method.

**Tech Stack:** Rust, tree-sitter

---

## Baseline

Before starting any task, confirm the test suite passes:

```
cargo test
```

Expected: `13 passed; 0 failed`

---

## Task 1: Extract `make_parser` helper in `src/language/mod.rs`

**Files:**
- Modify: `src/language/mod.rs`

The five language `parser()` methods all follow this pattern (only the grammar constant differs):

```rust
fn parser(&self) -> Result<Parser, String> {
    let mut parser = Parser::new();
    let language: tree_sitter::Language = tree_sitter_LANG::LANGUAGE.into();
    parser.set_language(&language).map_err(|e| format!("{e:?}"))?;
    Ok(parser)
}
```

- [ ] **Step 1: Add `make_parser` to `src/language/mod.rs`**

Add the following function directly after the `count_decisions` function (after line 135, before `pub mod c;`):

```rust
/// Creates a tree-sitter Parser configured for the given language grammar.
/// Shared by all language analyzers to avoid duplicating error-handling boilerplate.
pub fn make_parser(language: tree_sitter::Language) -> Result<Parser, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| format!("{e:?}"))?;
    Ok(parser)
}
```

- [ ] **Step 2: Verify it compiles**

```
cargo build 2>&1
```

Expected: no errors. (No callers yet, so `dead_code` warning is fine.)

---

## Task 2: Update `src/language/rust.rs` to use `make_parser`

**Files:**
- Modify: `src/language/rust.rs`

- [ ] **Step 1: Replace `parser()` body**

Find (lines 40–45):
```rust
    fn parser(&self) -> Result<Parser, String> {
        let mut parser = Parser::new();
        let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
        parser.set_language(&language).map_err(|e| format!("{e:?}"))?;
        Ok(parser)
    }
```

Replace with:
```rust
    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_rust::LANGUAGE.into())
    }
```

- [ ] **Step 2: Remove unused import**

The `tree_sitter::Parser` import at line 2 is now only used indirectly via `make_parser`. Check if `Parser` appears anywhere else in the file besides the old `parser()` body:

```
cargo build 2>&1 | grep "rust.rs"
```

If `unused import: \`Parser\`` appears, remove `Parser` from the import line at the top of `rust.rs`. The import should become:

```rust
use tree_sitter::Node;
```

- [ ] **Step 3: Run tests**

```
cargo test
```

Expected: `13 passed; 0 failed`

---

## Task 3: Update `src/language/c.rs` to use `make_parser`

**Files:**
- Modify: `src/language/c.rs`

- [ ] **Step 1: Replace `parser()` body**

Find (lines 37–42):
```rust
    fn parser(&self) -> Result<Parser, String> {
        let mut parser = Parser::new();
        let language: tree_sitter::Language = tree_sitter_c::LANGUAGE.into();
        parser.set_language(&language).map_err(|e| format!("{e:?}"))?;
        Ok(parser)
    }
```

Replace with:
```rust
    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_c::LANGUAGE.into())
    }
```

- [ ] **Step 2: Remove unused import if needed**

```
cargo build 2>&1 | grep "c.rs"
```

If `Parser` is unused, update the import at line 2:

```rust
use tree_sitter::Node;
```

- [ ] **Step 3: Run tests**

```
cargo test
```

Expected: `13 passed; 0 failed`

---

## Task 4: Update `src/language/python.rs` to use `make_parser`

**Files:**
- Modify: `src/language/python.rs`

- [ ] **Step 1: Replace `parser()` body**

Find (lines 38–43):
```rust
    fn parser(&self) -> Result<Parser, String> {
        let mut parser = Parser::new();
        let language: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
        parser.set_language(&language).map_err(|e| format!("{e:?}"))?;
        Ok(parser)
    }
```

Replace with:
```rust
    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_python::LANGUAGE.into())
    }
```

- [ ] **Step 2: Remove unused import if needed**

```
cargo build 2>&1 | grep "python.rs"
```

If `Parser` is unused, update the import at line 2:

```rust
use tree_sitter::Node;
```

- [ ] **Step 3: Run tests**

```
cargo test
```

Expected: `13 passed; 0 failed`

---

## Task 5: Update `src/language/javascript.rs` to use `make_parser`

**Files:**
- Modify: `src/language/javascript.rs`

- [ ] **Step 1: Replace `parser()` body**

Find (lines 18–23):
```rust
    fn parser(&self) -> Result<Parser, String> {
        let mut parser = Parser::new();
        let language: tree_sitter::Language = tree_sitter_javascript::LANGUAGE.into();
        parser.set_language(&language).map_err(|e| format!("{e:?}"))?;
        Ok(parser)
    }
```

Replace with:
```rust
    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_javascript::LANGUAGE.into())
    }
```

- [ ] **Step 2: Remove unused import**

The file currently imports `use tree_sitter::Parser;` (line 5). With `make_parser`, this is no longer needed directly. Check:

```
cargo build 2>&1 | grep "javascript.rs"
```

If unused, remove `use tree_sitter::Parser;` from the import section. The remaining imports are:

```rust
use crate::language::javascript_like::{
    extract_name, CLOSURE_KINDS, DECISION_KINDS, FUNCTION_KINDS, OPERAND_KINDS, OPERATOR_KINDS,
};
use crate::language::{LanguageAnalyzer, LanguageConfig};
```

- [ ] **Step 3: Run tests**

```
cargo test
```

Expected: `13 passed; 0 failed`

---

## Task 6: Update `src/language/typescript.rs` to use `make_parser`

**Files:**
- Modify: `src/language/typescript.rs`

Note: TypeScript uses `tree_sitter_typescript::LANGUAGE_TSX` (not `LANGUAGE`).

- [ ] **Step 1: Replace `parser()` body**

Find (lines 19–24):
```rust
    fn parser(&self) -> Result<Parser, String> {
        let mut parser = Parser::new();
        let language: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TSX.into();
        parser.set_language(&language).map_err(|e| format!("{e:?}"))?;
        Ok(parser)
    }
```

Replace with:
```rust
    fn parser(&self) -> Result<Parser, String> {
        crate::language::make_parser(tree_sitter_typescript::LANGUAGE_TSX.into())
    }
```

- [ ] **Step 2: Remove unused import**

The file imports `use tree_sitter::Parser;` (line 6). Check:

```
cargo build 2>&1 | grep "typescript.rs"
```

If unused, remove `use tree_sitter::Parser;`. The remaining imports should be:

```rust
use crate::language::javascript_like::{
    extract_name, CLOSURE_KINDS, DECISION_KINDS, FUNCTION_KINDS, OPERAND_KINDS, OPERATOR_KINDS,
};
use crate::language::{LanguageAnalyzer, LanguageConfig};
use std::path::Path;
```

- [ ] **Step 3: Run tests**

```
cargo test
```

Expected: `13 passed; 0 failed`

- [ ] **Step 4: Commit all parser dedup changes**

```bash
git add src/language/mod.rs src/language/rust.rs src/language/c.rs src/language/python.rs src/language/javascript.rs src/language/typescript.rs
git commit -m "refactor: extract make_parser helper to eliminate duplicated parser() bodies"
```

---

## Task 7: Replace `project_summary_row` match with static lookup table

**Files:**
- Modify: `src/output/markdown.rs`

Current `project_summary_row` (lines 157–175) is a 13-arm match. Replace with a static array of `(&str, fn(&SummaryStatistics) -> String)` pairs.

- [ ] **Step 1: Add type alias and static table before `project_summary_row`**

Replace the entire `project_summary_row` function (lines 157–175) with:

```rust
type ProjectFmt = fn(&SummaryStatistics) -> String;

static PROJECT_SUMMARY_FORMATTERS: &[(&str, ProjectFmt)] = &[
    ("files_analyzed",              |s| metric_row("Files Analyzed", s.files_analyzed)),
    ("total_functions",             |s| metric_row("Total Functions", s.total_functions)),
    ("total_lines",                 |s| metric_row("Total Lines", s.total_lines)),
    ("total_complexity",            |s| metric_row("Total Complexity", s.total_complexity)),
    ("avg_complexity_per_function", |s| metric_row_f64("Avg Complexity / Function", s.avg_complexity_per_function, 2)),
    ("max_nesting_depth",           |s| metric_row("Max Nesting Depth", s.max_nesting_depth)),
    ("avg_nesting_depth",           |s| metric_row_f64("Avg Nesting Depth", s.avg_nesting_depth, 2)),
    ("avg_halstead_volume",         |s| metric_row_f64("Avg Halstead Volume", s.avg_halstead_volume, 2)),
    ("avg_halstead_difficulty",     |s| metric_row_f64("Avg Halstead Difficulty", s.avg_halstead_difficulty, 2)),
    ("avg_halstead_effort",         |s| metric_row_f64("Avg Halstead Effort", s.avg_halstead_effort, 2)),
    ("avg_halstead_time",           |s| metric_row_f64("Avg Halstead Time", s.avg_halstead_time, 2)),
];

fn project_summary_row(key: &str, summary: &SummaryStatistics) -> Option<String> {
    PROJECT_SUMMARY_FORMATTERS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, fmt)| fmt(summary))
        .or_else(|| {
            eprintln!("Warning: unknown metric key '{}', ignoring", key);
            None
        })
}
```

- [ ] **Step 2: Run tests**

```
cargo test
```

Expected: `13 passed; 0 failed`. The `test_config_metric_filtering_markdown`, `test_config_unknown_metric_key_warns_and_succeeds`, and `test_rust_fixture_pretty` tests all exercise this path.

---

## Task 8: Replace `file_summary_row` match with static lookup table

**Files:**
- Modify: `src/output/markdown.rs`

Current `file_summary_row` (lines 177–201) is a 19-arm match. Same pattern as Task 7.

- [ ] **Step 1: Add type alias and static table before `file_summary_row`**

Replace the entire `file_summary_row` function (lines 177–201) with:

```rust
type FileFmt = fn(&FileResult, f64) -> String;

static FILE_SUMMARY_FORMATTERS: &[(&str, FileFmt)] = &[
    ("total_functions",             |f, _| metric_row("Total Functions", f.function_count)),
    ("total_lines",                 |f, _| metric_row("Total Lines", f.total_lines)),
    ("total_function_lines",        |f, _| metric_row("Total Function Lines", f.total_function_lines)),
    ("total_complexity",            |f, _| metric_row("Total Complexity", f.total_complexity)),
    ("avg_complexity_per_function", |_, a| metric_row_f64("Avg Complexity / Function", a, 2)),
    ("max_complexity",              |f, _| metric_row("Max Complexity", f.max_complexity)),
    ("max_nesting_depth",           |f, _| metric_row("Max Nesting Depth", f.max_nesting_depth)),
    ("avg_nesting_depth",           |f, _| metric_row_f64("Avg Nesting Depth", f.avg_nesting_depth, 2)),
    ("max_function_lines",          |f, _| metric_row("Max Function Lines", f.max_function_lines)),
    ("avg_halstead_volume",         |f, _| metric_row_f64("Avg Halstead Volume", f.avg_halstead_volume, 2)),
    ("max_halstead_volume",         |f, _| metric_row_f64("Max Halstead Volume", f.max_halstead_volume, 2)),
    ("avg_halstead_difficulty",     |f, _| metric_row_f64("Avg Halstead Difficulty", f.avg_halstead_difficulty, 2)),
    ("max_halstead_difficulty",     |f, _| metric_row_f64("Max Halstead Difficulty", f.max_halstead_difficulty, 2)),
    ("avg_halstead_effort",         |f, _| metric_row_f64("Avg Halstead Effort", f.avg_halstead_effort, 2)),
    ("max_halstead_effort",         |f, _| metric_row_f64("Max Halstead Effort", f.max_halstead_effort, 2)),
    ("avg_halstead_time",           |f, _| metric_row_f64("Avg Halstead Time", f.avg_halstead_time, 2)),
    ("max_halstead_time",           |f, _| metric_row_f64("Max Halstead Time", f.max_halstead_time, 2)),
];

fn file_summary_row(key: &str, file: &FileResult, avg_complexity: f64) -> Option<String> {
    FILE_SUMMARY_FORMATTERS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, fmt)| fmt(file, avg_complexity))
        .or_else(|| {
            eprintln!("Warning: unknown metric key '{}', ignoring", key);
            None
        })
}
```

- [ ] **Step 2: Run tests**

```
cargo test
```

Expected: `13 passed; 0 failed`.

- [ ] **Step 3: Commit markdown refactor**

```bash
git add src/output/markdown.rs
git commit -m "refactor: replace file_summary_row and project_summary_row match arms with static lookup tables"
```

---

## Verification

After all tasks complete:

```
cargo test
```

Expected: `13 passed; 0 failed`

```
cargo build --release
```

Expected: clean build, no warnings about new code.
