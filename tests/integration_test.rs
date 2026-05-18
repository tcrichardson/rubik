use std::process::Command;
use lede::output::OutputFormatter;

fn lede() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--");
    cmd
}

#[test]
fn test_rust_fixture_pretty() {
    let output = lede()
        .arg("tests/fixtures/rust_sample.rs")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("simple"));
    assert!(stdout.contains("with_if"));
    assert!(stdout.contains("with_match"));
    assert!(stdout.contains("nested"));
    assert!(!stdout.contains("<closure>"));
    assert!(stdout.contains("Halstead Vol"));
    assert!(stdout.contains("Nesting"));
}

#[test]
fn test_rust_fixture_include_closures() {
    let output = lede()
        .arg("tests/fixtures/rust_sample.rs")
        .arg("--include-closures")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("simple"));
    assert!(stdout.contains("nested"));
    assert!(stdout.contains("<closure>"));
}

#[test]
fn test_python_fixture_json() {
    let output = lede()
        .arg("tests/fixtures/python_sample.py")
        .arg("-f")
        .arg("json")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: lede::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
    assert_eq!(parsed.files.len(), 1);
    assert_eq!(parsed.summary.files_analyzed, 1);
    let file = &parsed.files[0];
    assert!(file.path.to_string_lossy().contains("python_sample.py"));
    let names: Vec<&str> = file.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"simple"));
    assert!(names.contains(&"with_if"));
    assert!(names.contains(&"with_match"));
    assert!(names.contains(&"nested"));
    assert!(file.functions.iter().any(|f| f.halstead_effort > 0.0));
    assert!(file.avg_halstead_effort > 0.0);
    assert!(file.avg_halstead_volume >= 0.0);
    assert!(parsed.summary.total_functions > 0);
    assert!(parsed.summary.total_lines > 0);
}

#[test]
fn test_js_fixture_json() {
    let output = lede()
        .arg("tests/fixtures/js_sample.js")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: lede::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
    assert_eq!(parsed.files.len(), 1);
    assert_eq!(parsed.summary.files_analyzed, 1);
    let file = &parsed.files[0];
    assert!(file.path.to_string_lossy().contains("js_sample.js"));
    let names: Vec<&str> = file.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"simple"));
    assert!(names.contains(&"withIf"));
    assert!(file.functions.iter().any(|f| f.halstead_effort > 0.0));
    assert!(file.avg_halstead_effort > 0.0);
    assert!(file.avg_halstead_volume >= 0.0);
    assert!(parsed.summary.total_functions > 0);
    assert!(parsed.summary.total_lines > 0);
}

#[test]
fn test_c_fixture_json() {
    let output = lede()
        .arg("tests/fixtures/c_sample.c")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: lede::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
    assert_eq!(parsed.files.len(), 1);
    assert_eq!(parsed.summary.files_analyzed, 1);
    let file = &parsed.files[0];
    assert!(file.path.to_string_lossy().contains("c_sample.c"));
    let names: Vec<&str> = file.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"simple"));
    assert!(names.contains(&"withIf"));
    assert!(names.contains(&"withSwitch"));
    assert!(names.contains(&"nested"));
    assert!(file.functions.iter().any(|f| f.halstead_effort > 0.0));
    assert!(file.avg_halstead_effort > 0.0);
    assert!(file.avg_halstead_volume >= 0.0);
    assert!(parsed.summary.total_functions > 0);
    assert!(parsed.summary.total_lines > 0);
}

#[test]
fn test_typescript_fixture_json() {
    let output = lede()
        .arg("tests/fixtures/typescript_sample.ts")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: lede::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
    assert_eq!(parsed.files.len(), 1);
    assert_eq!(parsed.summary.files_analyzed, 1);
    let file = &parsed.files[0];
    assert!(file.path.to_string_lossy().contains("typescript_sample.ts"));
    let names: Vec<&str> = file.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"simple"));
    assert!(names.contains(&"withIf"));
    assert!(names.contains(&"withSwitch"));
    assert!(names.contains(&"nested"));
    assert!(file.functions.iter().any(|f| f.halstead_effort > 0.0));
    assert!(file.avg_halstead_effort > 0.0);
    assert!(file.avg_halstead_volume >= 0.0);
    assert!(parsed.summary.total_functions > 0);
    assert!(parsed.summary.total_lines > 0);
}

#[test]
fn test_invalid_file_skips() {
    let output = lede()
        .arg("tests/fixtures/invalid.py")
        .output()
        .expect("failed to run lede");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error parsing") || stderr.contains("Failed to parse"));
    assert!(output.status.success());
}

#[test]
fn test_directory_scan() {
    let output = lede()
        .arg("tests/fixtures")
        .arg("-f")
        .arg("json")
        .output()
        .expect("failed to run lede");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: lede::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
    let paths: Vec<String> = parsed.files.iter().map(|r| r.path.to_string_lossy().to_string()).collect();
    assert!(paths.iter().any(|p| p.contains("rust_sample.rs")));
    assert!(paths.iter().any(|p| p.contains("python_sample.py")));
    assert!(paths.iter().any(|p| p.contains("js_sample.js")));
    assert!(paths.iter().any(|p| p.contains("c_sample.c")));
    assert!(paths.iter().any(|p| p.contains("typescript_sample.ts")));
    assert!(parsed.summary.files_analyzed >= 5);
}

#[test]
fn test_duplicate_clusters_in_output() {
    let results = lede::analyze_path(
        std::path::Path::new("tests/fixtures/duplicates/"),
        false,
    )
    .expect("failed to analyze duplicates directory");

    let clusters = lede::duplicates::compute_duplicates(&results);

    assert!(!clusters.is_empty(), "expected at least one duplicate cluster");

    let duplicated_cluster = clusters
        .iter()
        .find(|c| c.name == "duplicated")
        .expect("expected a cluster named 'duplicated'");

    assert_eq!(
        duplicated_cluster.instances.len(),
        2,
        "expected exactly 2 instances of 'duplicated'"
    );

    // Also verify markdown output contains the duplication section
    let formatter = lede::output::markdown::MarkdownFormatter { config: lede::config::ReportConfig::default() };
    let output = formatter.format(&results, &clusters);
    assert!(
        output.contains("Structural Duplication Candidates"),
        "expected markdown output to contain duplication heading"
    );
}

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
