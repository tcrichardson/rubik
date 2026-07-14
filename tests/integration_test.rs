use polygraph::output::OutputFormatter;
use std::process::Command;

fn polygraph() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--");
    cmd
}

#[test]
fn test_rust_fixture_pretty() {
    let output = polygraph()
        .arg("tests/fixtures/rust_sample.rs")
        .output()
        .expect("failed to run polygraph");
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
    let output = polygraph()
        .arg("tests/fixtures/rust_sample.rs")
        .arg("--include-closures")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("simple"));
    assert!(stdout.contains("nested"));
    assert!(stdout.contains("<closure>"));
}

#[test]
fn test_python_fixture_json() {
    let output = polygraph()
        .arg("tests/fixtures/python_sample.py")
        .arg("-f")
        .arg("json")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: polygraph::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
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
    let output = polygraph()
        .arg("tests/fixtures/js_sample.js")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: polygraph::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
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
    let output = polygraph()
        .arg("tests/fixtures/c_sample.c")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: polygraph::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
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
    let output = polygraph()
        .arg("tests/fixtures/typescript_sample.ts")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: polygraph::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
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
    let output = polygraph()
        .arg("tests/fixtures/invalid.py")
        .output()
        .expect("failed to run polygraph");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error parsing") || stderr.contains("Failed to parse"));
    assert!(output.status.success());
}

#[test]
fn test_directory_scan() {
    let output = polygraph()
        .arg("tests/fixtures")
        .arg("-f")
        .arg("json")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: polygraph::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
    let paths: Vec<String> = parsed
        .files
        .iter()
        .map(|r| r.path.to_string_lossy().to_string())
        .collect();
    assert!(paths.iter().any(|p| p.contains("rust_sample.rs")));
    assert!(paths.iter().any(|p| p.contains("python_sample.py")));
    assert!(paths.iter().any(|p| p.contains("js_sample.js")));
    assert!(paths.iter().any(|p| p.contains("c_sample.c")));
    assert!(paths.iter().any(|p| p.contains("typescript_sample.ts")));
    assert!(parsed.summary.files_analyzed >= 5);
}

#[test]
fn test_duplicate_clusters_in_output() {
    let results = polygraph::analyze_path(
        std::path::Path::new("tests/fixtures/duplicates/"),
        false,
        false,
    )
    .expect("failed to analyze duplicates directory");

    let clusters = polygraph::duplicates::compute_duplicates(&results);

    assert!(
        !clusters.is_empty(),
        "expected at least one duplicate cluster"
    );

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
    let formatter = polygraph::output::markdown::MarkdownFormatter {
        config: polygraph::config::ReportConfig::default(),
    };
    let output = formatter.format(&results, &clusters, &[]);
    assert!(
        output.contains("Structural Duplication Candidates"),
        "expected markdown output to contain duplication heading"
    );
}

#[test]
fn test_config_introduction_appears_in_markdown_output() {
    let output = polygraph()
        .arg("tests/fixtures/config_intro")
        .arg("--config")
        .arg("tests/fixtures/config_intro/polygraph.toml")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CUSTOM INTRO TEXT FOR TESTING"),
        "expected introduction text in output, got:\n{stdout}"
    );
    // introduction should appear before the summary heading
    let intro_pos = stdout.find("CUSTOM INTRO TEXT FOR TESTING").unwrap();
    let summary_pos = stdout.find("## Summary Statistics").unwrap_or(usize::MAX);
    assert!(
        intro_pos < summary_pos,
        "introduction should appear before summary"
    );
}

#[test]
fn test_config_metric_filtering_markdown() {
    let output = polygraph()
        .arg("tests/fixtures/config_filtered")
        .arg("--config")
        .arg("tests/fixtures/config_filtered/polygraph.toml")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Project summary: only "files_analyzed" and "total_complexity" should appear
    assert!(
        stdout.contains("Files Analyzed"),
        "expected Files Analyzed in output"
    );
    assert!(
        stdout.contains("Total Complexity"),
        "expected Total Complexity in output"
    );

    // File summary: only "max_complexity" and "max_nesting_depth" should appear
    assert!(
        stdout.contains("Max Complexity"),
        "expected Max Complexity in file summary"
    );
    assert!(
        stdout.contains("Max Nesting Depth"),
        "expected Max Nesting Depth in file summary"
    );

    // These labels only appear in summary table rows, not in the per-function table headers,
    // so their absence confirms metric filtering is working.
    assert!(
        !stdout.contains("Total Functions"),
        "Total Functions row should be absent (not in either summary list)"
    );
    assert!(
        !stdout.contains("Total Lines"),
        "Total Lines row should be absent (not in either summary list)"
    );
    assert!(
        !stdout.contains("Avg Complexity / Function"),
        "Avg Complexity / Function row should be absent"
    );
    assert!(
        !stdout.contains("Avg Halstead Volume"),
        "Avg Halstead Volume row should be absent"
    );
}

