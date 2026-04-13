#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Review {
    pub review_mode: ReviewMode,
    pub base_branch: String,
    pub selected_commit: Option<String>,
    pub merge_base_sha: String,
    pub head_sha: String,
    pub files: Vec<ReviewFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewMode {
    Branch,
    Commit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewFile {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub change: DiffChange,
    pub is_binary: bool,
    pub is_large: bool,
    pub is_loaded: bool,
    pub line_count: usize,
    pub hunks: Vec<DiffHunk>,
}

impl ReviewFile {
    pub fn path(&self) -> String {
        self.new_path
            .clone()
            .or_else(|| self.old_path.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffChange {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorSide {
    Base,
    Head,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentAnchor {
    pub base_sha: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub side: AnchorSide,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCommentDraft {
    pub anchor: CommentAnchor,
    pub source_head_sha: String,
    pub body: String,
}
