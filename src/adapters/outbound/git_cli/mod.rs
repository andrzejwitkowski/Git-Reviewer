use crate::application::ports::git_repository::{GitRepository, GitRepositoryError, RawReviewDiff};
use crate::domain::repo::{RepoSnapshot, RepoStatusEntry};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone)]
pub struct GitCliRepository {
    repo_path: PathBuf,
}

impl GitCliRepository {
    pub fn new(path: &Path) -> Self {
        Self {
            repo_path: path.to_path_buf(),
        }
    }

    fn run_trimmed(
        &self,
        args: &[&str],
        error: GitRepositoryError,
    ) -> Result<String, GitRepositoryError> {
        self.run_raw(args, error)
            .map(|stdout| stdout.trim_end_matches('\n').to_string())
    }

    fn run_raw(
        &self,
        args: &[&str],
        error: GitRepositoryError,
    ) -> Result<String, GitRepositoryError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|_| error.clone())?;

        if !output.status.success() {
            return Err(error);
        }

        String::from_utf8(output.stdout).map_err(|_| error)
    }
}

impl GitRepository for GitCliRepository {
    fn head_sha(&self) -> Result<String, GitRepositoryError> {
        self.run_trimmed(
            &["rev-parse", "HEAD"],
            GitRepositoryError::HeadShaUnavailable,
        )
    }

    fn current_branch(&self) -> Result<String, GitRepositoryError> {
        self.run_trimmed(
            &["rev-parse", "--abbrev-ref", "HEAD"],
            GitRepositoryError::CurrentBranchUnavailable,
        )
    }

    fn remote_branches(&self) -> Result<Vec<String>, GitRepositoryError> {
        let output = self.run_trimmed(
            &["for-each-ref", "--format=%(refname:short)", "refs/remotes"],
            GitRepositoryError::RemoteBranchesUnavailable,
        )?;

        Ok(output
            .lines()
            .filter(|line| !line.ends_with("/HEAD") && !line.is_empty())
            .map(|line| line.to_string())
            .collect())
    }

    fn raw_review_diff(&self, base_branch: &str) -> Result<RawReviewDiff, GitRepositoryError> {
        let merge_base = self
            .run_trimmed(
                &["merge-base", "HEAD", base_branch],
                GitRepositoryError::ReviewDiffUnavailable,
            )
            .map_err(|error| match error {
                GitRepositoryError::ReviewDiffUnavailable => GitRepositoryError::InvalidBaseBranch,
                other => other,
            })?;
        let head_sha = self.head_sha()?;
        let diff = self.run_raw(
            &[
                "diff",
                "--find-renames",
                "--binary",
                "--no-color",
                &merge_base,
                "--",
            ],
            GitRepositoryError::ReviewDiffUnavailable,
        )?;

        Ok(RawReviewDiff {
            base_branch: base_branch.to_string(),
            merge_base_sha: merge_base,
            head_sha,
            diff,
        })
    }

    fn repo_snapshot(&self) -> Result<RepoSnapshot, GitRepositoryError> {
        let head_sha = self.head_sha()?;
        let current_branch = self.current_branch()?;
        let status = self.run_trimmed(
            &["status", "--porcelain=v1", "--untracked-files=all"],
            GitRepositoryError::RepoSnapshotUnavailable,
        )?;
        let entries = status
            .lines()
            .filter(|line| !line.is_empty())
            .map(parse_status_line)
            .collect();

        Ok(RepoSnapshot::new(head_sha, current_branch, entries))
    }

    fn file_content_at_revision(
        &self,
        revision: &str,
        path: &str,
    ) -> Result<Option<String>, GitRepositoryError> {
        let exists = self.run_trimmed(
            &["ls-tree", "-r", "--name-only", revision, "--", path],
            GitRepositoryError::FileContentUnavailable,
        )?;

        if exists.lines().next().is_none() {
            return Ok(None);
        }

        self.run_raw(
            &["show", &format!("{revision}:{path}")],
            GitRepositoryError::FileContentUnavailable,
        )
        .map(Some)
    }
}

fn parse_status_line(line: &str) -> RepoStatusEntry {
    RepoStatusEntry {
        code: line[0..2].to_string(),
        path: decode_status_path(&line[3..]),
    }
}

fn decode_status_path(path: &str) -> String {
    if path.contains(" -> ") {
        decode_rename_path(path)
    } else if path.starts_with('"') {
        decode_quoted_path(path)
    } else {
        path.to_string()
    }
}

fn decode_rename_path(path: &str) -> String {
    let mut parts = path.splitn(2, " -> ");
    let from = parts.next().unwrap_or_default();
    let to = parts.next().unwrap_or_default();
    format!("{} -> {}", decode_quoted_path(from), decode_quoted_path(to))
}

fn decode_quoted_path(path: &str) -> String {
    let content = path.trim();
    let content = content.strip_prefix('"').unwrap_or(content);
    let content = content.strip_suffix('"').unwrap_or(content);
    let mut decoded = String::new();
    let mut chars = content.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                decoded.push(match next {
                    't' => '\t',
                    'n' => '\n',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                });
            }
        } else {
            decoded.push(ch);
        }
    }

    decoded
}
