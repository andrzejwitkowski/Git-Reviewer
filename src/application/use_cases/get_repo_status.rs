use crate::application::ports::git_repository::{GitRepository, GitRepositoryError};
use crate::domain::repo::RepoSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetRepoStatusError {
    GitRepository(GitRepositoryError),
}

impl From<GitRepositoryError> for GetRepoStatusError {
    fn from(error: GitRepositoryError) -> Self {
        Self::GitRepository(error)
    }
}

pub struct GetRepoStatus<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    repository: &'a TGitRepository,
}

impl<'a, TGitRepository> GetRepoStatus<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    pub fn new(repository: &'a TGitRepository) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<RepoSnapshot, GetRepoStatusError> {
        Ok(self.repository.repo_snapshot()?)
    }
}
