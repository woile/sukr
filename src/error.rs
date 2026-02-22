//! Error types for the sukr compiler, split by pipeline phase.
//!
//! - [`ParseError`] — failures during content discovery (S → C)
//! - [`CompileError`] — failures during rendering and output (C → O)
//! - [`Error`] — top-level sum type used by `run()`

use std::error::Error as StdError;
use std::fmt;
use std::path::PathBuf;

// ── Parse phase errors (S → C) ──────────────────────────────────────────

/// Errors that occur during content discovery and parsing.
#[derive(Debug)]
pub enum ParseError {
    /// Failed to read a content file.
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse frontmatter.
    Frontmatter { path: PathBuf, message: String },

    /// Content directory not found.
    ContentDirNotFound(PathBuf),

    /// Broken internal link detected during reference validation.
    BrokenLink {
        source_page: PathBuf,
        target: String,
        line: Option<usize>,
    },

    /// Failed to parse configuration file.
    Config { path: PathBuf, message: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ReadFile { path, source } => {
                write!(f, "failed to read {}: {}", path.display(), source)
            },
            ParseError::Frontmatter { path, message } => {
                write!(f, "invalid frontmatter in {}: {}", path.display(), message)
            },
            ParseError::ContentDirNotFound(path) => {
                write!(f, "content directory not found: {}", path.display())
            },
            ParseError::BrokenLink {
                source_page,
                target,
                line,
            } => {
                if let Some(ln) = line {
                    write!(
                        f,
                        "broken link in {} (line {}): {}",
                        source_page.display(),
                        ln,
                        target
                    )
                } else {
                    write!(f, "broken link in {}: {}", source_page.display(), target)
                }
            },
            ParseError::Config { path, message } => {
                write!(f, "invalid config in {}: {}", path.display(), message)
            },
        }
    }
}

impl StdError for ParseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ParseError::ReadFile { source, .. } => Some(source),
            _ => None,
        }
    }
}

// ── Compile phase errors (C → O) ────────────────────────────────────────

/// Errors that occur during rendering and output generation.
#[derive(Debug)]
pub enum CompileError {
    /// Failed to write output file.
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to create output directory.
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to load templates.
    TemplateLoad(tera::Error),

    /// Failed to render template.
    TemplateRender {
        template: String,
        source: tera::Error,
    },

    /// Failed to bundle CSS.
    CssBundle(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::WriteFile { path, source } => {
                write!(f, "failed to write {}: {}", path.display(), source)
            },
            CompileError::CreateDir { path, source } => {
                write!(
                    f,
                    "failed to create directory {}: {}",
                    path.display(),
                    source
                )
            },
            CompileError::TemplateLoad(e) => write!(f, "failed to load templates: {}", e),
            CompileError::TemplateRender { template, .. } => {
                write!(f, "failed to render template '{}'", template)
            },
            CompileError::CssBundle(msg) => write!(f, "CSS bundle error: {}", msg),
        }
    }
}

impl StdError for CompileError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CompileError::WriteFile { source, .. } => Some(source),
            CompileError::CreateDir { source, .. } => Some(source),
            CompileError::TemplateLoad(e) => Some(e),
            CompileError::TemplateRender { source, .. } => Some(source),
            _ => None,
        }
    }
}

// ── Top-level error (sum type for run()) ─────────────────────────────────

/// All errors that can occur during site compilation.
///
/// Composes [`ParseError`] and [`CompileError`] via `From` impls,
/// allowing `?` to propagate phase-specific errors up to `run()`.
#[derive(Debug)]
pub enum Error {
    /// Parse-phase error.
    Parse(ParseError),
    /// Compile-phase error.
    Compile(CompileError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(e) => e.fmt(f),
            Error::Compile(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Parse(e) => e.source(),
            Error::Compile(e) => e.source(),
        }
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}

impl From<CompileError> for Error {
    fn from(e: CompileError) -> Self {
        Error::Compile(e)
    }
}

/// Result type alias for top-level compiler operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Result type alias for parse-phase operations.
pub type ParseResult<T> = std::result::Result<T, ParseError>;

/// Result type alias for compile-phase operations.
pub type CompileResult<T> = std::result::Result<T, CompileError>;
