use crate::application::default_base_branch::{
    resolve_default_base_branch, DefaultBaseBranchError,
};
use crate::application::ports::git_repository::{GitRepository, GitRepositoryError};
use crate::application::use_cases::review_diff_parser::{parse_review, ParseReviewError};
use crate::domain::repo::ReviewCommitSummary;
use crate::domain::review::{DiffLineKind, Review, ReviewMode};

const LARGE_FILE_LINE_LIMIT: usize = 800;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadReviewRequest {
    pub review_mode: ReviewMode,
    pub selected_base: Option<String>,
    pub selected_commit: Option<String>,
    pub expanded_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadReviewError {
    GitRepository(GitRepositoryError),
    MissingRemoteBranches,
    InvalidBaseBranch,
    InvalidCommit,
    ParseReview(ParseReviewError),
}

impl From<GitRepositoryError> for LoadReviewError {
    fn from(error: GitRepositoryError) -> Self {
        match error {
            GitRepositoryError::InvalidBaseBranch => Self::InvalidBaseBranch,
            GitRepositoryError::InvalidCommit => Self::InvalidCommit,
            other => Self::GitRepository(other),
        }
    }
}

impl From<ParseReviewError> for LoadReviewError {
    fn from(error: ParseReviewError) -> Self {
        Self::ParseReview(error)
    }
}

pub struct LoadReview<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    repository: &'a TGitRepository,
}

impl<'a, TGitRepository> LoadReview<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    pub fn new(repository: &'a TGitRepository) -> Self {
        Self { repository }
    }

    pub fn execute(&self, request: LoadReviewRequest) -> Result<Review, LoadReviewError> {
        let review_mode = request.review_mode.clone();
        let base_branch = match request.selected_base {
            Some(base_branch) => base_branch,
            None => self.default_base_branch()?,
        };
        let raw_diff = match request.review_mode {
            ReviewMode::Branch => self.repository.raw_review_diff(&base_branch)?,
            ReviewMode::Commit => {
                let selected_commit = request
                    .selected_commit
                    .clone()
                    .ok_or(LoadReviewError::InvalidCommit)?;
                if selected_commit == ReviewCommitSummary::local_changes().sha {
                    self.repository.raw_local_changes_diff()?
                } else {
                    let available = self.repository.commit_summaries(&base_branch)?;
                    if !available.iter().any(|entry| entry.sha == selected_commit) {
                        return Err(LoadReviewError::InvalidCommit);
                    }
                    self.repository.raw_commit_diff(&selected_commit)?
                }
            }
        };
        let expanded_paths = request.expanded_paths;
        let merge_base_sha = raw_diff.merge_base_sha.clone();
        let mut review = parse_review(raw_diff)?;

        review.review_mode = review_mode;
        review.base_branch = base_branch;
        review.selected_commit = request.selected_commit;

        for file in &mut review.files {
            if file.is_binary {
                continue;
            }

            let line_count =
                detect_line_count(self.repository, file, &merge_base_sha, &review.head_sha);
            file.line_count = line_count;
            file.is_large = line_count > LARGE_FILE_LINE_LIMIT;
            if file.is_large && !expanded_paths.iter().any(|path| path == &file.path()) {
                file.is_loaded = false;
                file.hunks.clear();
            }
        }

        Ok(review)
    }

    fn default_base_branch(&self) -> Result<String, LoadReviewError> {
        match resolve_default_base_branch(&self.repository.remote_branches()?) {
            Ok(branch) => Ok(branch),
            Err(DefaultBaseBranchError::MissingRemoteBranches) => {
                Err(LoadReviewError::MissingRemoteBranches)
            }
        }
    }
}

fn detect_line_count<TGitRepository>(
    repository: &TGitRepository,
    file: &crate::domain::review::ReviewFile,
    merge_base_sha: &str,
    head_sha: &str,
) -> usize
where
    TGitRepository: GitRepository,
{
    let preferred = if file.new_path.is_some() {
        file.new_path.as_deref().and_then(|path| {
            repository
                .file_content_at_revision(head_sha, path)
                .ok()
                .flatten()
        })
    } else {
        file.old_path.as_deref().and_then(|path| {
            repository
                .file_content_at_revision(merge_base_sha, path)
                .ok()
                .flatten()
        })
    };

    preferred
        .map(|content| content.lines().count())
        .unwrap_or_else(|| estimate_line_count(file))
}

fn estimate_line_count(file: &crate::domain::review::ReviewFile) -> usize {
    file.hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .filter_map(|line| match line.kind {
            DiffLineKind::Added | DiffLineKind::Context => line.new_line_number,
            DiffLineKind::Removed => line.old_line_number,
        })
        .max()
        .unwrap_or(0)
}
