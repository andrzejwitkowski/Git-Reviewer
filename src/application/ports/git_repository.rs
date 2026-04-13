use crate::domain::repo::RepoSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitRepositoryError {
    HeadShaUnavailable,
    CurrentBranchUnavailable,
    RemoteBranchesUnavailable,
    InvalidBaseBranch,
    ReviewDiffUnavailable,
    RepoSnapshotUnavailable,
    FileContentUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawReviewDiff {
    pub base_branch: String,
    pub merge_base_sha: String,
    pub head_sha: String,
    pub diff: String,
}

pub trait GitRepository {
    fn head_sha(&self) -> Result<String, GitRepositoryError>;
    fn current_branch(&self) -> Result<String, GitRepositoryError>;
    fn remote_branches(&self) -> Result<Vec<String>, GitRepositoryError>;
    fn raw_review_diff(&self, base_branch: &str) -> Result<RawReviewDiff, GitRepositoryError>;
    fn repo_snapshot(&self) -> Result<RepoSnapshot, GitRepositoryError>;
    fn file_content_at_revision(
        &self,
        revision: &str,
        path: &str,
    ) -> Result<Option<String>, GitRepositoryError>;
}
