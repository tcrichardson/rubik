use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexity {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub lines: usize,
    pub complexity: u32,
    pub nesting_depth: u32,
    pub halstead_volume: f64,
    pub halstead_difficulty: f64,
    pub halstead_effort: f64,
    pub halstead_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    pub path: std::path::PathBuf,
    pub total_complexity: u32,
    pub total_lines: usize,
    pub function_count: usize,
    pub functions: Vec<FunctionComplexity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub max_nesting_depth: u32,
    pub avg_nesting_depth: f64,
    pub avg_halstead_volume: f64,
    pub avg_halstead_difficulty: f64,
    pub avg_halstead_effort: f64,
    pub avg_halstead_time: f64,
    pub max_complexity: u32,
    pub max_function_lines: usize,
    pub total_function_lines: usize,
    pub max_halstead_volume: f64,
    pub max_halstead_difficulty: f64,
    pub max_halstead_effort: f64,
    pub max_halstead_time: f64,
}

impl Default for FileResult {
    fn default() -> Self {
        Self {
            path: std::path::PathBuf::new(),
            total_complexity: 0,
            total_lines: 0,
            function_count: 0,
            functions: Vec::new(),
            error: None,
            max_nesting_depth: 0,
            avg_nesting_depth: 0.0,
            avg_halstead_volume: 0.0,
            avg_halstead_difficulty: 0.0,
            avg_halstead_effort: 0.0,
            avg_halstead_time: 0.0,
            max_complexity: 0,
            max_function_lines: 0,
            total_function_lines: 0,
            max_halstead_volume: 0.0,
            max_halstead_difficulty: 0.0,
            max_halstead_effort: 0.0,
            max_halstead_time: 0.0,
        }
    }
}

struct FileResultAccumulator {
    total_complexity: u32,
    total_function_lines: usize,
    max_complexity: u32,
    max_function_lines: usize,
    max_nesting_depth: u32,
    sum_nesting: f64,
    max_halstead_volume: f64,
    sum_halstead_volume: f64,
    max_halstead_difficulty: f64,
    sum_halstead_difficulty: f64,
    max_halstead_effort: f64,
    sum_halstead_effort: f64,
    max_halstead_time: f64,
    sum_halstead_time: f64,
}

impl FileResultAccumulator {
    fn new() -> Self {
        Self {
            total_complexity: 0,
            total_function_lines: 0,
            max_complexity: 0,
            max_function_lines: 0,
            max_nesting_depth: 0,
            sum_nesting: 0.0,
            max_halstead_volume: 0.0,
            sum_halstead_volume: 0.0,
            max_halstead_difficulty: 0.0,
            sum_halstead_difficulty: 0.0,
            max_halstead_effort: 0.0,
            sum_halstead_effort: 0.0,
            max_halstead_time: 0.0,
            sum_halstead_time: 0.0,
        }
    }

    fn add(&mut self, f: &FunctionComplexity) {
        self.total_complexity += f.complexity;
        self.total_function_lines += f.lines;
        self.max_complexity = self.max_complexity.max(f.complexity);
        self.max_function_lines = self.max_function_lines.max(f.lines);
        self.max_nesting_depth = self.max_nesting_depth.max(f.nesting_depth);
        self.sum_nesting += f.nesting_depth as f64;
        self.max_halstead_volume = self.max_halstead_volume.max(f.halstead_volume);
        self.sum_halstead_volume += f.halstead_volume;
        self.max_halstead_difficulty = self.max_halstead_difficulty.max(f.halstead_difficulty);
        self.sum_halstead_difficulty += f.halstead_difficulty;
        self.max_halstead_effort = self.max_halstead_effort.max(f.halstead_effort);
        self.sum_halstead_effort += f.halstead_effort;
        self.max_halstead_time = self.max_halstead_time.max(f.halstead_time);
        self.sum_halstead_time += f.halstead_time;
    }
}

impl FileResult {
    fn from_accumulator(
        path: &Path,
        total_lines: usize,
        functions: Vec<FunctionComplexity>,
        acc: &FileResultAccumulator,
    ) -> Self {
        let count = functions.len();
        let n = count as f64;
        Self {
            path: path.to_path_buf(),
            total_complexity: acc.total_complexity,
            total_lines,
            function_count: count,
            error: None,
            max_nesting_depth: acc.max_nesting_depth,
            avg_nesting_depth: acc.sum_nesting / n,
            max_complexity: acc.max_complexity,
            max_function_lines: acc.max_function_lines,
            total_function_lines: acc.total_function_lines,
            max_halstead_volume: acc.max_halstead_volume,
            avg_halstead_volume: acc.sum_halstead_volume / n,
            max_halstead_difficulty: acc.max_halstead_difficulty,
            avg_halstead_difficulty: acc.sum_halstead_difficulty / n,
            max_halstead_effort: acc.max_halstead_effort,
            avg_halstead_effort: acc.sum_halstead_effort / n,
            max_halstead_time: acc.max_halstead_time,
            avg_halstead_time: acc.sum_halstead_time / n,
            functions,
        }
    }

