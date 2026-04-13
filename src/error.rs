use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    MissingDirectory(PathBuf),
    NotGitRepository(PathBuf),
    HttpBindFailed(io::Error),
    HttpServeFailed(io::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDirectory(path) => {
                write!(f, "Path '{}' does not exist", path.display())
            }
            Self::NotGitRepository(path) => {
                write!(f, "Path '{}' is not a git repository", path.display())
            }
            Self::HttpBindFailed(error) => write!(f, "Failed to bind HTTP server: {error}"),
            Self::HttpServeFailed(error) => write!(f, "HTTP server failed: {error}"),
        }
    }
}

impl std::error::Error for AppError {}
