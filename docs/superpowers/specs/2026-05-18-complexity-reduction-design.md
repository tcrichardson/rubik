# Complexity Reduction — High-Priority Refactors

**Date:** 2026-05-18
**Source:** Code complexity analysis report (`analysis.md`)

---

## Scope

Two independent high-priority refactors identified in the complexity analysis:

1. Replace long `match` dispatch tables in `src/output/markdown.rs` with static lookup tables.
2. Extract a shared `make_parser` helper in `src/language/mod.rs` to eliminate the duplicated `parser()` method body across five language files.

---

## Refactor 1: Static Lookup Tables for Markdown Dispatch Functions

### Problem

`file_summary_row` (cyclomatic complexity 19) and `project_summary_row` (complexity 13) in `src/output/markdown.rs` are string-keyed dispatch tables implemented as `match` expressions with 17 and 11 arms respectively. The complexity score is near the "very high" threshold (20), but each arm is individually trivial — the issue is mechanical repetition, not inherent logic complexity.

### Design

Replace each `match` with a static array of `(&str, fn(…) -> String)` pairs. Rust coerces non-capturing closures to function pointers, so each row can be written inline.

**`project_summary_row`** — 11 known keys, signature `fn(&SummaryStatistics) -> String`:

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

**`file_summary_row`** — 17 known keys, signature `fn(&FileResult, f64) -> String` (second arg is precomputed `avg_complexity`):

Same pattern with a `FileFmt = fn(&FileResult, f64) -> String` type alias and a `FILE_SUMMARY_FORMATTERS` static array.

### Expected Outcome

| Function | Complexity before | Complexity after |
|---|---|---|
| `file_summary_row` | 19 | ~3 |
| `project_summary_row` | 13 | ~3 |

Behavior is identical. The unknown-key warning path is preserved.

---

## Refactor 2: Extract `make_parser` Helper

### Problem

Five language files (`c.rs`, `javascript.rs`, `python.rs`, `rust.rs`, `typescript.rs`) each contain an identical 6-line `parser()` method body. The only difference is the grammar constant from the corresponding `tree_sitter_*` crate.

```rust
// Current pattern in every language file
fn parser(&self) -> Result<Parser, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_LANG::LANGUAGE.into())
        .map_err(|e| format!("Error loading {} grammar: {}", self.language_name(), e))?;
    Ok(parser)
}
```

### Design

Add a public free function in `src/language/mod.rs`:

```rust
/// Creates a tree-sitter Parser configured for the given language grammar.
pub fn make_parser(language: tree_sitter::Language, language_name: &str) -> Result<Parser, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| format!("Error loading {} grammar: {}", language_name, e))?;
    Ok(parser)
}
```

Each language's `parser()` implementation becomes a single delegating line:

```rust
fn parser(&self) -> Result<Parser, String> {
    crate::language::make_parser(tree_sitter_rust::LANGUAGE.into(), self.language_name())
}
```

### Scope Boundary

`language_name()` overrides (3 lines per file, 6 files) are left unchanged. Each returns a language-specific string literal — consolidating further would require a macro or struct-field approach and is not worth the added complexity for a 3-line function.

### Expected Outcome

- 5 × 4 lines of duplicated error-handling logic removed from language files.
- Single testable error path in `make_parser`.
- Language files become lighter and easier to scan.

---

## Files Changed

| File | Change |
|---|---|
| `src/output/markdown.rs` | Replace `file_summary_row` and `project_summary_row` match arms with static lookup tables |
| `src/language/mod.rs` | Add `make_parser` free function |
| `src/language/c.rs` | Delegate `parser()` to `make_parser` |
| `src/language/javascript.rs` | Delegate `parser()` to `make_parser` |
| `src/language/python.rs` | Delegate `parser()` to `make_parser` |
| `src/language/rust.rs` | Delegate `parser()` to `make_parser` |
| `src/language/typescript.rs` | Delegate `parser()` to `make_parser` |

---

## Testing Strategy

- Existing tests cover both refactors: language-specific `test_simple_function`, `test_if_else`, etc. verify that parsing still works correctly.
- The markdown formatter is exercised by integration tests (if present) or by manual inspection of report output.
- Run `cargo test` after each refactor to confirm no regressions.

---

## Out of Scope

- Test function duplication (`test_simple_function`, `test_boolean_ops`, etc.) — medium priority, separate task.
- `count_decisions` / `analyze_file` guard-clause flattening — medium priority, separate task.
- Halstead false positives — no action needed.
