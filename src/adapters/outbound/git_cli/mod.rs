use crate::application::ports::git_repository::{GitRepository, GitRepositoryError, RawReviewDiff};
use crate::domain::repo::{RepoSnapshot, RepoStatusEntry, ReviewCommitSummary};
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
        self.run_raw_with_allowed_statuses(args, &[0], error)
    }

    fn run_raw_with_allowed_statuses(
        &self,
        args: &[&str],
        allowed_statuses: &[i32],
        error: GitRepositoryError,
    ) -> Result<String, GitRepositoryError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|_| error.clone())?;

        let status = output.status.code().unwrap_or_default();
        if !allowed_statuses.contains(&status) {
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

    fn commit_summaries(
        &self,
        base_branch: &str,
    ) -> Result<Vec<ReviewCommitSummary>, GitRepositoryError> {
        let output = self
            .run_trimmed(
                &[
                    "log",
                    "--format=%H%x09%h%x09%s",
                    &format!("{base_branch}..HEAD"),
                ],
                GitRepositoryError::ReviewDiffUnavailable,
            )
            .map_err(|error| match error {
                GitRepositoryError::ReviewDiffUnavailable => GitRepositoryError::InvalidBaseBranch,
                other => other,
            })?;

        Ok(output
            .lines()
            .filter(|line| !line.is_empty())
            .map(parse_commit_summary)
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

    fn raw_commit_diff(&self, commit_sha: &str) -> Result<RawReviewDiff, GitRepositoryError> {
        let commit = self
            .run_trimmed(
                &["rev-parse", commit_sha],
                GitRepositoryError::ReviewDiffUnavailable,
            )
            .map_err(|error| match error {
                GitRepositoryError::ReviewDiffUnavailable => GitRepositoryError::InvalidCommit,
                other => other,
            })?;
        let parent = self
            .run_trimmed(
                &["rev-parse", &format!("{commit}^")],
                GitRepositoryError::ReviewDiffUnavailable,
            )
            .map_err(|error| match error {
                GitRepositoryError::ReviewDiffUnavailable => GitRepositoryError::InvalidCommit,
                other => other,
            })?;
        let diff = self.run_raw(
            &[
                "diff",
                "--find-renames",
                "--binary",
                "--no-color",
                &parent,
                &commit,
                "--",
            ],
            GitRepositoryError::ReviewDiffUnavailable,
        )?;

        Ok(RawReviewDiff {
            base_branch: "commit".to_string(),
            merge_base_sha: parent,
            head_sha: commit,
            diff,
        })
    }

    fn raw_local_changes_diff(&self) -> Result<RawReviewDiff, GitRepositoryError> {
        let head_sha = self.head_sha()?;
        let tracked_diff = self.run_raw(
            &[
                "diff",
                "--find-renames",
                "--binary",
                "--no-color",
                "HEAD",
                "--",
            ],
            GitRepositoryError::ReviewDiffUnavailable,
        )?;
        let untracked = self.run_trimmed(
            &["ls-files", "--others", "--exclude-standard"],
            GitRepositoryError::ReviewDiffUnavailable,
        )?;
        let mut diff = tracked_diff;
        let empty_file = EmptyDiffSource::create(&self.repo_path)?;
        let empty_path = empty_file.path_string();

        for path in untracked.lines().filter(|line| !line.is_empty()) {
            let file_diff = self.run_raw_with_allowed_statuses(
                &[
                    "diff",
                    "--find-renames",
                    "--binary",
                    "--no-color",
                    "--no-index",
                    "--",
                    &empty_path,
                    path,
                ],
                &[1],
                GitRepositoryError::ReviewDiffUnavailable,
            )?;
            if !diff.is_empty() && !diff.ends_with('\n') {
                diff.push('\n');
            }
            diff.push_str(&normalize_untracked_diff(&file_diff, &empty_path, path));
            if !diff.ends_with('\n') {
                diff.push('\n');
            }
        }

        Ok(RawReviewDiff {
            base_branch: "local".to_string(),
            merge_base_sha: head_sha.clone(),
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
        let local_changes_sha = self.raw_local_changes_diff()?.diff;

        Ok(RepoSnapshot::new(
            head_sha,
            current_branch,
            local_changes_sha,
            entries,
        ))
    }

    fn file_content_at_revision(
        &self,
        revision: &str,
        path: &str,
    ) -> Result<Option<String>, GitRepositoryError> {
        if revision == "WORKTREE" {
            return std::fs::read_to_string(self.repo_path.join(path))
                .map(Some)
                .or_else(|error| {
                    if error.kind() == std::io::ErrorKind::NotFound {
                        Ok(None)
                    } else {
                        Err(GitRepositoryError::FileContentUnavailable)
                    }
                });
        }

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

fn parse_commit_summary(line: &str) -> ReviewCommitSummary {
    let mut parts = line.splitn(3, '\t');

    ReviewCommitSummary {
        sha: parts.next().unwrap_or_default().to_string(),
        short_sha: parts.next().unwrap_or_default().to_string(),
        subject: parts.next().unwrap_or_default().to_string(),
        is_local_changes: false,
    }
}

fn normalize_untracked_diff(diff: &str, empty_path: &str, path: &str) -> String {
    let diff_header = format!(" b/{path}");
    let source = format!("--- a/{empty_path}");
    let alt_source = format!("--- a/{}", empty_path.trim_start_matches('/'));
    let target = format!("+++ b/{path}");

    diff.lines()
        .map(|line| {
            if line.starts_with("diff --git a/") && line.ends_with(&diff_header) {
                format!("diff --git a/dev/null{diff_header}")
            } else if line == source || line == alt_source {
                "--- a/dev/null".to_string()
            } else if line == format!("+++ b/{empty_path}") {
                target.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

struct EmptyDiffSource {
    path: PathBuf,
}

impl EmptyDiffSource {
    fn create(repo_path: &Path) -> Result<Self, GitRepositoryError> {
        let git_dir = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(repo_path)
            .output()
            .map_err(|_| GitRepositoryError::ReviewDiffUnavailable)?;

        if !git_dir.status.success() {
            return Err(GitRepositoryError::ReviewDiffUnavailable);
        }

        let git_dir = String::from_utf8(git_dir.stdout)
            .map_err(|_| GitRepositoryError::ReviewDiffUnavailable)?;
        let git_dir = git_dir.trim_end_matches('\n');
        let path = repo_path
            .join(git_dir)
            .join(format!("git-reviewer-empty-{}", std::process::id()));

        std::fs::write(&path, b"").map_err(|_| GitRepositoryError::ReviewDiffUnavailable)?;

        Ok(Self { path })
    }

    fn path_string(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

impl Drop for EmptyDiffSource {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
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
