use crate::{
    AnalysisOutput, FileResult, SummaryStatistics, clones::ClonePair, config::ReportConfig,
    duplicates::DuplicateCluster, output::OutputFormatter,
};

pub struct JsonFormatter {
    pub config: ReportConfig,
}

impl OutputFormatter for JsonFormatter {
    fn format(
        &self,
        results: &[FileResult],
        clusters: &[DuplicateCluster],
        clones: &[ClonePair],
    ) -> String {
        let summary = SummaryStatistics::from_results(results);

        let output = AnalysisOutput {
            introduction: self.config.introduction.clone(),
            summary,
            files: results.to_vec(),
            clusters: if clusters.is_empty() {
                None
            } else {
                Some(clusters.to_vec())
            },
            clones: if clones.is_empty() {
                None
            } else {
                Some(clones.to_vec())
            },
        };

        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }
}
