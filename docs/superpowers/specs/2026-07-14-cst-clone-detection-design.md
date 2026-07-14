# CST-Based Clone Detection Design

**Date:** 2026-07-14
**Topic:** Structural (Type-1/2/3) function clone detection using tree-sitter syntax trees

---

## Problem Statement

Polygraph already detects one narrow kind of duplication: `src/duplicates.rs` flags functions that share the exact same name *and* have byte-for-byte identical complexity/nesting/Halstead metrics across files. Real copy-pasted code almost never stays this identical for long — a variable gets renamed, a literal gets tweaked, the function itself gets renamed, or a statement gets added or removed during a later edit. All of these break the existing exact-match check, so the tool misses the overwhelming majority of real-world clones: code that started as a copy-paste and has since drifted slightly. Users are left without visibility into a common source of maintenance burden — the same logic living in multiple places, edited inconsistently, with bugs fixed in one copy but not its siblings.

Note explicitly what this does *not* solve: functions that are *behaviorally* equivalent but structurally unrelated (e.g., two functions that each end up issuing an `UPDATE` against the same database table, written via completely different code paths) cannot be detected by comparing syntax trees — by definition, such "semantic clones" (Type-4) can have arbitrarily different structure. That is a different problem requiring different techniques (e.g., heuristic tagging of call targets/literals) and is out of scope here.

## Solution

Add an opt-in clone detector that compares function bodies using their tree-sitter concrete syntax trees, tolerant of renamed identifiers, changed literals, and small structural edits (Type-1/2/3 clones). Each function's body is walked into a normalized token sequence — call/method target names are preserved (they're the strongest signal that two functions do related things), while local variable names and literal values are replaced with generic placeholders. Two functions are compared via shingled token-sequence similarity (Jaccard/Dice over k-grams) and reported as a clone pair when their similarity meets a configurable threshold.

This ships as a new, separate, opt-in feature (`--clones` flag) that does not alter the existing exact-match duplicate-cluster feature. It covers Rust and Python for the first release, proving the per-language normalization approach generalizes across a statically- and dynamically-typed language before extending to the other five supported grammars (JavaScript, TypeScript, C, Go, Java).

## User Stories

1. As a maintainer of a Rust codebase, I want to see which functions have near-identical bodies to other functions — even under a different name — so that I can identify unintentional copy-paste duplication worth consolidating.
2. As a maintainer, I want clone detection to tolerate renamed local variables and changed literal values, so that a trivial rename doesn't mask a real duplicate.
3. As a maintainer, I want the detector to preserve call/method target names during comparison, so that two functions with a similar control-flow shape but calling unrelated APIs are not falsely flagged as clones.
4. As a maintainer, I want each reported clone match to include a similarity score, so that I can judge how confident to be before acting on it.
5. As a maintainer, I want trivial functions (one-line getters, tiny wrappers) excluded from comparison entirely, so that boilerplate noise doesn't dominate the results.
6. As a maintainer, I want the similarity threshold configurable in `polygraph.toml`, so that I can tune sensitivity to my codebase's style without recompiling the tool.
7. As a maintainer, I want to override the threshold via a CLI flag, so that I can experiment quickly without editing a config file.
8. As a maintainer, I want the minimum function size for comparison to also be configurable, so that I can tune what counts as "trivial" for my codebase.
9. As a CLI user, I want clone detection to be strictly opt-in via a flag, so that my existing workflows and output are unaffected unless I explicitly ask for this feature.
10. As a CLI user, I want clone detection to work on both Rust and Python source in this release, so that I can use the feature on either currently-supported language.
11. As a maintainer, I want clone pairs reported individually with their similarity score rather than merged into transitive clusters, so that I see exactly what was measured and the tool doesn't over-group loosely related functions.
12. As a maintainer, I want clone comparison scoped within a single language (Rust-to-Rust, Python-to-Python), so that results aren't polluted by spurious cross-language matches the tool has no shared vocabulary to judge.
13. As a maintainer, I want comparison based on function body only, not the signature, so that clones are still found even when someone changed a function's parameter list without changing its logic.
14. As a maintainer, I want this feature to coexist with the existing exact-metric duplicate-cluster feature without changing its behavior or output, so existing consumers of that feature are unaffected.
15. As a CLI user, I want clone results available in JSON, markdown, and pretty output, consistent with the rest of the tool, so that I can pipe results into other tooling or read them directly.
16. As a maintainer analyzing a larger codebase, I want obviously-mismatched-length function pairs skipped before the more expensive similarity computation, so that the tool stays reasonably fast without needing indexing infrastructure.
17. As a future contributor, I want the per-language token-normalization logic to be pluggable in the same style as the existing `extract_name` hook, so that extending clone detection to the remaining five languages is a matter of implementing one hook per language, not redesigning the algorithm.
18. As a maintainer, I want it documented clearly that this feature does not detect semantic/behavioral clones (e.g., two differently-written functions that both update the same database table), so that I don't rely on it for a class of problem it isn't designed to solve.
19. As a CLI user, I want running without `--clones` to produce byte-identical output and performance to today, so that adopting this feature carries zero cost until I opt in.

