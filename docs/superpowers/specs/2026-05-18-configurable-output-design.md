# Configurable Output Design

**Date:** 2026-05-18  
**Status:** Approved  
**Feature:** Report configuration via `lede.toml` — metric-level selection for project and file summaries, plus a configurable introduction

---

## Overview

lede gains the ability to read a `lede.toml` file from the analyzed directory. This file controls:

1. **Introduction text** — optional prose prepended to the output report
2. **Project summary metrics** — which of the project-level aggregate metrics to display
3. **File summary metrics** — which of the per-file aggregate metrics to display

If `lede.toml` is absent, or if a section is omitted from it, lede behaves identically to today — all metrics shown, no introduction.

---

## Config File Format

File name: `lede.toml`  
Location: the directory being analyzed (or the parent directory when analyzing a single file)

```toml
# Optional introduction text prepended to the report
introduction = """
This report analyzes the complexity and maintainability of the project.
All metrics are computed per-function and aggregated per-file and project-wide.
"""

[project_summary]
# Ordered list of metric keys to display in the project-level summary.
# If this section is absent, all metrics are shown.
metrics = [
  "files_analyzed",
  "total_functions",
  "total_complexity",
  "avg_complexity_per_function",
  "max_nesting_depth",
]

[file_summary]
# Ordered list of metric keys to display in per-file summary tables.
# If this section is absent, all metrics are shown.
metrics = [
  "total_functions",
  "total_complexity",
  "avg_complexity_per_function",
  "max_complexity",
  "max_nesting_depth",
]
```

### Available Project-Level Metric Keys

| Key | Description |
|-----|-------------|
| `files_analyzed` | Number of files successfully analyzed |
| `total_functions` | Total function count across all files |
| `total_lines` | Total source lines across all files |
| `total_complexity` | Sum of cyclomatic complexity across all functions |
| `avg_complexity_per_function` | Average cyclomatic complexity per function |
| `max_nesting_depth` | Maximum nesting depth found in any function |
| `avg_nesting_depth` | Average nesting depth across all functions |
| `avg_halstead_volume` | Average Halstead volume across all functions |
| `avg_halstead_difficulty` | Average Halstead difficulty across all functions |
| `avg_halstead_effort` | Average Halstead effort across all functions |
| `avg_halstead_time` | Average estimated implementation time across all functions |

### Available File-Level Metric Keys

| Key | Description |
|-----|-------------|
| `total_functions` | Number of functions in this file |
| `total_lines` | Total lines in this file |
| `total_function_lines` | Sum of lines across all functions |
| `total_complexity` | Sum of cyclomatic complexity for this file |
| `avg_complexity_per_function` | Average cyclomatic complexity per function |
| `max_complexity` | Highest cyclomatic complexity in any function |
| `max_nesting_depth` | Deepest nesting found in any function |
| `avg_nesting_depth` | Average nesting depth across functions in this file |
| `max_function_lines` | Length of the longest function |
| `avg_halstead_volume` | Average Halstead volume |
| `max_halstead_volume` | Maximum Halstead volume |
| `avg_halstead_difficulty` | Average Halstead difficulty |
| `max_halstead_difficulty` | Maximum Halstead difficulty |
| `avg_halstead_effort` | Average Halstead effort |
| `max_halstead_effort` | Maximum Halstead effort |
| `avg_halstead_time` | Average estimated implementation time |
| `max_halstead_time` | Maximum estimated implementation time |

---

## Architecture

### New module: `src/config.rs`

```rust
pub struct ReportConfig {
    pub introduction: Option<String>,
    pub project_summary_metrics: Option<Vec<String>>,  // None = show all
    pub file_summary_metrics: Option<Vec<String>>,     // None = show all
}

impl ReportConfig {
    /// Reads lede.toml from `dir`. Returns default config if file is absent
    /// or cannot be parsed (error emitted to stderr).
    pub fn load_from_dir(dir: &Path) -> Self { ... }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            introduction: None,
            project_summary_metrics: None,
            file_summary_metrics: None,
        }
    }
}
```

TOML deserialization uses a private `RawConfig` struct via the `toml` crate (added as an explicit dependency).

### Updated Formatter Structs (Approach A)

Each formatter struct gains a `config: ReportConfig` field:

```rust
pub struct MarkdownFormatter { pub config: ReportConfig }
pub struct PrettyFormatter   { pub config: ReportConfig }
pub struct JsonFormatter     { pub config: ReportConfig }
```

The `OutputFormatter` trait signature is unchanged:

```rust
pub trait OutputFormatter {
    fn format(&self, results: &[FileResult], clusters: &[DuplicateCluster]) -> String;
}
```

`get_formatter()` is updated to accept a config:

```rust
pub fn get_formatter(format: &str, config: ReportConfig) -> Box<dyn OutputFormatter>
```

### `main.rs` Changes

1. Resolve the target directory from `args.path` (use `path` itself if it's a directory; its parent if it's a file)
2. Call `ReportConfig::load_from_dir(&dir)` to get config
3. Pass config into `get_formatter()`

### Formatter Rendering Behavior

**Introduction (all three formatters):**
- Markdown/Pretty: if `config.introduction` is `Some(text)`, emit the text followed by a blank line before the first section
- JSON: if `config.introduction` is `Some(text)`, add an `"introduction"` key at the top level of the JSON output object

**Project summary:**
- If `config.project_summary_metrics` is `None` → render all metrics (current behavior)
- If `Some(list)` → render only the metrics whose keys appear in `list`, in the order they appear in `list`; unknown keys are silently skipped after a stderr warning

**File summary:**
- Same logic as project summary but using `config.file_summary_metrics`

The per-function detail table and duplicate cluster table are not affected by this config.

---

## Edge Cases

| Situation | Behavior |
|-----------|----------|
| `lede.toml` not found | `ReportConfig::default()` — all metrics shown, no introduction |
| `[project_summary]` or `[file_summary]` absent from file | That section shows all metrics |
| `metrics = []` (empty list) | Section header is shown but zero metric rows are rendered |
| Unknown metric key in list | Warning to stderr: `"Unknown metric key: <key>, ignoring"` — key skipped |
| `args.path` is a single file | Look for `lede.toml` in the file's parent directory |
| Malformed TOML | Error to stderr: `"Warning: could not parse lede.toml: <error> — using defaults"` — analysis continues with default config |

---

## Tests

### Unit tests (in `src/config.rs`)

1. `load_from_dir` with a temp dir containing valid `lede.toml` → fields populated correctly
2. `load_from_dir` with no file → all fields `None`
3. `load_from_dir` with malformed TOML → all fields `None`, no panic

### Integration tests (in `tests/`)

4. Run lede on a fixture dir with a `lede.toml` that sets `project_summary.metrics` to two keys → verify only those two rows appear in markdown output
5. Run lede on a fixture dir with a `lede.toml` containing `introduction` → verify text appears at the top of the output
6. Run lede on a fixture dir with a `lede.toml` containing an unknown metric key → analysis succeeds, warning on stderr, unknown key absent from output

---

## Dependency

The `toml` crate is already a transitive dependency; it must be added as a direct, explicit dependency in `Cargo.toml`:

```toml
[dependencies]
toml = { version = "0.8", features = ["parse"] }
```

---

## Non-Goals

- No CLI flag overrides (config file is the only mechanism)
- No per-function table column selection
- No section-level on/off toggles (use an empty `metrics = []` as a workaround to suppress a summary section's rows)
- No template placeholders in the introduction text
