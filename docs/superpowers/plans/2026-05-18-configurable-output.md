# Configurable Output Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow users to place a `lede.toml` in the analyzed directory to control which metrics appear in the project and file summary sections and to add a custom introduction to the report.

**Architecture:** A new `src/config.rs` module loads `lede.toml` from the analyzed directory into a `ReportConfig` struct. Each formatter struct gains a `config: ReportConfig` field; `get_formatter()` accepts a `ReportConfig` and stores it on the struct. `main.rs` resolves the config dir from the path argument and loads config before creating the formatter. Metric filtering applies to the markdown formatter's summary tables; introduction text applies to markdown, pretty, and JSON (JSON gains an `introduction` field on `AnalysisOutput`).

**Tech Stack:** Rust, `toml` crate (0.8, new explicit dependency), existing `serde`/`serde_json` for JSON, `clap` for CLI.

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `src/config.rs` | `ReportConfig` struct, TOML deserialization, `load_from_dir` |
| Modify | `Cargo.toml` | Add `toml = "0.8"` dependency |
| Modify | `src/lib.rs` | `pub mod config;`, add `introduction` field to `AnalysisOutput` |
| Modify | `src/output/mod.rs` | Add `config` field to formatter structs, update `get_formatter` signature |
| Modify | `src/output/markdown.rs` | Introduction rendering, metric-level filtering for both summary tables |
| Modify | `src/output/pretty.rs` | Introduction rendering only |
| Modify | `src/output/json.rs` | Populate `introduction` field from config |
| Modify | `src/main.rs` | Resolve config dir, load config, pass to `get_formatter` |
| Modify | `tests/integration_test.rs` | Fix `MarkdownFormatter` direct instantiation, add 3 new integration tests |
| Create | `tests/fixtures/config_filtered/rust_sample.rs` | Source fixture for metric-filtering test |
| Create | `tests/fixtures/config_filtered/lede.toml` | Config fixture that selects 2 project metrics and 2 file metrics |
| Create | `tests/fixtures/config_intro/rust_sample.rs` | Source fixture for introduction test |
| Create | `tests/fixtures/config_intro/lede.toml` | Config fixture with an `introduction` string |
| Create | `tests/fixtures/config_unknown_key/rust_sample.rs` | Source fixture for unknown-key test |
| Create | `tests/fixtures/config_unknown_key/lede.toml` | Config fixture with one valid and one unknown metric key |

---

## Task 1: Add `toml` dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add the dependency**

Open `Cargo.toml` and add `toml` to `[dependencies]`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
comfy-table = "7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tree-sitter = "0.24"
tree-sitter-c = "0.23"
tree-sitter-javascript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-rust = "0.23"
tree-sitter-typescript = "0.23"
walkdir = "2"
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build
```

Expected: Compiles successfully, `toml` appears in the dependency list.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "deps: add toml crate for config file parsing"
```

---

## Task 2: Create `src/config.rs`

**Files:**
- Create: `src/config.rs`
- Modify: `src/lib.rs` (add `pub mod config;`)

- [ ] **Step 1: Write the failing unit tests**

Create `src/config.rs` with the test module only:

