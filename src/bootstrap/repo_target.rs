use crate::error::AppError;
use std::path::{Path, PathBuf};

pub fn resolve(path: PathBuf) -> Result<PathBuf, AppError> {
    let target = path.canonicalize().unwrap_or(path);

    if !target.exists() {
        return Err(AppError::MissingDirectory(target));
    }

    if !is_git_repository(&target) {
        return Err(AppError::NotGitRepository(target));
    }

    Ok(target)
}

fn is_git_repository(path: &Path) -> bool {
    path.ancestors()
        .any(|candidate| candidate.join(".git").exists())
}
