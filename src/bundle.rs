//! Walking an OKF bundle and deciding which files to lint.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Ignore rules loaded from a `.okfignore` file at the bundle root.
///
/// A bundle may legitimately contain non-concept Markdown (READMEs, specs,
/// contributor docs). OKF treats every non-reserved `.md` as a concept document
/// requiring `type`, so `.okfignore` lets a bundle exclude such files.
///
/// Supported patterns (one per line, `#` comments, blank lines ignored):
/// - `README.md`        exact path (relative to bundle root)
/// - `docs/`            a directory and everything under it
/// - `*.draft.md`       suffix match (pattern beginning with `*`)
pub struct Ignore {
    patterns: Vec<String>,
}

impl Ignore {
    pub fn load(root: &Path) -> Self {
        let mut patterns = Vec::new();
        if let Ok(text) = fs::read_to_string(root.join(".okfignore")) {
            for line in text.lines() {
                let l = line.trim();
                if l.is_empty() || l.starts_with('#') {
                    continue;
                }
                patterns.push(l.trim_start_matches('/').to_string());
            }
        }
        Ignore { patterns }
    }

    /// Whether the given bundle-relative path (using `/` separators) is ignored.
    pub fn matches(&self, rel: &str) -> bool {
        self.patterns.iter().any(|p| pattern_match(p, rel))
    }
}

fn pattern_match(pat: &str, rel: &str) -> bool {
    if let Some(suffix) = pat.strip_prefix('*') {
        return rel.ends_with(suffix);
    }
    let pat = pat.trim_end_matches('/');
    rel == pat || rel.starts_with(&format!("{pat}/"))
}

/// Recursively collect every `.md` file under `root` that should be linted.
///
/// Dot-files and dot-directories (`.git`, `.claude`, …) are always skipped, as
/// are anything matched by `.okfignore`. Returned paths are absolute-ish
/// (i.e. `root` joined), sorted for stable output.
pub fn collect_markdown(root: &Path, ignore: &Ignore) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk(root, root, ignore, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(root: &Path, dir: &Path, ignore: &Ignore, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if ignore.matches(&rel) {
            continue;
        }
        let ft = entry.file_type()?;
        if ft.is_dir() {
            walk(root, &path, ignore, out)?;
        } else if ft.is_file() && name.ends_with(".md") {
            out.push(path);
        }
    }
    Ok(())
}
