use git_reviewer::application::ports::git_repository::{
    GitRepository, GitRepositoryError, RawReviewDiff,
};
use git_reviewer::application::use_cases::load_repo_context::{
    LoadRepoContext, LoadRepoContextError,
};
use git_reviewer::domain::repo::{RepoSnapshot, ReviewCommitSummary};

#[test]
fn prefers_origin_main_as_default_base_branch() {
    let repository = StubGitRepository::new(
        "abc123",
        "feature/refactor",
        vec!["origin/develop", "origin/main", "origin/master"],
    );

    let context = LoadRepoContext::new(&repository)
        .execute()
        .expect("repo context should load");

    assert_eq!(context.head_sha, "abc123");
    assert_eq!(context.current_branch, "feature/refactor");
    assert_eq!(
        context.remote_branches,
        vec![
            "origin/develop".to_string(),
            "origin/main".to_string(),
            "origin/master".to_string(),
        ]
    );
    assert_eq!(context.default_base_branch, "origin/main");
}

#[test]
fn falls_back_to_origin_master_when_origin_main_is_missing() {
    let repository = StubGitRepository::new(
        "def456",
        "feature/refactor",
        vec!["origin/release", "origin/master"],
    );

    let context = LoadRepoContext::new(&repository)
        .execute()
        .expect("repo context should load");

    assert_eq!(context.default_base_branch, "origin/master");
}

#[test]
fn falls_back_to_first_remote_branch_when_main_and_master_are_missing() {
    let repository = StubGitRepository::new(
        "fedcba",
        "feature/refactor",
        vec!["origin/release", "upstream/trunk"],
    );

    let context = LoadRepoContext::new(&repository)
        .execute()
        .expect("repo context should load");

    assert_eq!(context.default_base_branch, "origin/release");
}

#[test]
fn returns_error_when_no_remote_branches_exist() {
    let repository = StubGitRepository::new("abc123", "feature/refactor", vec![]);

    let error = LoadRepoContext::new(&repository)
        .execute()
        .expect_err("missing remote branches should fail");

    assert_eq!(error, LoadRepoContextError::MissingRemoteBranches);
}

#[test]
fn propagates_port_failures() {
    let repository = StubGitRepository::failing_head_sha();

    let error = LoadRepoContext::new(&repository)
        .execute()
        .expect_err("port failure should be returned");

    assert_eq!(
        error,
        LoadRepoContextError::GitRepository(GitRepositoryError::HeadShaUnavailable)
    );
}

struct StubGitRepository {
    head_sha: Result<String, GitRepositoryError>,
    current_branch: String,
    remote_branches: Vec<String>,
}

impl StubGitRepository {
    fn new(head_sha: &str, current_branch: &str, remote_branches: Vec<&str>) -> Self {
        Self {
            head_sha: Ok(head_sha.to_string()),
            current_branch: current_branch.to_string(),
            remote_branches: remote_branches
                .into_iter()
                .map(|branch| branch.to_string())
                .collect(),
        }
    }

    fn failing_head_sha() -> Self {
        Self {
            head_sha: Err(GitRepositoryError::HeadShaUnavailable),
            current_branch: "feature/refactor".to_string(),
            remote_branches: vec!["origin/main".to_string()],
        }
    }
}

impl GitRepository for StubGitRepository {
    fn head_sha(&self) -> Result<String, GitRepositoryError> {
        self.head_sha.clone()
    }

    fn current_branch(&self) -> Result<String, GitRepositoryError> {
        Ok(self.current_branch.clone())
    }

    fn remote_branches(&self) -> Result<Vec<String>, GitRepositoryError> {
        Ok(self.remote_branches.clone())
    }

    fn commit_summaries(
        &self,
        _base_branch: &str,
    ) -> Result<Vec<ReviewCommitSummary>, GitRepositoryError> {
        Err(GitRepositoryError::ReviewDiffUnavailable)
    }

    fn raw_review_diff(&self, _base_branch: &str) -> Result<RawReviewDiff, GitRepositoryError> {
        Err(GitRepositoryError::ReviewDiffUnavailable)
    }

    fn raw_commit_diff(&self, _commit_sha: &str) -> Result<RawReviewDiff, GitRepositoryError> {
        Err(GitRepositoryError::ReviewDiffUnavailable)
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