## Implementation Decisions

**Clone type scope:** Type-1 (exact), Type-2 (renamed), and Type-3 (near-miss, small edits) clones only. Type-4 semantic/behavioral clones are explicitly out of scope and not attempted.

**Language scope for this release:** Rust and Python only. The remaining five supported grammars (JavaScript, TypeScript, C, Go, Java) are deferred to a follow-up once this design is validated. Comparisons never cross languages.

**Similarity algorithm:** Each function body is walked into a normalized token sequence (node kind for most nodes; call/method target identifiers preserved as-is; local variable names and literal values replaced with generic placeholders). Sequences are compared via k-gram shingling with Jaccard/Dice similarity, producing a score in `[0.0, 1.0]`. A pair is reported when the score meets or exceeds a configurable threshold. Function signatures (parameter list, return type) are excluded from the token sequence — only body statements are compared.

**Per-language normalization hook:** `LanguageConfig` (or the `LanguageAnalyzer` trait, mirroring the existing `extract_name: fn(Node, &str) -> String` field) gains a new per-language hook responsible for classifying each node during the walk as "preserve" (e.g. call/method target identifier) or "normalize" (local binding, literal). Rust and Python each implement this hook for their grammar's node kinds. Languages without an implementation simply don't participate in clone detection yet.

**Data plumbing (mirrors the existing `include_closures` pattern):**
- A new `compute_clone_signature: bool` parameter threads from a new CLI flag through `analyze_path` → `analyze_file` → `LanguageAnalyzer::analyze()`, exactly like `include_closures` does today.
- When enabled, each `FunctionComplexity` gets its shingle data populated into a new field marked `#[serde(skip)]` — present in memory for the pure clone-computation step, absent from JSON output, and not computed at all when the flag is off (zero cost, zero schema change when `--clones` isn't used).
- `FileResult` gains a `language: &'static str` field, populated from the matching analyzer's existing `language_name()` method, so same-language grouping doesn't require re-inferring language from file extensions.

**Minimum size filter:** Functions below a configurable minimum (default ~5 lines / ~20 tokens) are excluded from comparison entirely — not merely filtered from the report, excluded before any comparison work happens.

**Comparison strategy:** Brute-force pairwise comparison among same-language functions that pass the minimum-size filter, with a cheap pre-filter that skips pairs whose token-sequence lengths differ beyond what could possibly meet the similarity threshold. No indexing/LSH/bucketing infrastructure for this release.

**New module `src/clones.rs`** (parallel to and independent of `src/duplicates.rs`): pure computation, no I/O.
- `pub struct ClonePair { pub a: ClonePairSide, pub b: ClonePairSide, pub similarity: f64 }`
- `pub struct ClonePairSide { pub path: PathBuf, pub name: String, pub line_start: usize, pub line_end: usize }`
- `pub struct CloneConfig { pub similarity_threshold: f64, pub min_lines: usize }` with a `Default` giving threshold ≈ 0.85 and `min_lines` ≈ 5.
- `pub fn compute_clones(results: &[FileResult], config: &CloneConfig) -> Vec<ClonePair>`

**Configuration:** `similarity_threshold` and `min_lines` (or an equivalent min-token setting) are configurable via `polygraph.toml` (extending `ReportConfig`/`config.rs` following its existing load pattern) and overridable via CLI flags on top of the config file, consistent with how `--config` already layers over `polygraph.toml`.

**CLI activation:** Off by default. A new `--clones` flag enables clone-signature computation and clone-pair reporting for the run. Without the flag, output and performance are unchanged from today.

**Relationship to existing duplicates feature:** Purely additive. `src/duplicates.rs` and its exact-metric-equality `DuplicateCluster` output are untouched — no shared code path, no behavior change, no output change when `--clones` is not passed.

**Output integration:** Clone pairs are reported as a new section/field per formatter (JSON, markdown, pretty), following the same wiring pattern the existing `clusters` output uses (`main.rs` computes the result once, passes it into the formatter alongside existing results). Reported as flat pairs with scores — not grouped into transitive clusters — since similarity is not guaranteed to be transitive and grouping risks pulling loosely-related functions together under one heading.

**No special-casing for test files or generated code:** Clone detection analyzes whatever path is given, consistent with how the rest of polygraph behaves today. Users who want to exclude tests point the tool at a narrower path.

## Testing Decisions

Tests should verify observable behavior (does this pair of functions get reported as a clone at a given threshold, does this trivial function get excluded, does turning the flag off leave output unchanged) rather than internal token-sequence representations.

- **`src/clones.rs` unit tests** (primary test surface, mirroring `duplicates.rs`'s `make_func`/`make_file` fixture-building pattern): build `FunctionComplexity` fixtures with hand-set shingle data and assert `compute_clones` produces the expected `ClonePair`s at given thresholds — no parsing involved. Cover at least: no clones below threshold, a clear clone pair above threshold, exclusion of pairs below the minimum size, exclusion of cross-language pairs, and pairs whose token-sequence lengths differ too much being pre-filtered out.
- **Per-language normalization tests** (in `src/language/rust.rs` and `src/language/python.rs`, alongside existing per-language tests): parse small fixture snippets and assert the produced token sequence preserves call/method target names while normalizing local identifiers and literals as expected. Follow the existing per-language fixture convention under `tests/fixtures/`.
- **Integration test**: add a `tests/fixtures/clones/` directory with a pair of Rust files (and a pair of Python files) containing a deliberately renamed/edited near-miss clone, run `analyze_path` with clone-signature computation enabled, and assert `compute_clones` reports the expected pair. Also assert that running without `--clones` produces the current, unchanged output (regression guard for the "zero cost when off" requirement).
- **CLI/config tests**: verify `--clones` toggles the feature on, verify threshold/min-size are read from `polygraph.toml` and are overridable via CLI flags, following the existing test patterns in `config.rs`.

## Out of Scope

- Type-4 semantic/behavioral clone detection (e.g., two differently-implemented functions that both update the same database table). This requires different techniques (heuristic tagging of literals/call targets, data flow analysis) and is not attempted by this feature.
- Cross-language clone detection. Comparisons never cross a language boundary; no shared/normalized vocabulary across grammars is built.
- JavaScript, TypeScript, C, Go, and Java support. Deferred until the Rust/Python implementation is validated.
- Indexing, bucketing, or LSH-based scaling infrastructure. Brute-force pairwise comparison (with a cheap length pre-filter) is accepted for this release.
- Transitive clustering of clone pairs into groups. Results are reported as individual pairs with scores.
- Special-casing of test files, generated code, or vendored code paths.
- Any change to the existing `src/duplicates.rs` exact-metric-equality feature or its output.
- Any change to output/behavior when `--clones` is not passed.

## Further Notes

If a future project wants to chase the original Type-4 motivating example (detecting functions that perform the same semantic action, like updating the same database table, despite being implemented completely differently), the callee-name-preservation decision in this design leaves a useful building block in place: the preserved call/method target names collected here are a natural input to a follow-up heuristic tagging feature (e.g., clustering functions by the set of APIs/literals they invoke, independent of structural similarity). That is a distinct feature with different testing/output needs and is not part of this spec.
