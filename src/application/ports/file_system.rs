#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSystemError {
    ReadFailed(String),
}

pub trait FileSystem {
    fn read_text(&self, path: &str) -> Result<Option<String>, FileSystemError>;
}
