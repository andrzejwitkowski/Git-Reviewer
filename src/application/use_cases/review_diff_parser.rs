use crate::application::ports::git_repository::RawReviewDiff;
use crate::domain::review::{DiffChange, DiffHunk, DiffLine, DiffLineKind, Review, ReviewFile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseReviewError {
    InvalidHunkHeader,
}

pub fn parse_review(raw: RawReviewDiff) -> Result<Review, ParseReviewError> {
    let mut parser = ReviewParser::default();

    for line in raw.diff.lines() {
        parser.push_line(line)?;
    }

    Ok(Review {
        base_branch: raw.base_branch,
        merge_base_sha: raw.merge_base_sha,
        head_sha: raw.head_sha,
        files: parser.finish(),
    })
}

#[derive(Default)]
struct ReviewParser {
    files: Vec<ReviewFile>,
    current_file: Option<FileBuilder>,
    current_hunk: Option<HunkState>,
}

impl ReviewParser {
    fn push_line(&mut self, line: &str) -> Result<(), ParseReviewError> {
        if let Some(diff_line) = line.strip_prefix("diff --git ") {
            self.flush_hunk();
            self.flush_file();
            self.current_file = Some(FileBuilder::new(diff_line));
            return Ok(());
        }

        let Some(file) = self.current_file.as_mut() else {
            return Ok(());
        };

        if let Some(path) = line.strip_prefix("rename from ") {
            file.old_path = Some(path.to_string());
            file.change = Some(DiffChange::Renamed);
            return Ok(());
        }

        if let Some(path) = line.strip_prefix("rename to ") {
            file.new_path = Some(path.to_string());
            file.change = Some(DiffChange::Renamed);
            return Ok(());
        }

        if line == "new file mode 100644" || line.starts_with("new file mode ") {
            file.change = Some(DiffChange::Added);
            return Ok(());
        }

        if line == "deleted file mode 100644" || line.starts_with("deleted file mode ") {
            file.change = Some(DiffChange::Deleted);
            return Ok(());
        }

        if line.starts_with("Binary files ") || line == "GIT binary patch" {
            file.is_binary = true;
            return Ok(());
        }

        if let Some(path) = line.strip_prefix("--- ") {
            file.old_path = parse_patch_path(path);
            return Ok(());
        }

        if let Some(path) = line.strip_prefix("+++ ") {
            file.new_path = parse_patch_path(path);
            return Ok(());
        }

        if let Some(header) = line.strip_prefix("@@ ") {
            self.flush_hunk();
            self.current_hunk = Some(HunkState::new(header)?);
            return Ok(());
        }

        if line == r"\ No newline at end of file" {
            return Ok(());
        }

        if let Some(hunk) = self.current_hunk.as_mut() {
            hunk.push_line(line);
        }

        Ok(())
    }

    fn flush_hunk(&mut self) {
        if let Some(hunk) = self.current_hunk.take() {
            if let Some(file) = self.current_file.as_mut() {
                file.hunks.push(hunk.finish());
            }
        }
    }

    fn flush_file(&mut self) {
        if let Some(file) = self.current_file.take() {
            self.files.push(file.finish());
        }
    }

    fn finish(mut self) -> Vec<ReviewFile> {
        self.flush_hunk();
        self.flush_file();
        self.files
    }
}

struct FileBuilder {
    old_path: Option<String>,
    new_path: Option<String>,
    change: Option<DiffChange>,
    is_binary: bool,
    hunks: Vec<DiffHunk>,
}

impl FileBuilder {
    fn new(diff_line: &str) -> Self {
        let (old_path, new_path) = parse_diff_git_paths(diff_line);

        Self {
            old_path,
            new_path,
            change: None,
            is_binary: false,
            hunks: Vec::new(),
        }
    }

    fn finish(self) -> ReviewFile {
        let change = self
            .change
            .unwrap_or_else(|| match (&self.old_path, &self.new_path) {
                (None, Some(_)) => DiffChange::Added,
                (Some(_), None) => DiffChange::Deleted,
                (Some(old_path), Some(new_path)) if old_path != new_path => DiffChange::Renamed,
                _ => DiffChange::Modified,
            });

        ReviewFile {
            old_path: self.old_path,
            new_path: self.new_path,
            change,
            is_binary: self.is_binary,
            is_large: false,
            is_loaded: true,
            line_count: 0,
            hunks: self.hunks,
        }
    }
}

struct HunkState {
    hunk: DiffHunk,
    next_old_line: usize,
    next_new_line: usize,
}

impl HunkState {
    fn new(header: &str) -> Result<Self, ParseReviewError> {
        let mut parts = header.split_whitespace();
        let old_range = parts.next().ok_or(ParseReviewError::InvalidHunkHeader)?;
        let new_range = parts.next().ok_or(ParseReviewError::InvalidHunkHeader)?;
        let (old_start, old_count) = parse_range(old_range)?;
        let (new_start, new_count) = parse_range(new_range)?;

        Ok(Self {
            hunk: DiffHunk {
                old_start,
                old_count,
                new_start,
                new_count,
                lines: Vec::new(),
            },
            next_old_line: old_start,
            next_new_line: new_start,
        })
    }

    fn push_line(&mut self, line: &str) {
        let mut chars = line.chars();
        let Some(prefix) = chars.next() else {
            return;
        };
        let text = chars.as_str().to_string();

        match prefix {
            ' ' => {
                self.hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Context,
                    old_line_number: Some(self.next_old_line),
                    new_line_number: Some(self.next_new_line),
                    text,
                });
                self.next_old_line += 1;
                self.next_new_line += 1;
            }
            '+' => {
                self.hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Added,
                    old_line_number: None,
                    new_line_number: Some(self.next_new_line),
                    text,
                });
                self.next_new_line += 1;
            }
            '-' => {
                self.hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Removed,
                    old_line_number: Some(self.next_old_line),
                    new_line_number: None,
                    text,
                });
                self.next_old_line += 1;
            }
            _ => {}
        }
    }

    fn finish(self) -> DiffHunk {
        self.hunk
    }
}

