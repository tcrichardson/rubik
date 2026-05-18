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
