use crate::{FileResult, config::ReportConfig, duplicates::DuplicateCluster, output::OutputFormatter};
use comfy_table::{Table, ContentArrangement};

pub struct PrettyFormatter {
    pub config: ReportConfig,
}

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

fn format_file_entry(file: &FileResult) -> String {
    let mut out = String::new();
    if let Some(ref err) = file.error {
        out.push_str(&format!("{}: ERROR: {}\n", file.path.display(), err));
        return out;
    }
    if file.functions.is_empty() {
        return out;
    }

    out.push_str(&format!(
        "{} (total complexity: {}, total lines: {}, functions: {})\n",
        file.path.display(),
        file.total_complexity,
        file.total_lines,
        file.function_count
    ));

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Function", "Lines", "Line Range", "Complexity"]);
    for func in &file.functions {
        table.add_row(vec![
            &func.name,
            &func.lines.to_string(),
            &format!("{}-{}", func.line_start, func.line_end),
            &func.complexity.to_string(),
        ]);
    }
    out.push_str(&table.to_string());
    out.push('\n');
    out
}
