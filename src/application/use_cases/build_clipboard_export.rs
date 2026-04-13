use crate::application::ports::git_repository::{GitRepository, GitRepositoryError};
use crate::domain::review::{AnchorSide, ReviewCommentDraft};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildClipboardExportRequest {
    pub head_sha: String,
    pub comments: Vec<ReviewCommentDraft>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildClipboardExportError {
    GitRepository(GitRepositoryError),
    MissingAnchoredPath,
    MissingSourceContent,
}

impl From<GitRepositoryError> for BuildClipboardExportError {
    fn from(error: GitRepositoryError) -> Self {
        Self::GitRepository(error)
    }
}

pub struct BuildClipboardExport<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    repository: &'a TGitRepository,
}

impl<'a, TGitRepository> BuildClipboardExport<'a, TGitRepository>
where
    TGitRepository: GitRepository,
{
    pub fn new(repository: &'a TGitRepository) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        request: BuildClipboardExportRequest,
    ) -> Result<String, BuildClipboardExportError> {
        let BuildClipboardExportRequest { head_sha, comments } = request;
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for comment in comments {
            let path = anchored_path(&comment)?;
            let content = self
                .repository
                .file_content_at_revision(&source_revision(&comment), &path)?
                .ok_or(BuildClipboardExportError::MissingSourceContent)?;

            groups
                .entry(path.clone())
                .or_default()
                .push(format_comment_section(&comment, &path, &content));
        }

        Ok(groups
            .into_iter()
            .map(|(path, sections)| format!("{path} - {head_sha}\n\n{}", sections.join("\n\n")))
            .collect::<Vec<_>>()
            .join("\n\n"))
    }
}

fn source_revision(comment: &ReviewCommentDraft) -> String {
    match comment.anchor.side {
        AnchorSide::Base => comment.anchor.base_sha.clone(),
        AnchorSide::Head => comment.source_head_sha.clone(),
    }
}

fn anchored_path(comment: &ReviewCommentDraft) -> Result<String, BuildClipboardExportError> {
    match comment.anchor.side {
        AnchorSide::Base => comment.anchor.old_path.clone(),
        AnchorSide::Head => comment.anchor.new_path.clone(),
    }
    .or_else(|| comment.anchor.new_path.clone())
    .or_else(|| comment.anchor.old_path.clone())
    .ok_or(BuildClipboardExportError::MissingAnchoredPath)
}

fn format_comment_section(comment: &ReviewCommentDraft, path: &str, content: &str) -> String {
    let side = match comment.anchor.side {
        AnchorSide::Base => "base",
        AnchorSide::Head => "head",
    };
    let mut section = vec![
        format!("{path}:{} [{side}]", comment.anchor.line_number),
        format!("Comment: {}", comment.body),
        "Context:".to_string(),
    ];

    for (line_number, text) in context_window(content, comment.anchor.line_number, 10) {
        section.push(format!("{line_number:>3} | {text}"));
    }

    section.join("\n")
}

fn context_window(content: &str, line_number: usize, radius: usize) -> Vec<(usize, String)> {
    let lines: Vec<&str> = content.split_terminator('\n').collect();
    let start = line_number.saturating_sub(radius + 1);
    let end = usize::min(lines.len(), line_number + radius);

    lines[start..end]
        .iter()
        .enumerate()
        .map(|(offset, text)| (start + offset + 1, (*text).to_string()))
        .collect()
}
