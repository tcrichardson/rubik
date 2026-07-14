use crate::{FileResult, FunctionComplexity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCluster {
    pub name: String,
    pub instances: Vec<ClusterInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInstance {
    pub path: std::path::PathBuf,
    pub line_start: usize,
    pub complexity: u32,
    pub lines: usize,
    pub nesting_depth: u32,
    pub halstead_volume: f64,
    pub halstead_difficulty: f64,
}

pub fn compute_duplicates(results: &[FileResult]) -> Vec<DuplicateCluster> {
    let mut by_name: HashMap<String, Vec<(&FileResult, &FunctionComplexity)>> = HashMap::new();

    for file in results {
        for func in &file.functions {
            by_name
                .entry(func.name.clone())
                .or_default()
                .push((file, func));
        }
    }

    let mut clusters = Vec::new();

    for (name, instances) in by_name {
        if instances.len() < 2 {
            continue;
        }

        let first = instances[0].1;
        let all_match = instances.iter().all(|(_, func)| {
            func.lines == first.lines
                && func.complexity == first.complexity
                && func.nesting_depth == first.nesting_depth
                && func.halstead_volume == first.halstead_volume
                && func.halstead_difficulty == first.halstead_difficulty
        });

        if all_match {
            let mut cluster_instances: Vec<ClusterInstance> = instances
                .iter()
                .map(|(file, func)| ClusterInstance {
                    path: file.path.clone(),
                    line_start: func.line_start,
                    complexity: func.complexity,
                    lines: func.lines,
                    nesting_depth: func.nesting_depth,
                    halstead_volume: func.halstead_volume,
                    halstead_difficulty: func.halstead_difficulty,
                })
                .collect();

            cluster_instances.sort_by(|a, b| a.path.cmp(&b.path));

            clusters.push(DuplicateCluster {
                name,
                instances: cluster_instances,
            });
        }
    }

    clusters.sort_by(|a, b| {
        b.instances
            .len()
            .cmp(&a.instances.len())
            .then_with(|| a.name.cmp(&b.name))
    });

    clusters
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FunctionComplexity;

    fn make_func(
        name: &str,
        lines: usize,
        complexity: u32,
        nesting: u32,
        volume: f64,
        difficulty: f64,
    ) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_start: 1,
            line_end: lines,
            lines,
            complexity,
            nesting_depth: nesting,
            halstead_volume: volume,
            halstead_difficulty: difficulty,
            halstead_effort: 0.0,
            halstead_time: 0.0,
            clone_tokens: Vec::new(),
        }
    }

    fn make_file(path: &str, functions: Vec<FunctionComplexity>) -> FileResult {
        crate::FileResult::from_functions(std::path::Path::new(path), 100, functions, "Rust")
    }

    #[test]
    fn test_no_duplicates() {
        let files = vec![
            make_file("a.rs", vec![make_func("foo", 10, 2, 1, 100.0, 0.0)]),
            make_file("b.rs", vec![make_func("bar", 10, 2, 1, 100.0, 0.0)]),
        ];
        let clusters = compute_duplicates(&files);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_exact_match_cluster() {
        let files = vec![
            make_file("a.rs", vec![make_func("collect", 28, 3, 1, 469.13, 0.0)]),
            make_file("b.rs", vec![make_func("collect", 28, 3, 1, 469.13, 0.0)]),
        ];
        let clusters = compute_duplicates(&files);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].name, "collect");
        assert_eq!(clusters[0].instances.len(), 2);
    }

    #[test]
    fn test_same_name_different_metrics() {
        let files = vec![
            make_file("a.rs", vec![make_func("collect", 28, 3, 1, 469.13, 0.0)]),
            make_file("b.rs", vec![make_func("collect", 30, 3, 1, 469.13, 0.0)]),
        ];
        let clusters = compute_duplicates(&files);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_sorting_behavior() {
        let files = vec![
            make_file("b.rs", vec![make_func("alpha", 10, 2, 1, 100.0, 0.0)]),
            make_file("a.rs", vec![make_func("alpha", 10, 2, 1, 100.0, 0.0)]),
            make_file("c.rs", vec![make_func("beta", 10, 2, 1, 100.0, 0.0)]),
            make_file("d.rs", vec![make_func("beta", 10, 2, 1, 100.0, 0.0)]),
            make_file("e.rs", vec![make_func("beta", 10, 2, 1, 100.0, 0.0)]),
        ];
        let clusters = compute_duplicates(&files);
        assert_eq!(clusters.len(), 2);
        // Clusters sorted by instance count descending, then name ascending
        assert_eq!(clusters[0].name, "beta");
        assert_eq!(clusters[0].instances.len(), 3);
        assert_eq!(clusters[1].name, "alpha");
        assert_eq!(clusters[1].instances.len(), 2);
        // Instances within cluster sorted by path ascending
        assert_eq!(clusters[1].instances[0].path.as_os_str(), "a.rs");
        assert_eq!(clusters[1].instances[1].path.as_os_str(), "b.rs");
    }

    #[test]
    fn test_multiple_distinct_clusters() {
        let files = vec![
            make_file("a.rs", vec![make_func("foo", 10, 2, 1, 100.0, 0.0)]),
            make_file("b.rs", vec![make_func("foo", 10, 2, 1, 100.0, 0.0)]),
            make_file("c.rs", vec![make_func("bar", 20, 4, 2, 200.0, 1.0)]),
            make_file("d.rs", vec![make_func("bar", 20, 4, 2, 200.0, 1.0)]),
        ];
        let clusters = compute_duplicates(&files);
        assert_eq!(clusters.len(), 2);
        let names: Vec<&str> = clusters.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"foo"));
        assert!(names.contains(&"bar"));
    }

    #[test]
    fn test_three_instances() {
        let files = vec![
            make_file("a.rs", vec![make_func("collect", 28, 3, 1, 469.13, 5.0)]),
            make_file("b.rs", vec![make_func("collect", 28, 3, 1, 469.13, 5.0)]),
            make_file("c.rs", vec![make_func("collect", 28, 3, 1, 469.13, 5.0)]),
        ];
        let clusters = compute_duplicates(&files);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].name, "collect");
        assert_eq!(clusters[0].instances.len(), 3);
    }
}
