use std::fmt;

/// Severity of a lint finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A violation of an OKF MUST rule. Causes a non-zero exit code.
    Error,
    /// A violation of an OKF SHOULD / recommendation, or a tolerated issue
    /// (e.g. a broken link, which OKF consumers must tolerate).
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
        }
    }
}

/// A single lint finding, anchored to a file (and optionally a line) within
/// the bundle. `path` is relative to the bundle root.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub path: String,
    pub line: Option<usize>,
    pub code: &'static str,
    pub message: String,
}

impl Diagnostic {
    pub fn error(path: &str, line: Option<usize>, code: &'static str, message: impl Into<String>) -> Self {
        Diagnostic { severity: Severity::Error, path: path.to_string(), line, code, message: message.into() }
    }

    pub fn warning(path: &str, line: Option<usize>, code: &'static str, message: impl Into<String>) -> Self {
        Diagnostic { severity: Severity::Warning, path: path.to_string(), line, code, message: message.into() }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(l) => write!(f, "{}:{}: {} [{}] {}", self.path, l, self.severity, self.code, self.message),
            None => write!(f, "{}: {} [{}] {}", self.path, self.severity, self.code, self.message),
        }
    }
}
