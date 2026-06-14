use crate::{
    FileResult, FunctionComplexity, SummaryStatistics, config::ReportConfig,
    duplicates::DuplicateCluster, output::OutputFormatter,
};

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

fn format_summary(summary: &SummaryStatistics, config: &ReportConfig) -> String {
    let rows: Vec<String> = match &config.project_summary_metrics {
        None => vec![
            metric_row("Files Analyzed", summary.files_analyzed),
            metric_row("Total Functions", summary.total_functions),
            metric_row("Total Lines", summary.total_lines),
            metric_row("Total Complexity", summary.total_complexity),
            metric_row_f64(
                "Avg Complexity / Function",
                summary.avg_complexity_per_function,
                2,
            ),
            metric_row("Max Nesting Depth", summary.max_nesting_depth),
            metric_row_f64("Avg Nesting Depth", summary.avg_nesting_depth, 2),
            metric_row_f64("Avg Halstead Volume", summary.avg_halstead_volume, 2),
            metric_row_f64(
                "Avg Halstead Difficulty",
                summary.avg_halstead_difficulty,
                2,
            ),
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

fn format_file(file: &FileResult, config: &ReportConfig) -> String {
    if let Some(ref err) = file.error {
        return format!("**{}**: ERROR: {}\n\n", file.path.display(), err);
    }
    if file.functions.is_empty() {
        return String::new();
    }

    let mut out = format!("### {}\n\n", file.path.display());
    out.push_str("#### File Summary\n\n");
    out.push_str(&format_file_summary(file, config));
    out.push_str(&format_function_table(&file.functions));
    out
}

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

type ProjectFmt = fn(&SummaryStatistics) -> String;

static PROJECT_SUMMARY_FORMATTERS: &[(&str, ProjectFmt)] = &[
    ("files_analyzed", |s| {
        metric_row("Files Analyzed", s.files_analyzed)
    }),
    ("total_functions", |s| {
        metric_row("Total Functions", s.total_functions)
    }),
    ("total_lines", |s| metric_row("Total Lines", s.total_lines)),
    ("total_complexity", |s| {
        metric_row("Total Complexity", s.total_complexity)
    }),
    ("avg_complexity_per_function", |s| {
        metric_row_f64(
            "Avg Complexity / Function",
            s.avg_complexity_per_function,
            2,
        )
    }),
    ("max_nesting_depth", |s| {
        metric_row("Max Nesting Depth", s.max_nesting_depth)
    }),
    ("avg_nesting_depth", |s| {
        metric_row_f64("Avg Nesting Depth", s.avg_nesting_depth, 2)
    }),
    ("avg_halstead_volume", |s| {
        metric_row_f64("Avg Halstead Volume", s.avg_halstead_volume, 2)
    }),
    ("avg_halstead_difficulty", |s| {
        metric_row_f64("Avg Halstead Difficulty", s.avg_halstead_difficulty, 2)
    }),
    ("avg_halstead_effort", |s| {
        metric_row_f64("Avg Halstead Effort", s.avg_halstead_effort, 2)
    }),
    ("avg_halstead_time", |s| {
        metric_row_f64("Avg Halstead Time", s.avg_halstead_time, 2)
    }),
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

type FileFmt = fn(&FileResult, f64) -> String;

static FILE_SUMMARY_FORMATTERS: &[(&str, FileFmt)] = &[
    ("total_functions", |f, _| {
        metric_row("Total Functions", f.function_count)
    }),
    ("total_lines", |f, _| {
        metric_row("Total Lines", f.total_lines)
    }),
    ("total_function_lines", |f, _| {
        metric_row("Total Function Lines", f.total_function_lines)
    }),
    ("total_complexity", |f, _| {
        metric_row("Total Complexity", f.total_complexity)
    }),
    ("avg_complexity_per_function", |_, a| {
        metric_row_f64("Avg Complexity / Function", a, 2)
    }),
    ("max_complexity", |f, _| {
        metric_row("Max Complexity", f.max_complexity)
    }),
    ("max_nesting_depth", |f, _| {
        metric_row("Max Nesting Depth", f.max_nesting_depth)
    }),
    ("avg_nesting_depth", |f, _| {
        metric_row_f64("Avg Nesting Depth", f.avg_nesting_depth, 2)
    }),
    ("max_function_lines", |f, _| {
        metric_row("Max Function Lines", f.max_function_lines)
    }),
    ("avg_halstead_volume", |f, _| {
        metric_row_f64("Avg Halstead Volume", f.avg_halstead_volume, 2)
    }),
    ("max_halstead_volume", |f, _| {
        metric_row_f64("Max Halstead Volume", f.max_halstead_volume, 2)
    }),
    ("avg_halstead_difficulty", |f, _| {
        metric_row_f64("Avg Halstead Difficulty", f.avg_halstead_difficulty, 2)
    }),
    ("max_halstead_difficulty", |f, _| {
        metric_row_f64("Max Halstead Difficulty", f.max_halstead_difficulty, 2)
    }),
    ("avg_halstead_effort", |f, _| {
        metric_row_f64("Avg Halstead Effort", f.avg_halstead_effort, 2)
    }),
    ("max_halstead_effort", |f, _| {
        metric_row_f64("Max Halstead Effort", f.max_halstead_effort, 2)
    }),
    ("avg_halstead_time", |f, _| {
        metric_row_f64("Avg Halstead Time", f.avg_halstead_time, 2)
    }),
    ("max_halstead_time", |f, _| {
        metric_row_f64("Max Halstead Time", f.max_halstead_time, 2)
    }),
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

fn format_clusters(clusters: &[DuplicateCluster]) -> String {
    let mut out = String::from("## Structural Duplication Candidates\n\n");
    for cluster in clusters {
        let n = cluster.instances.len();
        let suffix = if n == 1 { "" } else { "es" };
        out.push_str(&format!(
            "### {} ({} exact match{})\n\n",
            cluster.name, n, suffix
        ));
        out.push_str(
            "| File | Line | Complexity | Lines | Halstead Volume | Halstead Difficulty |\n",
        );
        out.push_str(
            "|------|------|------------|-------|-----------------|---------------------|\n",
        );
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
