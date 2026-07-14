use crate::{
    FileResult,
    language::{
        LanguageAnalyzer, c::CAnalyzer, go::GoAnalyzer, java::JavaAnalyzer,
        javascript::JavaScriptAnalyzer, python::PythonAnalyzer, rust::RustAnalyzer,
        typescript::TypeScriptAnalyzer,
    },
};
use std::path::Path;
use walkdir::WalkDir;

static ANALYZERS: &[&dyn LanguageAnalyzer] = &[
    &RustAnalyzer,
    &PythonAnalyzer,
    &JavaScriptAnalyzer,
    &TypeScriptAnalyzer,
    &CAnalyzer,
    &GoAnalyzer,
    &JavaAnalyzer,
];

pub fn analyze_path(
    path: &Path,
    include_closures: bool,
    compute_clone_signature: bool,
) -> Result<Vec<FileResult>, std::io::Error> {
    if path.is_file() {
        Ok(vec![analyze_file(
            path,
            include_closures,
            compute_clone_signature,
        )?])
    } else if path.is_dir() {
        analyze_directory(path, include_closures, compute_clone_signature)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{} is not a file or directory", path.display()),
        ))
    }
}

fn analyze_directory(
    path: &Path,
    include_closures: bool,
    compute_clone_signature: bool,
) -> Result<Vec<FileResult>, std::io::Error> {
    let mut results = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: {e}");
                continue;
            }
        };
        let p = entry.path();
        if p.is_file() {
            results.push(analyze_file(p, include_closures, compute_clone_signature)?);
        }
    }
    Ok(results)
}

fn analyze_file(
    path: &Path,
    include_closures: bool,
    compute_clone_signature: bool,
) -> Result<FileResult, std::io::Error> {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Ok(build_error_result(path, 0, e.to_string())),
    };
    let total_lines = source.lines().count();

    for analyzer in ANALYZERS {
        if analyzer.can_analyze(path) {
            match analyzer.analyze_full(&source, include_closures, compute_clone_signature) {
                Ok(functions) => {
                    return Ok(FileResult::from_functions(
                        path,
                        total_lines,
                        functions,
                        analyzer.language_name(),
                    ));
                }
                Err(e) => return Ok(build_error_result(path, total_lines, e)),
            }
        }
    }

    Ok(build_empty_result(path, total_lines))
}

fn build_error_result(path: &Path, total_lines: usize, error: String) -> FileResult {
    FileResult {
        path: path.to_path_buf(),
        total_lines,
        error: Some(error),
        ..Default::default()
    }
}

fn build_empty_result(path: &Path, total_lines: usize) -> FileResult {
    FileResult {
        path: path.to_path_buf(),
        total_lines,
        ..Default::default()
    }
}