#[test]
fn test_config_introduction_appears_in_json_output() {
    let output = polygraph()
        .arg("tests/fixtures/config_intro")
        .arg("--config")
        .arg("tests/fixtures/config_intro/polygraph.toml")
        .arg("-f")
        .arg("json")
        .output()
        .expect("failed to run polygraph");
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
    let output = polygraph()
        .arg("tests/fixtures/config_unknown_key")
        .arg("--config")
        .arg("tests/fixtures/config_unknown_key/polygraph.toml")
        .output()
        .expect("failed to run polygraph");

    // Analysis must succeed
    assert!(
        output.status.success(),
        "polygraph should exit 0 even with unknown metric key"
    );

    // Known key still appears in output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Files Analyzed"),
        "known metric key should still render"
    );

    // Unknown key does NOT appear in output
    assert!(
        !stdout.contains("totally_fake_metric"),
        "unknown key should not appear in output"
    );

    // Warning appears on stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("totally_fake_metric"),
        "expected warning about unknown key on stderr, got: {stderr}"
    );
}

#[test]
fn test_compute_clones_detects_renamed_rust_pair() {
    let results = polygraph::analyze_path(
        std::path::Path::new("tests/fixtures/clones/rust"),
        false,
        true,
    )
    .expect("failed to analyze clones/rust directory");

    let clone_config = polygraph::clones::CloneConfig::default();
    let pairs = polygraph::clones::compute_clones(&results, &clone_config);

    let found = pairs.iter().find(|p| {
        (p.a.name == "calculate_total" && p.b.name == "compute_sum")
            || (p.a.name == "compute_sum" && p.b.name == "calculate_total")
    });
    assert!(
        found.is_some(),
        "expected a clone pair between calculate_total and compute_sum, got: {:#?}",
        pairs
    );
    assert!(found.unwrap().similarity >= clone_config.similarity_threshold);

    // The unrelated/dissimilar helper functions must not be reported as clones of each other.
    assert!(
        !pairs
            .iter()
            .any(|p| p.a.name == "unrelated_helper" || p.b.name == "unrelated_helper"),
        "unrelated_helper must not appear in any clone pair, got: {:#?}",
        pairs
    );
}

#[test]
fn test_compute_clones_detects_renamed_python_pair() {
    let results = polygraph::analyze_path(
        std::path::Path::new("tests/fixtures/clones/python"),
        false,
        true,
    )
    .expect("failed to analyze clones/python directory");

    let clone_config = polygraph::clones::CloneConfig::default();
    let pairs = polygraph::clones::compute_clones(&results, &clone_config);

    let found = pairs.iter().find(|p| {
        (p.a.name == "calculate_total" && p.b.name == "compute_sum")
            || (p.a.name == "compute_sum" && p.b.name == "calculate_total")
    });
    assert!(
        found.is_some(),
        "expected a clone pair between calculate_total and compute_sum, got: {:#?}",
        pairs
    );
    assert!(found.unwrap().similarity >= clone_config.similarity_threshold);
}