```rust
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub introduction: Option<String>,
    pub project_summary_metrics: Option<Vec<String>>,
    pub file_summary_metrics: Option<Vec<String>>,
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

pub fn load_from_dir(_dir: &Path) -> ReportConfig {
    ReportConfig::default()  // stub — tests will fail
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_toml(dir: &std::path::PathBuf, content: &str) {
        fs::write(dir.join("lede.toml"), content).unwrap();
    }

    #[test]
    fn test_load_valid_config() {
        let dir = std::env::temp_dir().join("lede_test_config_valid");
        fs::create_dir_all(&dir).unwrap();
        write_toml(&dir, r#"
introduction = "Hello, this is a test report."

[project_summary]
metrics = ["files_analyzed", "total_functions"]

[file_summary]
metrics = ["total_complexity", "max_nesting_depth"]
"#);
        let cfg = load_from_dir(&dir);
        assert_eq!(cfg.introduction.as_deref(), Some("Hello, this is a test report."));
        assert_eq!(
            cfg.project_summary_metrics.as_deref(),
            Some(&["files_analyzed".to_string(), "total_functions".to_string()][..])
        );
        assert_eq!(
            cfg.file_summary_metrics.as_deref(),
            Some(&["total_complexity".to_string(), "max_nesting_depth".to_string()][..])
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_missing_file_returns_defaults() {
        let dir = std::env::temp_dir().join("lede_test_config_missing");
        fs::create_dir_all(&dir).unwrap();
        // Ensure no lede.toml exists
        let _ = fs::remove_file(dir.join("lede.toml"));
        let cfg = load_from_dir(&dir);
        assert!(cfg.introduction.is_none());
        assert!(cfg.project_summary_metrics.is_none());
        assert!(cfg.file_summary_metrics.is_none());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_malformed_toml_returns_defaults() {
        let dir = std::env::temp_dir().join("lede_test_config_malformed");
        fs::create_dir_all(&dir).unwrap();
        write_toml(&dir, "this is not valid toml }{][");
        let cfg = load_from_dir(&dir);
        assert!(cfg.introduction.is_none());
        assert!(cfg.project_summary_metrics.is_none());
        assert!(cfg.file_summary_metrics.is_none());
        fs::remove_dir_all(&dir).unwrap();
    }
}
```

- [ ] **Step 2: Add `pub mod config;` to `src/lib.rs`**

In `src/lib.rs`, add after the existing `pub mod output;` line:

```rust
pub mod config;
```

- [ ] **Step 3: Run tests to confirm they fail**

```bash
cargo test config::tests
```

Expected: 3 failures — `test_load_valid_config` fails because `load_from_dir` is a stub.

- [ ] **Step 4: Implement `load_from_dir`**

Replace the stub `load_from_dir` function in `src/config.rs` with:

```rust
use serde::Deserialize;

#[derive(Deserialize, Default)]
struct RawConfig {
    introduction: Option<String>,
    project_summary: Option<RawSection>,
    file_summary: Option<RawSection>,
}

#[derive(Deserialize)]
struct RawSection {
    metrics: Option<Vec<String>>,
}

pub fn load_from_dir(dir: &Path) -> ReportConfig {
    let toml_path = dir.join("lede.toml");
    if !toml_path.exists() {
        return ReportConfig::default();
    }
    let content = match std::fs::read_to_string(&toml_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: could not read lede.toml: {} — using defaults", e);
            return ReportConfig::default();
        }
    };
    let raw: RawConfig = match toml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Warning: could not parse lede.toml: {} — using defaults", e);
            return ReportConfig::default();
        }
    };
    ReportConfig {
        introduction: raw.introduction,
        project_summary_metrics: raw.project_summary.and_then(|s| s.metrics),
        file_summary_metrics: raw.file_summary.and_then(|s| s.metrics),
    }
}
```

Also add `use serde::Deserialize;` at the top of `src/config.rs`. The full file top should be:

```rust
use std::path::Path;
use serde::Deserialize;
```

- [ ] **Step 5: Run tests to confirm they pass**

```bash
cargo test config::tests
```

Expected: 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/config.rs src/lib.rs
git commit -m "feat: add ReportConfig and load_from_dir for lede.toml parsing"
```

---

## Task 3: Thread `ReportConfig` through formatter structs and `main.rs`

**Files:**
- Modify: `src/output/mod.rs`
- Modify: `src/output/markdown.rs`
- Modify: `src/output/pretty.rs`
- Modify: `src/output/json.rs`
- Modify: `src/main.rs`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Update formatter structs in `src/output/mod.rs`**

Replace the entire contents of `src/output/mod.rs` with:

```rust
use crate::config::ReportConfig;
use crate::duplicates::DuplicateCluster;
use crate::FileResult;

pub trait OutputFormatter {
    fn format(&self, results: &[FileResult], clusters: &[DuplicateCluster]) -> String;
}

pub mod json;
pub mod markdown;
pub mod pretty;