fn parse_patch_path(path: &str) -> Option<String> {
    let normalized = normalize_diff_path(path);

    if normalized == "/dev/null" {
        None
    } else {
        Some(
            normalized
                .trim_start_matches("a/")
                .trim_start_matches("b/")
                .to_string(),
        )
    }
}

fn parse_diff_git_paths(line: &str) -> (Option<String>, Option<String>) {
    let paths = split_diff_git_paths(line);
    let old_path = paths.first().and_then(|path| parse_patch_path(path));
    let new_path = paths.get(1).and_then(|path| parse_patch_path(path));
    (old_path, new_path)
}

fn split_diff_git_paths(line: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    current.push(match next {
                        't' => '\t',
                        'n' => '\n',
                        '"' => '"',
                        '\\' => '\\',
                        other => other,
                    });
                }
            } else if ch == '"' {
                in_quotes = false;
                paths.push(current.clone());
                current.clear();
            } else {
                current.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == ' ' {
            if !current.is_empty() {
                paths.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        paths.push(current);
    }

    paths
}

fn normalize_diff_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('\t').trim_end();

    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        decode_quoted_path(&trimmed[1..trimmed.len() - 1])
    } else {
        trimmed.to_string()
    }
}

fn decode_quoted_path(path: &str) -> String {
    let mut decoded = String::new();
    let mut chars = path.chars();

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

fn parse_range(range: &str) -> Result<(usize, usize), ParseReviewError> {
    let trimmed = range.trim_start_matches('-').trim_start_matches('+');
    let mut parts = trimmed.split(',');
    let start = parts
        .next()
        .ok_or(ParseReviewError::InvalidHunkHeader)?
        .parse()
        .map_err(|_| ParseReviewError::InvalidHunkHeader)?;
    let count = match parts.next() {
        Some(value) => value
            .parse()
            .map_err(|_| ParseReviewError::InvalidHunkHeader)?,
        None => 1,
    };

    Ok((start, count))
}

#[cfg(test)]
mod tests {
    use super::parse_review;
    use crate::application::ports::git_repository::RawReviewDiff;

    #[test]
    fn parses_quoted_diff_paths_and_patch_lines_with_trailing_tabs() {
        let review = parse_review(RawReviewDiff {
            base_branch: "origin/main".to_string(),
            merge_base_sha: "base".to_string(),
            head_sha: "head".to_string(),
            diff: concat!(
                "diff --git \"a/old name.txt\" \"b/new name.txt\"\n",
                "rename from old name.txt\n",
                "rename to new name.txt\n",
                "--- \"a/old name.txt\"\t\n",
                "+++ \"b/new name.txt\"\t\n",
                "@@ -1 +1 @@\n",
                "-before\n",
                "+after\n"
            )
            .to_string(),
        })
        .expect("review should parse");

        let file = review.files.first().expect("file should exist");
        assert_eq!(file.old_path.as_deref(), Some("old name.txt"));
        assert_eq!(file.new_path.as_deref(), Some("new name.txt"));
    }
}