    pub fn from_functions(path: &Path, total_lines: usize, functions: Vec<FunctionComplexity>) -> Self {
        let count = functions.len();
        if count == 0 {
            return Self {
                path: path.to_path_buf(),
                total_lines,
                ..Default::default()
            };
        }

        let mut acc = FileResultAccumulator::new();
        for f in &functions {
            acc.add(f);
        }

        Self::from_accumulator(path, total_lines, functions, &acc)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStatistics {
    pub files_analyzed: usize,
    pub total_functions: usize,
    pub total_lines: usize,
    pub total_complexity: u32,
    pub avg_complexity_per_function: f64,
    pub max_nesting_depth: u32,
    pub avg_nesting_depth: f64,
    pub avg_halstead_volume: f64,
    pub avg_halstead_difficulty: f64,
    pub avg_halstead_effort: f64,
    pub avg_halstead_time: f64,
}

impl Default for SummaryStatistics {
    fn default() -> Self {
        Self {
            files_analyzed: 0,
            total_functions: 0,
            total_lines: 0,
            total_complexity: 0,
            avg_complexity_per_function: 0.0,
            max_nesting_depth: 0,
            avg_nesting_depth: 0.0,
            avg_halstead_volume: 0.0,
            avg_halstead_difficulty: 0.0,
            avg_halstead_effort: 0.0,
            avg_halstead_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOutput {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub introduction: Option<String>,
    pub summary: SummaryStatistics,
    pub files: Vec<FileResult>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub clusters: Option<Vec<crate::duplicates::DuplicateCluster>>,
}

fn safe_div(numerator: f64, denominator: f64) -> f64 {
    if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}

struct SummaryAccumulator {
    total_functions: usize,
    total_lines: usize,
    total_complexity: u32,
    max_nesting_depth: u32,
    weighted_sum_nesting: f64,
    weighted_sum_halstead_volume: f64,
    weighted_sum_halstead_difficulty: f64,
    weighted_sum_halstead_effort: f64,
    weighted_sum_halstead_time: f64,
}

impl SummaryAccumulator {
    fn new() -> Self {
        Self {
            total_functions: 0,
            total_lines: 0,
            total_complexity: 0,
            max_nesting_depth: 0,
            weighted_sum_nesting: 0.0,
            weighted_sum_halstead_volume: 0.0,
            weighted_sum_halstead_difficulty: 0.0,
            weighted_sum_halstead_effort: 0.0,
            weighted_sum_halstead_time: 0.0,
        }
    }

    fn add_file(&mut self, file: &FileResult) {
        let n = file.function_count as f64;
        self.total_functions += file.function_count;
        self.total_lines += file.total_lines;
        self.total_complexity += file.total_complexity;
        self.max_nesting_depth = self.max_nesting_depth.max(file.max_nesting_depth);
        self.weighted_sum_nesting += file.avg_nesting_depth * n;
        self.weighted_sum_halstead_volume += file.avg_halstead_volume * n;
        self.weighted_sum_halstead_difficulty += file.avg_halstead_difficulty * n;
        self.weighted_sum_halstead_effort += file.avg_halstead_effort * n;
        self.weighted_sum_halstead_time += file.avg_halstead_time * n;
    }
}

impl SummaryStatistics {
    fn from_accumulator(acc: SummaryAccumulator, files_analyzed: usize) -> Self {
        let n = acc.total_functions as f64;
        Self {
            files_analyzed,
            total_functions: acc.total_functions,
            total_lines: acc.total_lines,
            total_complexity: acc.total_complexity,
            avg_complexity_per_function: safe_div(acc.total_complexity as f64, n),
            max_nesting_depth: acc.max_nesting_depth,
            avg_nesting_depth: safe_div(acc.weighted_sum_nesting, n),
            avg_halstead_volume: safe_div(acc.weighted_sum_halstead_volume, n),
            avg_halstead_difficulty: safe_div(acc.weighted_sum_halstead_difficulty, n),
            avg_halstead_effort: safe_div(acc.weighted_sum_halstead_effort, n),
            avg_halstead_time: safe_div(acc.weighted_sum_halstead_time, n),
        }
    }

    pub fn from_results(results: &[FileResult]) -> Self {
        let successful: Vec<&FileResult> = results
            .iter()
            .filter(|f| f.error.is_none() && !f.functions.is_empty())
            .collect();

        if successful.is_empty() {
            return SummaryStatistics::default();
        }

        let total_files = successful.len();
        let mut acc = SummaryAccumulator::new();
        for file in &successful {
            acc.add_file(file);
        }

        SummaryStatistics::from_accumulator(acc, total_files)
    }
}

pub mod analyzer;
pub mod cognitive;
pub mod complexity;
pub mod config;
pub mod duplicates;
pub mod language;
pub mod output;

pub use analyzer::analyze_path;