pub fn get_formatter(format: &str, config: ReportConfig) -> Box<dyn OutputFormatter> {
    match format {
        "json" => Box::new(json::JsonFormatter { config }),
        "pretty" => Box::new(pretty::PrettyFormatter { config }),
        _ => Box::new(markdown::MarkdownFormatter { config }),
    }
}
```

- [ ] **Step 2: Add `config` field to `MarkdownFormatter`**

In `src/output/markdown.rs`, replace the struct definition and `use` line at the top:

```rust
use crate::{FileResult, FunctionComplexity, SummaryStatistics, config::ReportConfig, duplicates::DuplicateCluster, output::OutputFormatter};

pub struct MarkdownFormatter {
    pub config: ReportConfig,
}
```

- [ ] **Step 3: Add `config` field to `PrettyFormatter`**

In `src/output/pretty.rs`, replace the struct definition and `use` line:

```rust
use crate::{FileResult, config::ReportConfig, duplicates::DuplicateCluster, output::OutputFormatter};
use comfy_table::{Table, ContentArrangement};

pub struct PrettyFormatter {
    pub config: ReportConfig,
}
```

- [ ] **Step 4: Add `config` field to `JsonFormatter`**

In `src/output/json.rs`, replace the struct definition and `use` line:

```rust
use crate::{AnalysisOutput, FileResult, SummaryStatistics, config::ReportConfig, duplicates::DuplicateCluster, output::OutputFormatter};

pub struct JsonFormatter {
    pub config: ReportConfig,
}
```

- [ ] **Step 5: Update `main.rs` to load config and pass it to `get_formatter`**

Replace the entire contents of `src/main.rs` with:

```rust
use clap::Parser;
use lede::{analyze_path, config::ReportConfig, output};
use std::process;

#[derive(Parser)]
#[command(name = "lede", version)]
struct Args {
    /// Path to a file or directory to analyze
    path: std::path::PathBuf,

    /// Output format: markdown, pretty, or json
    #[arg(short, long, default_value = "markdown")]
    format: String,

    /// Include closures and lambda expressions in the analysis
    #[arg(long)]
    include_closures: bool,
}

fn main() {
    let args = Args::parse();

    let config_dir = if args.path.is_dir() {
        args.path.clone()
    } else {
        args.path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf()
    };

    let config = ReportConfig::load_from_dir(&config_dir);

    let results = match analyze_path(&args.path, args.include_closures) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    for file in &results {
        if let Some(ref err) = file.error {
            eprintln!("Error parsing {}: {}", file.path.display(), err);
        }
    }

    let clusters = lede::duplicates::compute_duplicates(&results);
    let formatter = output::get_formatter(&args.format, config);
    println!("{}", formatter.format(&results, &clusters));
}
```

- [ ] **Step 6: Fix the breaking test in `tests/integration_test.rs`**

In `tests/integration_test.rs`, find the line:

```rust
    let formatter = lede::output::markdown::MarkdownFormatter;
```

Replace it with:

```rust
    let formatter = lede::output::markdown::MarkdownFormatter { config: lede::config::ReportConfig::default() };
```

- [ ] **Step 7: Verify the build and all existing tests pass**

```bash
cargo test
```

Expected: All existing tests pass (no behaviour change yet — just structural wiring).

- [ ] **Step 8: Commit**

```bash
git add src/output/mod.rs src/output/markdown.rs src/output/pretty.rs src/output/json.rs src/main.rs tests/integration_test.rs
git commit -m "refactor: thread ReportConfig through formatter structs and main"
```

---

## Task 4: Add `introduction` field to `AnalysisOutput` and render it in all formatters

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/output/markdown.rs`
- Modify: `src/output/pretty.rs`
- Modify: `src/output/json.rs`
- Create: `tests/fixtures/config_intro/rust_sample.rs`
- Create: `tests/fixtures/config_intro/lede.toml`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Create the `config_intro` fixture directory**

Create `tests/fixtures/config_intro/rust_sample.rs` with:

```rust
fn hello() -> i32 {
    let x = 1;
    if x > 0 {
        x + 1
    } else {
        0
    }
}
```

Create `tests/fixtures/config_intro/lede.toml` with:

```toml
introduction = "CUSTOM INTRO TEXT FOR TESTING"
```

- [ ] **Step 2: Write the failing integration test**

Add to `tests/integration_test.rs`:

