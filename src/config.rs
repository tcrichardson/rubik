use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub introduction: Option<String>,
    pub project_summary_metrics: Option<Vec<String>>,
    pub file_summary_metrics: Option<Vec<String>>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            introduction: None,
            project_summary_metrics: None,
            file_summary_metrics: None,
        }
    }
}

#[derive(Deserialize, Default)]
struct RawConfig {
    introduction: Option<String>,
    project_summary: Option<RawSection>,
    file_summary: Option<RawSection>,
}

#[derive(Deserialize)]
struct RawSection {
    metrics: Option<Vec<String>>,
}

pub fn load_from_dir(dir: &Path) -> ReportConfig {
    let toml_path = dir.join("lede.toml");
    if !toml_path.exists() {
        return ReportConfig::default();
    }
    let content = match std::fs::read_to_string(&toml_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: could not read lede.toml: {} — using defaults", e);
            return ReportConfig::default();
        }
    };
    let raw: RawConfig = match toml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Warning: could not parse lede.toml: {} — using defaults", e);
            return ReportConfig::default();
        }
    };
    ReportConfig {
        introduction: raw.introduction,
        project_summary_metrics: raw.project_summary.and_then(|s| s.metrics),
        file_summary_metrics: raw.file_summary.and_then(|s| s.metrics),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_toml(dir: &std::path::PathBuf, content: &str) {
        fs::write(dir.join("lede.toml"), content).unwrap();
    }

    #[test]
    fn test_load_valid_config() {
        let dir = std::env::temp_dir().join("lede_test_config_valid");
        fs::create_dir_all(&dir).unwrap();
        write_toml(&dir, r#"
introduction = "Hello, this is a test report."

[project_summary]
metrics = ["files_analyzed", "total_functions"]

[file_summary]
metrics = ["total_complexity", "max_nesting_depth"]
"#);
        let cfg = load_from_dir(&dir);
        assert_eq!(cfg.introduction.as_deref(), Some("Hello, this is a test report."));
        assert_eq!(
            cfg.project_summary_metrics.as_deref(),
            Some(&["files_analyzed".to_string(), "total_functions".to_string()][..])
        );
        assert_eq!(
            cfg.file_summary_metrics.as_deref(),
            Some(&["total_complexity".to_string(), "max_nesting_depth".to_string()][..])
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_missing_file_returns_defaults() {
        let dir = std::env::temp_dir().join("lede_test_config_missing");
        fs::create_dir_all(&dir).unwrap();
        // Ensure no lede.toml exists
        let _ = fs::remove_file(dir.join("lede.toml"));
        let cfg = load_from_dir(&dir);
        assert!(cfg.introduction.is_none());
        assert!(cfg.project_summary_metrics.is_none());
        assert!(cfg.file_summary_metrics.is_none());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_malformed_toml_returns_defaults() {
        let dir = std::env::temp_dir().join("lede_test_config_malformed");
        fs::create_dir_all(&dir).unwrap();
        write_toml(&dir, "this is not valid toml }{][");
        let cfg = load_from_dir(&dir);
        assert!(cfg.introduction.is_none());
        assert!(cfg.project_summary_metrics.is_none());
        assert!(cfg.file_summary_metrics.is_none());
        fs::remove_dir_all(&dir).unwrap();
    }
}