#[test]
fn test_compute_clones_detects_renamed_javascript_pair() {
    let results = polygraph::analyze_path(
        std::path::Path::new("tests/fixtures/clones/javascript"),
        false,
        true,
    )
    .expect("failed to analyze clones/javascript directory");

    let clone_config = polygraph::clones::CloneConfig::default();
    let pairs = polygraph::clones::compute_clones(&results, &clone_config);

    let found = pairs.iter().find(|p| {
        (p.a.name == "calculateTotal" && p.b.name == "computeSum")
            || (p.a.name == "computeSum" && p.b.name == "calculateTotal")
    });
    assert!(
        found.is_some(),
        "expected a clone pair between calculateTotal and computeSum, got: {:#?}",
        pairs
    );
    assert!(found.unwrap().similarity >= clone_config.similarity_threshold);

    assert!(
        !pairs
            .iter()
            .any(|p| p.a.name == "unrelatedHelper" || p.b.name == "unrelatedHelper"),
        "unrelatedHelper must not appear in any clone pair, got: {:#?}",
        pairs
    );
}

#[test]
fn test_compute_clones_detects_renamed_typescript_pair() {
    let results = polygraph::analyze_path(
        std::path::Path::new("tests/fixtures/clones/typescript"),
        false,
        true,
    )
    .expect("failed to analyze clones/typescript directory");

    let clone_config = polygraph::clones::CloneConfig::default();
    let pairs = polygraph::clones::compute_clones(&results, &clone_config);

    let found = pairs.iter().find(|p| {
        (p.a.name == "calculateTotal" && p.b.name == "computeSum")
            || (p.a.name == "computeSum" && p.b.name == "calculateTotal")
    });
    assert!(
        found.is_some(),
        "expected a clone pair between calculateTotal and computeSum, got: {:#?}",
        pairs
    );
    assert!(found.unwrap().similarity >= clone_config.similarity_threshold);

    assert!(
        !pairs
            .iter()
            .any(|p| p.a.name == "unrelatedHelper" || p.b.name == "unrelatedHelper"),
        "unrelatedHelper must not appear in any clone pair, got: {:#?}",
        pairs
    );
}

#[test]
fn test_clones_cli_off_by_default_leaves_output_unchanged() {
    let without_flag = polygraph()
        .arg("tests/fixtures/clones/rust")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&without_flag.stdout);
    assert!(
        !stdout.contains("Clone Candidates"),
        "clone section must not appear unless --clones is passed"
    );

    // Running the same command twice without --clones must be byte-identical,
    // confirming the flag carries zero cost/behavior change until opted in.
    let again = polygraph()
        .arg("tests/fixtures/clones/rust")
        .output()
        .expect("failed to run polygraph");
    assert_eq!(without_flag.stdout, again.stdout);
}

#[test]
fn test_clones_cli_flag_reports_clone_section_markdown() {
    let output = polygraph()
        .arg("tests/fixtures/clones/rust")
        .arg("--clones")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Clone Candidates"),
        "expected clone section in markdown output, got:\n{stdout}"
    );
    assert!(stdout.contains("calculate_total"));
    assert!(stdout.contains("compute_sum"));
}

#[test]
fn test_clones_cli_flag_reports_clone_section_json() {
    let output = polygraph()
        .arg("tests/fixtures/clones/rust")
        .arg("--clones")
        .arg("-f")
        .arg("json")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: polygraph::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
    let clones = parsed
        .clones
        .expect("expected clones field to be populated");
    assert!(!clones.is_empty());
    assert!(clones.iter().any(
        |p| (p.a.name == "calculate_total" && p.b.name == "compute_sum")
            || (p.a.name == "compute_sum" && p.b.name == "calculate_total")
    ));
}

#[test]
fn test_clones_cli_threshold_override_excludes_pair() {
    // A near-impossible threshold should exclude the otherwise-detected pair.
    let output = polygraph()
        .arg("tests/fixtures/clones/rust")
        .arg("--clones")
        .arg("--clone-threshold")
        .arg("1.5")
        .arg("-f")
        .arg("json")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: polygraph::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
    assert!(
        parsed.clones.is_none(),
        "a threshold above the maximum possible similarity must exclude every pair"
    );
}

#[test]
fn test_clones_cli_min_lines_override_excludes_small_functions() {
    // A very high min-lines requirement should exclude every function in the fixture.
    let output = polygraph()
        .arg("tests/fixtures/clones/rust")
        .arg("--clones")
        .arg("--clone-min-lines")
        .arg("1000")
        .arg("-f")
        .arg("json")
        .output()
        .expect("failed to run polygraph");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: polygraph::AnalysisOutput = serde_json::from_str(&stdout).expect("invalid JSON");
    assert!(parsed.clones.is_none());
}
