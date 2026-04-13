use crate::application::default_base_branch::{
    resolve_default_base_branch, DefaultBaseBranchError,
};
use crate::application::ports::git_repository::{GitRepository, GitRepositoryError};
use crate::domain::repo::RepoContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadRepoContextError {
    GitRepository(GitRepositoryError),
    MissingRemoteBranches,
}

impl From<GitRepositoryError> for LoadRepoContextError {
    fn from(error: GitRepositoryError) -> Self {
        Self::GitRepository(error)
    }
}

pub struct LoadRepoContext<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    repository: &'a TGitRepository,
}

impl<'a, TGitRepository> LoadRepoContext<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    pub fn new(repository: &'a TGitRepository) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<RepoContext, LoadRepoContextError> {
        let head_sha = self.repository.head_sha()?;
        let current_branch = self.repository.current_branch()?;
        let remote_branches = self.repository.remote_branches()?;
        let default_base_branch =
            resolve_default_base_branch(&remote_branches).map_err(|error| match error {
                DefaultBaseBranchError::MissingRemoteBranches => {
                    LoadRepoContextError::MissingRemoteBranches
                }
            })?;

        Ok(RepoContext::new(
            head_sha,
            current_branch,
            remote_branches,
            default_base_branch,
        ))
    }
}
