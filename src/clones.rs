use crate::{FileResult, FunctionComplexity};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Number of tokens per shingle when computing k-gram similarity.
const SHINGLE_SIZE: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClonePairSide {
    pub path: PathBuf,
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClonePair {
    pub a: ClonePairSide,
    pub b: ClonePairSide,
    pub similarity: f64,
}

#[derive(Debug, Clone)]
pub struct CloneConfig {
    pub similarity_threshold: f64,
    pub min_lines: usize,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            min_lines: 5,
        }
    }
}

/// Compares function bodies (via their pre-computed, normalized token
/// sequences) pairwise within each language and reports pairs whose
/// shingled-token Jaccard similarity meets `config.similarity_threshold`.
///
/// Functions below `config.min_lines`, functions without a computed token
/// sequence (i.e. their language doesn't yet support clone detection), and
/// cross-language pairs are excluded before any comparison work happens.
pub fn compute_clones(results: &[FileResult], config: &CloneConfig) -> Vec<ClonePair> {
    let candidates: Vec<(&FileResult, &FunctionComplexity)> = results
        .iter()
        .flat_map(|file| file.functions.iter().map(move |func| (file, func)))
        .filter(|(_, func)| func.lines >= config.min_lines && !func.clone_tokens.is_empty())
        .collect();

    let mut pairs = Vec::new();

    for i in 0..candidates.len() {
        let (file_a, func_a) = candidates[i];
        for &(file_b, func_b) in &candidates[i + 1..] {
            if file_a.language != file_b.language {
                continue;
            }
            if !length_prefilter(
                func_a.clone_tokens.len(),
                func_b.clone_tokens.len(),
                config.similarity_threshold,
            ) {
                continue;
            }
            let similarity = jaccard_similarity(&func_a.clone_tokens, &func_b.clone_tokens);
            if similarity >= config.similarity_threshold {
                pairs.push(ClonePair {
                    a: side(file_a, func_a),
                    b: side(file_b, func_b),
                    similarity,
                });
            }
        }
    }

    pairs.sort_by(|p1, p2| {
        p2.similarity
            .partial_cmp(&p1.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| p1.a.path.cmp(&p2.a.path))
            .then_with(|| p1.a.line_start.cmp(&p2.a.line_start))
    });

    pairs
}

fn side(file: &FileResult, func: &FunctionComplexity) -> ClonePairSide {
    ClonePairSide {
        path: file.path.clone(),
        name: func.name.clone(),
        line_start: func.line_start,
        line_end: func.line_end,
    }
}

/// Cheap length-based bound: two token sequences whose lengths differ enough
/// that even a perfect containment couldn't reach the threshold can never
/// meet a Jaccard similarity requirement, so skip the (relatively) expensive
/// shingle computation for them.
fn length_prefilter(len_a: usize, len_b: usize, threshold: f64) -> bool {
    if len_a == 0 || len_b == 0 {
        return false;
    }
    let (shorter, longer) = if len_a < len_b {
        (len_a, len_b)
    } else {
        (len_b, len_a)
    };
    (shorter as f64 / longer as f64) >= threshold
}

fn shingles(tokens: &[String]) -> HashSet<&[String]> {
    if tokens.len() < SHINGLE_SIZE {
        HashSet::from([tokens])
    } else {
        tokens.windows(SHINGLE_SIZE).collect()
    }
}

fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    let sa = shingles(a);
    let sb = shingles(b);
    let intersection = sa.intersection(&sb).count();
    let union = sa.union(&sb).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FunctionComplexity;

    fn make_func(name: &str, lines: usize, tokens: Vec<&str>) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_start: 1,
            line_end: lines,
            lines,
            complexity: 1,
            nesting_depth: 0,
            halstead_volume: 0.0,
            halstead_difficulty: 0.0,
            halstead_effort: 0.0,
            halstead_time: 0.0,
            clone_tokens: tokens.into_iter().map(|t| t.to_string()).collect(),
        }
    }

    fn make_file(
        path: &str,
        language: &'static str,
        functions: Vec<FunctionComplexity>,
    ) -> FileResult {
        crate::FileResult::from_functions(std::path::Path::new(path), 100, functions, language)
    }

    fn identical_tokens() -> Vec<&'static str> {
        vec![
            "block",
            "if_expression",
            "<ID>",
            "block",
            "<LIT>",
            "block",
            "<ID>",
            "block",
            "<LIT>",
        ]
    }

    fn near_miss_tokens() -> Vec<&'static str> {
        // One token swapped relative to `identical_tokens`, still similar
        // enough to clear a moderate threshold.
        vec![
            "block",
            "if_expression",
            "<ID>",
            "block",
            "<LIT>",
            "block",
            "<ID>",
            "block",
            "foo",
        ]
    }

    fn unrelated_tokens() -> Vec<&'static str> {
        vec![
            "match_expression",
            "arm",
            "call_expression",
            "return_expression",
            "identifier",
        ]
    }

    #[test]
    fn no_clones_below_threshold() {
        let files = vec![
            make_file(
                "a.rs",
                "Rust",
                vec![make_func("foo", 10, identical_tokens())],
            ),
            make_file(
                "b.rs",
                "Rust",
                vec![make_func("bar", 10, unrelated_tokens())],
            ),
        ];
        let config = CloneConfig {
            similarity_threshold: 0.85,
            min_lines: 5,
        };
        let pairs = compute_clones(&files, &config);
        assert!(pairs.is_empty());
    }

    #[test]
    fn clear_clone_pair_above_threshold() {
        let files = vec![
            make_file(
                "a.rs",
                "Rust",
                vec![make_func("foo", 10, identical_tokens())],
            ),
            make_file(
                "b.rs",
                "Rust",
                vec![make_func("renamed_foo", 10, identical_tokens())],
            ),
        ];
        let config = CloneConfig {
            similarity_threshold: 0.85,
            min_lines: 5,
        };
        let pairs = compute_clones(&files, &config);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].similarity, 1.0);
        assert_eq!(pairs[0].a.name, "foo");
        assert_eq!(pairs[0].b.name, "renamed_foo");
    }

    #[test]
    fn near_miss_clone_detected_at_moderate_threshold() {
        let files = vec![
            make_file(
                "a.rs",
                "Rust",
                vec![make_func("foo", 10, identical_tokens())],
            ),
            make_file(
                "b.rs",
                "Rust",
                vec![make_func("bar", 10, near_miss_tokens())],
            ),
        ];
        let config = CloneConfig {
            similarity_threshold: 0.5,
            min_lines: 5,
        };
        let pairs = compute_clones(&files, &config);
        assert_eq!(pairs.len(), 1);
        assert!(pairs[0].similarity > 0.5 && pairs[0].similarity < 1.0);
    }

    #[test]
    fn exclusion_below_minimum_size() {
        let files = vec![
            make_file(
                "a.rs",
                "Rust",
                vec![make_func("foo", 2, identical_tokens())],
            ),
            make_file(
                "b.rs",
                "Rust",
                vec![make_func("bar", 2, identical_tokens())],
            ),
        ];
        let config = CloneConfig {
            similarity_threshold: 0.85,
            min_lines: 5,
        };
        let pairs = compute_clones(&files, &config);
        assert!(
            pairs.is_empty(),
            "functions below min_lines must be excluded before comparison"
        );
    }

    #[test]
    fn exclusion_of_cross_language_pairs() {
        let files = vec![
            make_file(
                "a.rs",
                "Rust",
                vec![make_func("foo", 10, identical_tokens())],
            ),
            make_file(
                "b.py",
                "Python",
                vec![make_func("foo", 10, identical_tokens())],
            ),
        ];
        let config = CloneConfig {
            similarity_threshold: 0.85,
            min_lines: 5,
        };
        let pairs = compute_clones(&files, &config);
        assert!(
            pairs.is_empty(),
            "cross-language pairs must never be reported"
        );
    }

    #[test]
    fn exclusion_of_functions_without_a_token_sequence() {
        // Simulates a language without clone-detection support: no tokens
        // were ever computed, so the function must never be compared.
        let files = vec![
            make_file("a.js", "JS", vec![make_func("foo", 10, vec![])]),
            make_file("b.js", "JS", vec![make_func("bar", 10, vec![])]),
        ];
        let config = CloneConfig::default();
        let pairs = compute_clones(&files, &config);
        assert!(pairs.is_empty());
    }

    #[test]
    fn length_mismatched_pairs_are_prefiltered_out() {
        let short = vec!["block", "<ID>"];
        let long: Vec<&str> = std::iter::repeat_n("block", 50).collect();
        let files = vec![
            make_file("a.rs", "Rust", vec![make_func("foo", 10, short)]),
            make_file("b.rs", "Rust", vec![make_func("bar", 10, long)]),
        ];
        let config = CloneConfig {
            similarity_threshold: 0.85,
            min_lines: 5,
        };
        let pairs = compute_clones(&files, &config);
        assert!(pairs.is_empty());
    }

    #[test]
    fn pairs_sorted_by_similarity_descending() {
        let files = vec![
            make_file(
                "a.rs",
                "Rust",
                vec![
                    make_func("foo", 10, identical_tokens()),
                    make_func("far", 10, near_miss_tokens()),
                ],
            ),
            make_file(
                "b.rs",
                "Rust",
                vec![
                    make_func("bar", 10, near_miss_tokens()),
                    make_func("baz", 10, identical_tokens()),
                ],
            ),
        ];
        let config = CloneConfig {
            similarity_threshold: 0.5,
            min_lines: 5,
        };
        let pairs = compute_clones(&files, &config);
        assert!(pairs.len() >= 2);
        for i in 1..pairs.len() {
            assert!(pairs[i - 1].similarity >= pairs[i].similarity);
        }
        // The exact-match pair should be sorted first.
        assert_eq!(pairs[0].similarity, 1.0);
    }
}
