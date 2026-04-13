use git_reviewer::application::ports::git_repository::{
    GitRepository, GitRepositoryError, RawReviewDiff,
};
use git_reviewer::application::use_cases::load_review_commits::{
    LoadReviewCommits, LoadReviewCommitsError,
};
use git_reviewer::domain::repo::{RepoSnapshot, ReviewCommitSummary};

#[test]
fn returns_commits_for_selected_base_branch() {
    let repository = StubGitRepository::new(vec![ReviewCommitSummary {
        sha: "abc123456789".to_string(),
        short_sha: "abc1234".to_string(),
        subject: "feature commit".to_string(),
        is_local_changes: false,
    }]);

    let commits = LoadReviewCommits::new(&repository)
        .execute("origin/main")
        .expect("commit list should load");

    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].subject, "LOCAL CHANGES");
    assert!(commits[0].is_local_changes);
    assert_eq!(commits[1].subject, "feature commit");
}

#[test]
fn maps_invalid_base_branch_to_client_error() {
    let repository = StubGitRepository::invalid_base();

    let error = LoadReviewCommits::new(&repository)
        .execute("origin/missing")
        .expect_err("invalid base should fail");

    assert_eq!(error, LoadReviewCommitsError::InvalidBaseBranch);
}

struct StubGitRepository {
    commit_summaries: Result<Vec<ReviewCommitSummary>, GitRepositoryError>,
}

impl StubGitRepository {
    fn new(commits: Vec<ReviewCommitSummary>) -> Self {
        Self {
            commit_summaries: Ok(commits),
        }
    }

    fn invalid_base() -> Self {
        Self {
            commit_summaries: Err(GitRepositoryError::InvalidBaseBranch),
        }
    }
}

impl GitRepository for StubGitRepository {
    fn head_sha(&self) -> Result<String, GitRepositoryError> {
        Err(GitRepositoryError::HeadShaUnavailable)
    }

    fn current_branch(&self) -> Result<String, GitRepositoryError> {
        Err(GitRepositoryError::CurrentBranchUnavailable)
    }

    fn remote_branches(&self) -> Result<Vec<String>, GitRepositoryError> {
        Err(GitRepositoryError::RemoteBranchesUnavailable)
    }

    fn commit_summaries(
        &self,
        _base_branch: &str,
    ) -> Result<Vec<ReviewCommitSummary>, GitRepositoryError> {
        self.commit_summaries.clone()
    }

    fn raw_review_diff(&self, _base_branch: &str) -> Result<RawReviewDiff, GitRepositoryError> {
        Err(GitRepositoryError::ReviewDiffUnavailable)
    }

    fn raw_commit_diff(&self, _commit_sha: &str) -> Result<RawReviewDiff, GitRepositoryError> {
        Err(GitRepositoryError::InvalidCommit)
    }

    fn raw_local_changes_diff(&self) -> Result<RawReviewDiff, GitRepositoryError> {
        Err(GitRepositoryError::ReviewDiffUnavailable)
    }

    fn repo_snapshot(&self) -> Result<RepoSnapshot, GitRepositoryError> {
        Err(GitRepositoryError::RepoSnapshotUnavailable)
    }

    fn file_content_at_revision(
        &self,
        _revision: &str,
        _path: &str,
    ) -> Result<Option<String>, GitRepositoryError> {
        Err(GitRepositoryError::FileContentUnavailable)
    }
}
