//! Custom error types for the sukr compiler.

use std::error::Error as StdError;
use std::fmt;
use std::path::PathBuf;

/// All errors that can occur during site compilation.
#[derive(Debug)]
pub enum Error {
    /// Failed to read a content file.
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse frontmatter.
    Frontmatter { path: PathBuf, message: String },

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

    /// Content directory not found.
    ContentDirNotFound(PathBuf),

    /// Failed to parse configuration file.
    Config { path: PathBuf, message: String },

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ReadFile { path, source } => {
                write!(f, "failed to read {}: {}", path.display(), source)
            },
            Error::Frontmatter { path, message } => {
                write!(f, "invalid frontmatter in {}: {}", path.display(), message)
            },
            Error::WriteFile { path, source } => {
                write!(f, "failed to write {}: {}", path.display(), source)
            },
            Error::CreateDir { path, source } => {
                write!(
                    f,
                    "failed to create directory {}: {}",
                    path.display(),
                    source
                )
            },
            Error::ContentDirNotFound(path) => {
                write!(f, "content directory not found: {}", path.display())
            },
            Error::Config { path, message } => {
                write!(f, "invalid config in {}: {}", path.display(), message)
            },
            Error::TemplateLoad(e) => write!(f, "failed to load templates: {}", e),
            Error::TemplateRender { template, .. } => {
                write!(f, "failed to render template '{}'", template)
            },
            Error::CssBundle(msg) => write!(f, "CSS bundle error: {}", msg),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::ReadFile { source, .. } => Some(source),
            Error::WriteFile { source, .. } => Some(source),
            Error::CreateDir { source, .. } => Some(source),
            Error::TemplateLoad(e) => Some(e),
            Error::TemplateRender { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Result type alias for compiler operations.
pub type Result<T> = std::result::Result<T, Error>;
