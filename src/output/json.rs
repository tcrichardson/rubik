use crate::{AnalysisOutput, FileResult, SummaryStatistics, config::ReportConfig, duplicates::DuplicateCluster, output::OutputFormatter};

pub struct JsonFormatter {
    pub config: ReportConfig,
}

impl OutputFormatter for JsonFormatter {
    fn format(&self, results: &[FileResult], clusters: &[DuplicateCluster]) -> String {
        let summary = SummaryStatistics::from_results(results);

        let output = AnalysisOutput {
            summary,
            files: results.to_vec(),
            clusters: if clusters.is_empty() { None } else { Some(clusters.to_vec()) },
        };

        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }
}
