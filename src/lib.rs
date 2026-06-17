//! okflint — a dependency-free linter for Open Knowledge Format (OKF) v0.1 bundles.
//!
//! OKF v0.1 represents knowledge as a directory of Markdown files with YAML
//! frontmatter (see <https://github.com/GoogleCloudPlatform/knowledge-catalog>).
//! This crate validates a bundle against the spec's MUST rules (reported as
//! errors) and its SHOULD / tolerated rules (reported as warnings).

mod bundle;
mod checks;
mod diagnostic;
mod parse;

pub use diagnostic::{Diagnostic, Severity};

use std::fs;
use std::io;
use std::path::Path;

/// Lint the OKF bundle rooted at `root`. Returns all findings sorted by path
/// and line. Returns an `io::Error` only for problems reading the bundle root
/// itself (missing directory, etc.); per-file read errors become diagnostics.
pub fn lint(root: &Path) -> io::Result<Vec<Diagnostic>> {
    let meta = fs::metadata(root)?;
    if !meta.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a directory", root.display()),
        ));
    }

    let ignore = bundle::Ignore::load(root);
    let files = bundle::collect_markdown(root, &ignore)?;
    // All bundle paths, for O(1) link-existence checks (no stat per link).
    let all_files = bundle::collect_all_files(root)?;

    let mut diags = Vec::new();
    for path in &files {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                diags.push(Diagnostic::error(&rel, None, "OKF000", format!("could not read file: {e}")));
                continue;
            }
        };

        let doc = parse::parse(&content);
        let basename = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();

        if doc.has_frontmatter && !doc.fm_closed {
            diags.push(Diagnostic::error(&rel, Some(1), "OKF003", "frontmatter opened with `---` but never closed"));
        }

        match basename.as_str() {
            "index.md" => checks::check_index_no_frontmatter(&rel, &doc, &mut diags),
            "log.md" => checks::check_log_headings(&rel, &content, &mut diags),
            _ => checks::check_concept_type(&rel, &doc, &mut diags),
        }

        checks::check_timestamp(&rel, &doc, &mut diags);
        checks::check_links(&all_files, &rel, &content, &mut diags);
    }

    diags.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    Ok(diags)
}

/// Convenience summary: number of errors and warnings in a result set.
pub fn counts(diags: &[Diagnostic]) -> (usize, usize) {
    let errors = diags.iter().filter(|d| d.severity == Severity::Error).count();
    (errors, diags.len() - errors)
}
