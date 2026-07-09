# polygraph

A fast CLI tool that computes cyclomatic code complexity and Halstead metrics for Rust, Python, JavaScript, TypeScript, C, and Java source files. It reports complexity and cognitive metrics per function and per file. By default, closures and anonymous functions are excluded from analysis so they don't skew aggregate metrics — you can opt to include them with `--include-closures`.

## Features

- **Multi-language support:** Rust, Python, JavaScript/JSX, TypeScript/TSX, C, and Java
- **Cyclomatic complexity:** Classic decision-point counting per function and file
- **Halstead metrics:** Volume, difficulty, effort, and estimated time per function
- **Nesting depth analysis:** Maximum and average control-flow nesting per function and file
- **Per-function & per-file reporting:** See complexity and Halstead metrics at every level
- **Project-level summary:** Aggregated statistics across all analyzed files
- **Closure handling:** Closures, lambdas, and arrow functions are excluded by default so they don't inflate function counts or dilute averages. Use `--include-closures` to analyze them
- **Two output formats:** Pretty-printed tables (default) and JSON
- **Directory scanning:** Analyze entire codebases recursively
- **Graceful error handling:** Unparseable files are reported to stderr but do not stop the analysis

## Installation

Build from source with Cargo:

```bash
git clone https://github.com/tcrichardson/polygraph
cd polygraph
cargo build --release
```

The binary will be available at `target/release/polygraph`.

## Usage

Analyze a single file:

```bash
polygraph src/main.rs
```

Analyze an entire directory:

```bash
polygraph src/
```

Output as JSON:

```bash
polygraph src/ -f json
```

Include closures and lambdas in the analysis:

```bash
polygraph src/ --include-closures
```

### CLI Options

```
Usage: polygraph [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to a file or directory to analyze

Options:
  -c, --config <CONFIG>     Path to a polygraph.toml configuration file
  -f, --format <FORMAT>     Output format: pretty or json [default: pretty]
      --include-closures    Include closures, lambdas, and arrow functions in the analysis
  -h, --help                Print help
  -V, --version             Print version
```

## Configuration

polygraph uses a `polygraph.toml` configuration file to control the report output. By default it reads from `config/polygraph.toml`. You can specify a different file with the `--config` (or `-c`) option.

Use the configuration file to add a custom introduction to the report and to select which metrics appear in the project and file summary sections.

If the configuration file is absent or a section is omitted, all metrics are shown (default behaviour unchanged).

### Example `polygraph.toml`

~~~toml
# Optional introduction text prepended to the report
introduction = """
This report analyzes the complexity of the Acme project.
Metrics are computed per-function and aggregated per-file and project-wide.
"""

[project_summary]
# If absent, all metrics are shown. Listed keys appear in the order specified.
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
~~~

### Available Metric Keys

**Project summary** (`[project_summary]`):

| Key | Description |
|-----|-------------|
| `files_analyzed` | Number of files successfully analyzed |
| `total_functions` | Total function count across all files |
| `total_lines` | Total source lines across all files |
| `total_complexity` | Sum of cyclomatic complexity |
| `avg_complexity_per_function` | Average cyclomatic complexity per function |
| `max_nesting_depth` | Deepest nesting found in any function |
| `avg_nesting_depth` | Average nesting depth across all functions |
| `avg_halstead_volume` | Average Halstead volume |
| `avg_halstead_difficulty` | Average Halstead difficulty |
| `avg_halstead_effort` | Average Halstead effort |
| `avg_halstead_time` | Average estimated implementation time |

**File summary** (`[file_summary]`):

| Key | Description |
|-----|-------------|
| `total_functions` | Number of functions in the file |
| `total_lines` | Total lines in the file |
| `total_function_lines` | Sum of lines across all functions |
| `total_complexity` | Sum of cyclomatic complexity |
| `avg_complexity_per_function` | Average cyclomatic complexity per function |
| `max_complexity` | Highest cyclomatic complexity in any function |
| `max_nesting_depth` | Deepest nesting in any function |
| `avg_nesting_depth` | Average nesting depth across functions |
| `max_function_lines` | Length of the longest function |
| `avg_halstead_volume` | Average Halstead volume |
| `max_halstead_volume` | Maximum Halstead volume |
| `avg_halstead_difficulty` | Average Halstead difficulty |
| `max_halstead_difficulty` | Maximum Halstead difficulty |
| `avg_halstead_effort` | Average Halstead effort |
| `max_halstead_effort` | Maximum Halstead effort |
| `avg_halstead_time` | Average estimated implementation time |
| `max_halstead_time` | Maximum estimated implementation time |

## Example Output

### Pretty format (default)

## Summary Statistics

| Metric | Value |
|--------|-------|
| Files Analyzed | 3 |
| Total Functions | 12 |
| Total Lines | 450 |
| Total Complexity | 34 |
| Avg Complexity / Function | 2.83 |
| Max Nesting Depth | 4 |
| Avg Nesting Depth | 1.50 |
| Avg Halstead Volume | 78.34 |
| Avg Halstead Difficulty | 4.20 |
| Avg Halstead Effort | 329.03 |
| Avg Halstead Time | 18.28 |

### src/main.rs

#### File Summary

