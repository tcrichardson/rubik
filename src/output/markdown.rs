use crate::{FileResult, FunctionComplexity, SummaryStatistics, config::ReportConfig, duplicates::DuplicateCluster, output::OutputFormatter};

pub struct MarkdownFormatter {
    pub config: ReportConfig,
}

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

fn format_summary(summary: &SummaryStatistics, _config: &ReportConfig) -> String {
    let rows = vec![
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
    ];

    let mut out = String::from("## Summary Statistics\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    for row in rows {
        out.push_str(&row);
    }
    out.push('\n');
    out
}

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

fn format_file_summary(file: &FileResult, _config: &ReportConfig) -> String {
    let fc = file.function_count;
    let avg_complexity = if fc > 0 {
        file.total_complexity as f64 / fc as f64
    } else {
        0.0
    };

    let rows = vec![
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
    ];

    let mut out = String::from("| Metric | Value |\n|--------|-------|\n");
    for row in rows {
        out.push_str(&row);
    }
    out.push('\n');
    out
}

fn format_function_table(functions: &[FunctionComplexity]) -> String {
    let mut out = String::from(
        "| Function | Lines | Line Range | Complexity | Nesting | Halstead Vol | Difficulty | Halstead Effort | Halstead Time |\n",
    );
    out.push_str("|----------|-------|------------|------------|---------|--------------|------------|-----------------|---------------|\n");
    for func in functions {
        out.push_str(&format_function_row(func));
    }
    out.push('\n');
    out
}

fn metric_row<T: std::fmt::Display>(name: &str, value: T) -> String {
    format!("| {} | {} |\n", name, value)
}

fn metric_row_f64(name: &str, value: f64, precision: usize) -> String {
    format!("| {} | {:.precision$} |\n", name, value)
}

fn format_function_row(func: &FunctionComplexity) -> String {
    format!(
        "| {} | {} | {}-{} | {} | {} | {:.2} | {:.2} | {:.2} | {:.2} |\n",
        func.name,
        func.lines,
        func.line_start,
        func.line_end,
        func.complexity,
        func.nesting_depth,
        func.halstead_volume,
        func.halstead_difficulty,
        func.halstead_effort,
        func.halstead_time
    )
}

fn format_clusters(clusters: &[DuplicateCluster]) -> String {
    let mut out = String::from("## Structural Duplication Candidates\n\n");
    for cluster in clusters {
        let n = cluster.instances.len();
        let suffix = if n == 1 { "" } else { "es" };
        out.push_str(&format!("### {} ({} exact match{})\n\n", cluster.name, n, suffix));
        out.push_str("| File | Line | Complexity | Lines | Halstead Volume | Halstead Difficulty |\n");
        out.push_str("|------|------|------------|-------|-----------------|---------------------|\n");
        for inst in &cluster.instances {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {:.2} | {:.2} |\n",
                inst.path.display(),
                inst.line_start,
                inst.complexity,
                inst.lines,
                inst.halstead_volume,
                inst.halstead_difficulty
            ));
        }
        out.push('\n');
    }
    out
}
