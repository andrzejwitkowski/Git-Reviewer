use crate::domain::repo::{RepoContext, RepoSnapshot, RepoStatusEntry, ReviewCommitSummary};
use crate::domain::review::{
    AnchorSide, CommentAnchor, DiffHunk, DiffLine, DiffLineKind, Review, ReviewCommentDraft,
    ReviewFile, ReviewMode,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoContextResponse {
    pub repo_path: String,
    pub head_sha: String,
    pub current_branch: String,
    pub remote_branches: Vec<String>,
    pub default_base_branch: String,
}

impl RepoContextResponse {
    pub fn from_domain(repo_path: String, context: RepoContext) -> Self {
        Self {
            repo_path,
            head_sha: context.head_sha,
            current_branch: context.current_branch,
            remote_branches: context.remote_branches,
            default_base_branch: context.default_base_branch,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResponse {
    pub review_mode: &'static str,
    pub base_branch: String,
    pub selected_commit: Option<String>,
    pub merge_base_sha: String,
    pub head_sha: String,
    pub files: Vec<ReviewFileResponse>,
}

impl From<Review> for ReviewResponse {
    fn from(review: Review) -> Self {
        Self {
            review_mode: match review.review_mode {
                ReviewMode::Branch => "branch",
                ReviewMode::Commit => "commit",
            },
            base_branch: review.base_branch,
            selected_commit: review.selected_commit,
            merge_base_sha: review.merge_base_sha,
            head_sha: review.head_sha,
            files: review.files.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFileResponse {
    pub path: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub change: &'static str,
    pub is_binary: bool,
    pub is_large: bool,
    pub is_loaded: bool,
    pub line_count: usize,
    pub hunks: Vec<ReviewHunkResponse>,
}

impl From<ReviewFile> for ReviewFileResponse {
    fn from(file: ReviewFile) -> Self {
        let path = file.path();

        Self {
            path,
            old_path: file.old_path,
            new_path: file.new_path,
            change: match file.change {
                crate::domain::review::DiffChange::Added => "added",
                crate::domain::review::DiffChange::Modified => "modified",
                crate::domain::review::DiffChange::Deleted => "deleted",
                crate::domain::review::DiffChange::Renamed => "renamed",
            },
            is_binary: file.is_binary,
            is_large: file.is_large,
            is_loaded: file.is_loaded,
            line_count: file.line_count,
            hunks: file.hunks.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHunkResponse {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<ReviewLineResponse>,
}

impl From<DiffHunk> for ReviewHunkResponse {
    fn from(hunk: DiffHunk) -> Self {
        Self {
            old_start: hunk.old_start,
            old_count: hunk.old_count,
            new_start: hunk.new_start,
            new_count: hunk.new_count,
            lines: hunk.lines.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewLineResponse {
    pub kind: &'static str,
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
    pub text: String,
}

impl From<DiffLine> for ReviewLineResponse {
    fn from(line: DiffLine) -> Self {
        Self {
            kind: match line.kind {
                DiffLineKind::Context => "context",
                DiffLineKind::Added => "added",
                DiffLineKind::Removed => "removed",
            },
            old_line_number: line.old_line_number,
            new_line_number: line.new_line_number,
            text: line.text,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatusResponse {
    pub head_sha: String,
    pub current_branch: String,
    pub local_changes_sha: String,
    pub entries: Vec<RepoStatusEntryResponse>,
}

impl From<RepoSnapshot> for RepoStatusResponse {
    fn from(snapshot: RepoSnapshot) -> Self {
        Self {
            head_sha: snapshot.head_sha,
            current_branch: snapshot.current_branch,
            local_changes_sha: snapshot.local_changes_sha,
            entries: snapshot.entries.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatusEntryResponse {
    pub code: String,
    pub path: String,
}

impl From<RepoStatusEntry> for RepoStatusEntryResponse {
    fn from(entry: RepoStatusEntry) -> Self {
        Self {
            code: entry.code,
            path: entry.path,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCommitSummaryResponse {
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
    pub is_local_changes: bool,
}

impl From<ReviewCommitSummary> for ReviewCommitSummaryResponse {
    fn from(summary: ReviewCommitSummary) -> Self {
        Self {
            sha: summary.sha,
            short_sha: summary.short_sha,
            subject: summary.subject,
            is_local_changes: summary.is_local_changes,
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardExportRequest {
    pub head_sha: String,
    pub comments: Vec<ClipboardCommentRequest>,
}

impl ClipboardExportRequest {
    pub fn into_domain(self) -> Result<(String, Vec<ReviewCommentDraft>), InvalidClipboardRequest> {
        Ok((
            self.head_sha,
            self.comments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardCommentRequest {
    pub anchor: ClipboardCommentAnchorRequest,
    pub source_head_sha: String,
    pub body: String,
}

impl TryFrom<ClipboardCommentRequest> for ReviewCommentDraft {
    type Error = InvalidClipboardRequest;

    fn try_from(comment: ClipboardCommentRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            anchor: comment.anchor.try_into()?,
            source_head_sha: comment.source_head_sha,
            body: comment.body,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardCommentAnchorRequest {
    pub base_sha: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub side: String,
    pub line_number: usize,
}

impl TryFrom<ClipboardCommentAnchorRequest> for CommentAnchor {
    type Error = InvalidClipboardRequest;

    fn try_from(anchor: ClipboardCommentAnchorRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            base_sha: anchor.base_sha,
            old_path: anchor.old_path,
            new_path: anchor.new_path,
            side: match anchor.side.as_str() {
                "base" => AnchorSide::Base,
                "head" => AnchorSide::Head,
                _ => return Err(InvalidClipboardRequest),
            },
            line_number: anchor.line_number,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InvalidClipboardRequest;

#[derive(Serialize)]
pub struct ClipboardExportResponse {
    pub text: String,
}