```rust
#[test]
fn test_config_introduction_appears_in_markdown_output() {
    let output = lede()
        .arg("tests/fixtures/config_intro")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CUSTOM INTRO TEXT FOR TESTING"),
        "expected introduction text in output, got:\n{stdout}"
    );
    // introduction should appear before the summary heading
    let intro_pos = stdout.find("CUSTOM INTRO TEXT FOR TESTING").unwrap();
    let summary_pos = stdout.find("## Summary Statistics").unwrap_or(usize::MAX);
    assert!(intro_pos < summary_pos, "introduction should appear before summary");
}

#[test]
fn test_config_introduction_appears_in_json_output() {
    let output = lede()
        .arg("tests/fixtures/config_intro")
        .arg("-f")
        .arg("json")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
    assert_eq!(
        parsed["introduction"].as_str(),
        Some("CUSTOM INTRO TEXT FOR TESTING"),
        "expected 'introduction' key in JSON output"
    );
}
```

- [ ] **Step 3: Run the new tests to confirm they fail**

```bash
cargo test test_config_introduction
```

Expected: Both tests fail.

- [ ] **Step 4: Add `introduction` field to `AnalysisOutput` in `src/lib.rs`**

In `src/lib.rs`, replace the `AnalysisOutput` struct definition:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOutput {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub introduction: Option<String>,
    pub summary: SummaryStatistics,
    pub files: Vec<FileResult>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub clusters: Option<Vec<crate::duplicates::DuplicateCluster>>,
}
```

- [ ] **Step 5: Render introduction in `src/output/markdown.rs`**

In `src/output/markdown.rs`, replace the `format` method body:

```rust
impl OutputFormatter for MarkdownFormatter {
    fn format(&self, results: &[FileResult], clusters: &[DuplicateCluster]) -> String {
        let mut out = String::new();

        if let Some(ref intro) = self.config.introduction {
            out.push_str(intro);
            out.push_str("\n\n");
        }

        let summary = SummaryStatistics::from_results(results);
        if summary.files_analyzed > 0 {
            out.push_str(&format_summary(&summary, &self.config));
        }

        if !clusters.is_empty() {
            out.push_str(&format_clusters(clusters));
        }

        for file in results {
            out.push_str(&format_file(file, &self.config));
        }

        out
    }
}
```

Note: `format_summary` and `format_file` signatures will be updated in Task 5 — for now just pass `&self.config` as a parameter to both functions and update their signatures to accept `_config: &ReportConfig` (unused for now).

Add `use crate::config::ReportConfig;` to the imports in `markdown.rs` if not already present (it will be from the struct definition).

Update `format_summary` signature:

```rust
fn format_summary(summary: &SummaryStatistics, _config: &ReportConfig) -> String {
```

Update `format_file` signature:

```rust
fn format_file(file: &FileResult, _config: &ReportConfig) -> String {
    if let Some(ref err) = file.error {
        return format!("**{}**: ERROR: {}\n\n", file.path.display(), err);
    }
    if file.functions.is_empty() {
        return String::new();
    }

    let mut out = format!("### {}\n\n", file.path.display());
    out.push_str("#### File Summary\n\n");
    out.push_str(&format_file_summary(file, _config));
    out.push_str(&format_function_table(&file.functions));
    out
}
```

Update `format_file_summary` signature:

```rust
fn format_file_summary(file: &FileResult, _config: &ReportConfig) -> String {
```

- [ ] **Step 6: Render introduction in `src/output/pretty.rs`**

In `src/output/pretty.rs`, replace the `format` method body:

```rust
impl OutputFormatter for PrettyFormatter {
    fn format(&self, results: &[FileResult], clusters: &[DuplicateCluster]) -> String {
        let mut out = String::new();

        if let Some(ref intro) = self.config.introduction {
            out.push_str(intro);
            out.push('\n');
            out.push('\n');
        }

        if !clusters.is_empty() {
            out.push_str("Structural Duplication Candidates\n\n");
            for cluster in clusters {
                let n = cluster.instances.len();
                let suffix = if n == 1 { "" } else { "es" };
                out.push_str(&format!("{} ({} exact match{})\n", cluster.name, n, suffix));
                for inst in &cluster.instances {
                    out.push_str(&format!(
                        "  {}:{}  CC={}  lines={}  nest={}  vol={:.2}  diff={:.2}\n",
                        inst.path.display(),
                        inst.line_start,
                        inst.complexity,
                        inst.lines,
                        inst.nesting_depth,
                        inst.halstead_volume,
                        inst.halstead_difficulty
                    ));
                }
                out.push('\n');
            }
        }
        out.push_str(&results.iter().map(format_file_entry).collect::<String>());
        out
    }
}
```

- [ ] **Step 7: Render introduction in `src/output/json.rs`**

In `src/output/json.rs`, replace the `format` method body:

```rust
impl OutputFormatter for JsonFormatter {
    fn format(&self, results: &[FileResult], clusters: &[DuplicateCluster]) -> String {
        let summary = SummaryStatistics::from_results(results);

        let output = AnalysisOutput {
            introduction: self.config.introduction.clone(),
            summary,
            files: results.to_vec(),
            clusters: if clusters.is_empty() { None } else { Some(clusters.to_vec()) },
        };

        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }
}
```

- [ ] **Step 8: Run the new tests to confirm they pass**

```bash
cargo test test_config_introduction
```

Expected: Both tests pass.

- [ ] **Step 9: Run full test suite**

```bash
cargo test
```

Expected: All tests pass.

- [ ] **Step 10: Commit**

```bash
git add src/lib.rs src/output/markdown.rs src/output/pretty.rs src/output/json.rs tests/integration_test.rs tests/fixtures/config_intro/
git commit -m "feat: render introduction text from lede.toml in all formatters"
```

---

## Task 5: Implement metric filtering for project summary in markdown

**Files:**
- Modify: `src/output/markdown.rs`
- Create: `tests/fixtures/config_filtered/rust_sample.rs`
- Create: `tests/fixtures/config_filtered/lede.toml`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Create the `config_filtered` fixture directory**

Create `tests/fixtures/config_filtered/rust_sample.rs` with:

```rust
fn hello() -> i32 {
    let x = 1;
    if x > 0 {
        x + 1
    } else {
        0
    }
}

fn world(n: i32) -> i32 {
    for i in 0..n {
        if i % 2 == 0 {
            return i;
        }
    }
    0
}
```

Create `tests/fixtures/config_filtered/lede.toml` with:

```toml
[project_summary]
metrics = ["files_analyzed", "total_complexity"]

[file_summary]
metrics = ["max_complexity", "max_nesting_depth"]
```

- [ ] **Step 2: Write the failing integration test**

Add to `tests/integration_test.rs`:

```rust
#[test]
fn test_config_metric_filtering_markdown() {
    let output = lede()
        .arg("tests/fixtures/config_filtered")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Project summary: only "files_analyzed" and "total_complexity" should appear
    assert!(stdout.contains("Files Analyzed"), "expected Files Analyzed in output");
    assert!(stdout.contains("Total Complexity"), "expected Total Complexity in output");

    // File summary: only "max_complexity" and "max_nesting_depth" should appear
    assert!(stdout.contains("Max Complexity"), "expected Max Complexity in file summary");
    assert!(stdout.contains("Max Nesting Depth"), "expected Max Nesting Depth in file summary");

    // These labels only appear in summary table rows, not in the per-function table headers,
    // so their absence confirms metric filtering is working.
    assert!(!stdout.contains("Total Functions"), "Total Functions row should be absent (not in either summary list)");
    assert!(!stdout.contains("Total Lines"), "Total Lines row should be absent (not in either summary list)");
    assert!(!stdout.contains("Avg Complexity / Function"), "Avg Complexity / Function row should be absent");
    assert!(!stdout.contains("Avg Halstead Volume"), "Avg Halstead Volume row should be absent");
}
```

- [ ] **Step 3: Run the test to confirm it fails**

```bash
cargo test test_config_metric_filtering_markdown
```

Expected: FAIL — currently all metrics appear.

- [ ] **Step 4: Implement project summary filtering in `src/output/markdown.rs`**

Define a helper at the bottom of `markdown.rs` that maps metric keys to their label+value pair for the summary table:

```rust
fn project_summary_row(key: &str, summary: &SummaryStatistics) -> Option<String> {
    match key {
        "files_analyzed" => Some(metric_row("Files Analyzed", summary.files_analyzed)),
        "total_functions" => Some(metric_row("Total Functions", summary.total_functions)),
        "total_lines" => Some(metric_row("Total Lines", summary.total_lines)),
        "total_complexity" => Some(metric_row("Total Complexity", summary.total_complexity)),
        "avg_complexity_per_function" => Some(metric_row_f64("Avg Complexity / Function", summary.avg_complexity_per_function, 2)),
        "max_nesting_depth" => Some(metric_row("Max Nesting Depth", summary.max_nesting_depth)),
        "avg_nesting_depth" => Some(metric_row_f64("Avg Nesting Depth", summary.avg_nesting_depth, 2)),
        "avg_halstead_volume" => Some(metric_row_f64("Avg Halstead Volume", summary.avg_halstead_volume, 2)),
        "avg_halstead_difficulty" => Some(metric_row_f64("Avg Halstead Difficulty", summary.avg_halstead_difficulty, 2)),
        "avg_halstead_effort" => Some(metric_row_f64("Avg Halstead Effort", summary.avg_halstead_effort, 2)),
        "avg_halstead_time" => Some(metric_row_f64("Avg Halstead Time", summary.avg_halstead_time, 2)),
        unknown => {
            eprintln!("Warning: unknown metric key '{}', ignoring", unknown);
            None
        }
    }
}
```

Replace the `format_summary` function in `markdown.rs` with:

```rust
fn format_summary(summary: &SummaryStatistics, config: &ReportConfig) -> String {
    let rows: Vec<String> = match &config.project_summary_metrics {
        None => vec![
            metric_row("Files Analyzed", summary.files_analyzed),
            metric_row("Total Functions", summary.total_functions),
            metric_row("Total Lines", summary.total_lines),
            metric_row("Total Complexity", summary.total_complexity),
            metric_row_f64("Avg Complexity / Function", summary.avg_complexity_per_function, 2),
            metric_row("Max Nesting Depth", summary.max_nesting_depth),
            metric_row_f64("Avg Nesting Depth", summary.avg_nesting_depth, 2),
            metric_row_f64("Avg Halstead Volume", summary.avg_halstead_volume, 2),
            metric_row_f64("Avg Halstead Difficulty", summary.avg_halstead_difficulty, 2),
            metric_row_f64("Avg Halstead Effort", summary.avg_halstead_effort, 2),
            metric_row_f64("Avg Halstead Time", summary.avg_halstead_time, 2),
        ],
        Some(keys) => keys
            .iter()
            .filter_map(|k| project_summary_row(k, summary))
            .collect(),
    };

    let mut out = String::from("## Summary Statistics\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    for row in rows {
        out.push_str(&row);
    }
    out.push('\n');
    out
}
```

- [ ] **Step 5: Run the test and check project summary filtering passes (file summary part will still fail)**

```bash
cargo test test_config_metric_filtering_markdown
```

Expected: Still failing on the file summary assertions (Total Lines and Halstead still appear). That's expected — Task 6 covers file summary.

---

## Task 6: Implement metric filtering for file summary in markdown

**Files:**
- Modify: `src/output/markdown.rs`

- [ ] **Step 1: Add `file_summary_row` helper to `src/output/markdown.rs`**

Add this function alongside `project_summary_row`:

```rust
fn file_summary_row(key: &str, file: &FileResult, avg_complexity: f64) -> Option<String> {
    match key {
        "total_functions" => Some(metric_row("Total Functions", file.function_count)),
        "total_lines" => Some(metric_row("Total Lines", file.total_lines)),
        "total_function_lines" => Some(metric_row("Total Function Lines", file.total_function_lines)),
        "total_complexity" => Some(metric_row("Total Complexity", file.total_complexity)),
        "avg_complexity_per_function" => Some(metric_row_f64("Avg Complexity / Function", avg_complexity, 2)),
        "max_complexity" => Some(metric_row("Max Complexity", file.max_complexity)),
        "max_nesting_depth" => Some(metric_row("Max Nesting Depth", file.max_nesting_depth)),
        "avg_nesting_depth" => Some(metric_row_f64("Avg Nesting Depth", file.avg_nesting_depth, 2)),
        "max_function_lines" => Some(metric_row("Max Function Lines", file.max_function_lines)),
        "avg_halstead_volume" => Some(metric_row_f64("Avg Halstead Volume", file.avg_halstead_volume, 2)),
        "max_halstead_volume" => Some(metric_row_f64("Max Halstead Volume", file.max_halstead_volume, 2)),
        "avg_halstead_difficulty" => Some(metric_row_f64("Avg Halstead Difficulty", file.avg_halstead_difficulty, 2)),
        "max_halstead_difficulty" => Some(metric_row_f64("Max Halstead Difficulty", file.max_halstead_difficulty, 2)),
        "avg_halstead_effort" => Some(metric_row_f64("Avg Halstead Effort", file.avg_halstead_effort, 2)),
        "max_halstead_effort" => Some(metric_row_f64("Max Halstead Effort", file.max_halstead_effort, 2)),
        "avg_halstead_time" => Some(metric_row_f64("Avg Halstead Time", file.avg_halstead_time, 2)),
        "max_halstead_time" => Some(metric_row_f64("Max Halstead Time", file.max_halstead_time, 2)),
        unknown => {
            eprintln!("Warning: unknown metric key '{}', ignoring", unknown);
            None
        }
    }
}
```

- [ ] **Step 2: Replace `format_file_summary` in `src/output/markdown.rs`**

```rust
fn format_file_summary(file: &FileResult, config: &ReportConfig) -> String {
    let fc = file.function_count;
    let avg_complexity = if fc > 0 {
        file.total_complexity as f64 / fc as f64
    } else {
        0.0
    };

    let rows: Vec<String> = match &config.file_summary_metrics {
        None => vec![
            metric_row("Total Functions", file.function_count),
            metric_row("Total Lines", file.total_lines),
            metric_row("Total Function Lines", file.total_function_lines),
            metric_row("Total Complexity", file.total_complexity),
            metric_row_f64("Avg Complexity / Function", avg_complexity, 2),
            metric_row("Max Complexity", file.max_complexity),
            metric_row("Max Nesting Depth", file.max_nesting_depth),
            metric_row_f64("Avg Nesting Depth", file.avg_nesting_depth, 2),
            metric_row("Max Function Lines", file.max_function_lines),
            metric_row_f64("Avg Halstead Volume", file.avg_halstead_volume, 2),
            metric_row_f64("Max Halstead Volume", file.max_halstead_volume, 2),
            metric_row_f64("Avg Halstead Difficulty", file.avg_halstead_difficulty, 2),
            metric_row_f64("Max Halstead Difficulty", file.max_halstead_difficulty, 2),
            metric_row_f64("Avg Halstead Effort", file.avg_halstead_effort, 2),
            metric_row_f64("Max Halstead Effort", file.max_halstead_effort, 2),
            metric_row_f64("Avg Halstead Time", file.avg_halstead_time, 2),
            metric_row_f64("Max Halstead Time", file.max_halstead_time, 2),
        ],
        Some(keys) => keys
            .iter()
            .filter_map(|k| file_summary_row(k, file, avg_complexity))
            .collect(),
    };

    let mut out = String::from("| Metric | Value |\n|--------|-------|\n");
    for row in rows {
        out.push_str(&row);
    }
    out.push('\n');
    out
}
```

- [ ] **Step 3: Run the filtering test — it should now fully pass**

```bash
cargo test test_config_metric_filtering_markdown
```

Expected: PASS.

- [ ] **Step 4: Run the full test suite**

```bash
cargo test
```

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/output/markdown.rs tests/integration_test.rs tests/fixtures/config_filtered/
git commit -m "feat: implement metric-level filtering for project and file summaries in markdown"
```

---

## Task 7: Integration test for unknown metric key warning

**Files:**
- Create: `tests/fixtures/config_unknown_key/rust_sample.rs`
- Create: `tests/fixtures/config_unknown_key/lede.toml`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Create the `config_unknown_key` fixture directory**

Create `tests/fixtures/config_unknown_key/rust_sample.rs`:

```rust
fn simple(x: i32) -> i32 {
    x + 1
}
```

Create `tests/fixtures/config_unknown_key/lede.toml`:

```toml
[project_summary]
metrics = ["files_analyzed", "totally_fake_metric"]
```

- [ ] **Step 2: Add the integration test**

Add to `tests/integration_test.rs`:

```rust
#[test]
fn test_config_unknown_metric_key_warns_and_succeeds() {
    let output = lede()
        .arg("tests/fixtures/config_unknown_key")
        .output()
        .expect("failed to run lede");

    // Analysis must succeed
    assert!(output.status.success(), "lede should exit 0 even with unknown metric key");

    // Known key still appears in output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Files Analyzed"), "known metric key should still render");

    // Unknown key does NOT appear in output
    assert!(!stdout.contains("totally_fake_metric"), "unknown key should not appear in output");

    // Warning appears on stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("totally_fake_metric"),
        "expected warning about unknown key on stderr, got: {stderr}"
    );
}
```

- [ ] **Step 3: Run the test**

```bash
cargo test test_config_unknown_metric_key_warns_and_succeeds
```

Expected: PASS (the `project_summary_row` function already emits warnings for unknown keys).

- [ ] **Step 4: Run the full test suite**

```bash
cargo test
```

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add tests/integration_test.rs tests/fixtures/config_unknown_key/
git commit -m "test: add integration test for unknown metric key warning"
```

---

## Task 8: Update README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add `lede.toml` configuration section to `README.md`**

Add the following new section after the `### CLI Options` block and before `## Example Output`:

```markdown
## Configuration

lede reads an optional `lede.toml` file from the analyzed directory (or the source file's parent directory). Use it to add a custom introduction to the report and to select which metrics appear in the project and file summary sections.

### Example `lede.toml`

```toml
# Optional introduction text prepended to the report
introduction = """
This report analyzes the complexity of the Acme project.
Metrics are computed per-function and aggregated per-file and project-wide.
"""

[project_summary]
# If absent, all metrics are shown. Listed keys appear in order.
metrics = [
  "files_analyzed",
  "total_functions",
  "total_complexity",
  "avg_complexity_per_function",
]

[file_summary]
# If absent, all metrics are shown.
metrics = [
  "total_functions",
  "total_complexity",
  "avg_complexity_per_function",
  "max_complexity",
  "max_nesting_depth",
]
```

### Available Metric Keys

**Project summary** (`[project_summary]`): `files_analyzed`, `total_functions`, `total_lines`, `total_complexity`, `avg_complexity_per_function`, `max_nesting_depth`, `avg_nesting_depth`, `avg_halstead_volume`, `avg_halstead_difficulty`, `avg_halstead_effort`, `avg_halstead_time`

**File summary** (`[file_summary]`): `total_functions`, `total_lines`, `total_function_lines`, `total_complexity`, `avg_complexity_per_function`, `max_complexity`, `max_nesting_depth`, `avg_nesting_depth`, `max_function_lines`, `avg_halstead_volume`, `max_halstead_volume`, `avg_halstead_difficulty`, `max_halstead_difficulty`, `avg_halstead_effort`, `max_halstead_effort`, `avg_halstead_time`, `max_halstead_time`

If `lede.toml` is absent or a section is omitted, all metrics are shown (default behaviour).
```

(Note: the inner fenced code block uses triple backticks — ensure your Markdown renderer handles nested fences correctly, or use `~~~` for the outer fence.)

- [ ] **Step 2: Verify the build and test suite still clean**

```bash
cargo test
```

Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: document lede.toml configuration in README"
```

---

## Self-Review Checklist

After completing all tasks, run:

```bash
cargo test
```

Verify:
- All existing tests still pass
- 6 new tests pass: `test_config_introduction_appears_in_markdown_output`, `test_config_introduction_appears_in_json_output`, `test_config_metric_filtering_markdown`, `test_config_unknown_metric_key_warns_and_succeeds`, plus the 3 `config::tests` unit tests
- `lede tests/fixtures/config_intro` outputs the introduction text
- `lede tests/fixtures/config_filtered` shows only the 2 configured project metrics and 2 file metrics
- `lede tests/fixtures` (no lede.toml) behaves identically to before this feature
