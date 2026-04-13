use crate::application::ports::file_system::{FileSystem, FileSystemError};
use std::fs;
use std::path::{Path, PathBuf};

pub struct NativeFileSystem {
    root: PathBuf,
}

impl NativeFileSystem {
    pub fn new(path: &Path) -> Self {
        Self {
            root: path.to_path_buf(),
        }
    }
}

impl FileSystem for NativeFileSystem {
    fn read_text(&self, path: &str) -> Result<Option<String>, FileSystemError> {
        let target = self.root.join(path);

        if !target.exists() {
            return Ok(None);
        }

        fs::read_to_string(target)
            .map(Some)
            .map_err(|error| FileSystemError::ReadFailed(error.to_string()))
    }
}
