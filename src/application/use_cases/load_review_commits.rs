use crate::application::ports::git_repository::{GitRepository, GitRepositoryError};
use crate::domain::repo::ReviewCommitSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadReviewCommitsError {
    GitRepository(GitRepositoryError),
    InvalidBaseBranch,
}

impl From<GitRepositoryError> for LoadReviewCommitsError {
    fn from(error: GitRepositoryError) -> Self {
        match error {
            GitRepositoryError::InvalidBaseBranch => Self::InvalidBaseBranch,
            other => Self::GitRepository(other),
        }
    }
}

pub struct LoadReviewCommits<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    repository: &'a TGitRepository,
}

impl<'a, TGitRepository> LoadReviewCommits<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    pub fn new(repository: &'a TGitRepository) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        base_branch: &str,
    ) -> Result<Vec<ReviewCommitSummary>, LoadReviewCommitsError> {
        let mut commits = vec![ReviewCommitSummary::local_changes()];
        commits.extend(self.repository.commit_summaries(base_branch)?);
        Ok(commits)
    }
}
