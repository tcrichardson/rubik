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