| Metric | Value |
|--------|-------|
| Total Functions | 1 |
| Total Lines | 42 |
| Total Function Lines | 20 |
| Total Complexity | 5 |
| Avg Complexity / Function | 5.00 |
| Max Complexity | 5 |
| Max Nesting Depth | 2 |
| Avg Nesting Depth | 2.00 |
| Max Function Lines | 20 |
| Avg Halstead Volume | 45.60 |
| Max Halstead Volume | 45.60 |
| Avg Halstead Difficulty | 3.20 |
| Max Halstead Difficulty | 3.20 |
| Avg Halstead Effort | 145.92 |
| Max Halstead Effort | 145.92 |
| Avg Halstead Time | 8.11 |
| Max Halstead Time | 8.11 |

| Function | Lines | Line Range | Complexity | Nesting | Halstead Vol | Difficulty | Halstead Effort | Halstead Time |
|----------|-------|------------|------------|---------|--------------|------------|-----------------|---------------|
| main | 20 | 16-35 | 5 | 2 | 45.60 | 3.20 | 145.92 | 8.11 |

### JSON format

```json
{
  "summary": {
    "files_analyzed": 3,
    "total_functions": 12,
    "total_lines": 450,
    "total_complexity": 34,
    "avg_complexity_per_function": 2.83,
    "max_nesting_depth": 4,
    "avg_nesting_depth": 1.50,
    "avg_halstead_volume": 78.34,
    "avg_halstead_difficulty": 4.20,
    "avg_halstead_effort": 329.03,
    "avg_halstead_time": 18.28
  },
  "files": [
    {
      "path": "src/main.rs",
      "total_complexity": 5,
      "total_lines": 42,
      "function_count": 1,
      "functions": [
        {
          "name": "main",
          "line_start": 16,
          "line_end": 35,
          "lines": 20,
          "complexity": 5,
          "nesting_depth": 2,
          "halstead_volume": 45.60,
          "halstead_difficulty": 3.20,
          "halstead_effort": 145.92,
          "halstead_time": 8.11
        }
      ],
      "max_nesting_depth": 2,
      "avg_nesting_depth": 2.00,
      "avg_halstead_volume": 45.60,
      "avg_halstead_difficulty": 3.20,
      "avg_halstead_effort": 145.92,
      "avg_halstead_time": 8.11,
      "max_complexity": 5,
      "max_function_lines": 20,
      "total_function_lines": 20,
      "max_halstead_volume": 45.60,
      "max_halstead_difficulty": 3.20,
      "max_halstead_effort": 145.92,
      "max_halstead_time": 8.11
    }
  ]
}
```

## How Complexity is Calculated

For each function or closure, complexity starts at **1** and increments by **1** for each decision point:

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

\* Only counted when `--include-closures` is passed. By default, closures are excluded so they don't inflate function counts or dilute average metrics.

Per-file complexity is the sum of all function complexities in that file.

## How Metrics are Calculated

### Nesting Depth
The maximum depth of control-flow block nesting inside a function. For example, an `if` inside a `for` loop has a nesting depth of 2. Nested functions are not counted.

### Halstead Metrics
Derived from counting distinct operators and operands within each function:
- **Volume** — `N × log₂(η)` where `N` is total tokens and `η` is distinct tokens
- **Difficulty** — `(η₁ / 2) × (N₂ / η₂)` where `η₁` is distinct operators, `N₂` is total operands, and `η₂` is distinct operands
- **Effort** — `Volume × Difficulty`
- **Time** — `Effort / 18` (estimated time to implement, in seconds)

### Per-File Aggregates
- **avg_complexity_per_function** — average cyclomatic complexity across all functions
- **max_complexity** — highest complexity found in any function
- **max_nesting_depth** — deepest nesting found in any function
- **avg_nesting_depth** — average nesting depth across all functions
- **max_function_lines** — longest function in lines
- **total_function_lines** — sum of all function line counts
- **avg/max_halstead_volume** — average and maximum Halstead volume
- **avg/max_halstead_difficulty** — average and maximum Halstead difficulty
- **avg/max_halstead_effort** — average and maximum Halstead effort
- **avg/max_halstead_time** — average and maximum estimated implementation time

### Project-Level Summary
When analyzing multiple files, the JSON and pretty output include a top-level summary aggregating statistics across all successfully analyzed files with functions.

## Supported File Extensions

| Language | Extensions |
|---|---|
| Rust | `.rs` |
| Python | `.py` |
| JavaScript | `.js`, `.jsx` |
| TypeScript | `.ts`, `.tsx` |
| C | `.c`, `.h` |
| Java | `.java` |

Files with unsupported extensions are silently skipped.

## Testing

Run the full test suite:

```bash
cargo test
```

The suite includes:
- **Unit tests** for each language analyzer and the cognitive metrics module (61 tests)
- **Integration tests** that exercise the CLI against fixture files (13 tests)

## Architecture

Polygraph uses [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) to parse source code into ASTs. Each language has a dedicated analyzer that walks the AST to find function boundaries and count decision points. A shared `cognitive` module computes nesting depth and Halstead metrics for every function. A shared dispatcher routes files to the correct analyzer based on extension.

## License

MIT
