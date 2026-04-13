mod support;

use git_reviewer::adapters::outbound::git_cli::GitCliRepository;
use git_reviewer::application::use_cases::build_clipboard_export::{
    BuildClipboardExport, BuildClipboardExportRequest,
};
use git_reviewer::application::use_cases::get_repo_status::GetRepoStatus;
use git_reviewer::application::use_cases::load_review::{
    LoadReview, LoadReviewError, LoadReviewRequest,
};
use git_reviewer::domain::review::{AnchorSide, CommentAnchor, ReviewCommentDraft, ReviewMode};
use support::git_repo::TempGitRepo;

#[test]
fn build_clipboard_export_uses_base_for_removed_lines_and_worktree_for_added_lines() {
    let repo = TempGitRepo::new();

    repo.write_file(
        "sample.txt",
        "line 1\nline 2\nold target\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\n",
    );
    let base_sha = repo.commit_all("base");
    repo.checkout_new_branch("feature/review");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.write_file(
        "sample.txt",
        "line 1\nline 2\nnew target\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\n",
    );
    let head_sha = repo.commit_all("head");
    repo.write_file(
        "sample.txt",
        "line 1\nline 2\nnew target\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13 worktree\n",
    );

    let git = GitCliRepository::new(repo.path());
    let review = LoadReview::new(&git)
        .execute(LoadReviewRequest {
            review_mode: ReviewMode::Branch,
            selected_base: Some("origin/main".to_string()),
            selected_commit: None,
            expanded_paths: vec![],
        })
        .expect("review should load");

    let file = review.files.first().expect("review file should exist");
    let hunk = file.hunks.first().expect("review hunk should exist");
    let removed = hunk
        .lines
        .iter()
        .find(|line| line.text == "old target")
        .expect("removed line should exist");
    let added = hunk
        .lines
        .iter()
        .find(|line| line.text == "new target")
        .expect("added line should exist");

    let export = BuildClipboardExport::new(&git)
        .execute(BuildClipboardExportRequest {
            head_sha: head_sha.clone(),
            comments: vec![
                ReviewCommentDraft {
                    anchor: CommentAnchor {
                        base_sha: review.merge_base_sha.clone(),
                        old_path: file.old_path.clone(),
                        new_path: file.new_path.clone(),
                        side: AnchorSide::Base,
                        line_number: removed.old_line_number.expect("old number should exist"),
                    },
                    source_head_sha: head_sha.clone(),
                    body: "removed line comment".to_string(),
                },
                ReviewCommentDraft {
                    anchor: CommentAnchor {
                        base_sha: review.merge_base_sha.clone(),
                        old_path: file.old_path.clone(),
                        new_path: file.new_path.clone(),
                        side: AnchorSide::Head,
                        line_number: added.new_line_number.expect("new number should exist"),
                    },
                    source_head_sha: head_sha.clone(),
                    body: "added line comment".to_string(),
                },
            ],
        })
        .expect("clipboard export should build");

    assert!(export.contains(&format!("sample.txt - {head_sha}")));
    assert!(export.contains("removed line comment"));
    assert!(export.contains("added line comment"));
    assert!(export.contains("old target"));
    assert!(export.contains("new target"));
    assert!(export.contains("line 12"));
    assert!(!export.contains("HEAD sample.txt:3 [base]\n  3 | new target"));
}

#[test]
fn get_repo_status_returns_snapshot_from_port() {
    let repo = TempGitRepo::new();

    repo.write_file("tracked.txt", "base\n");
    repo.commit_all("base");
    repo.write_file("tracked.txt", "changed\n");

    let git = GitCliRepository::new(repo.path());
    let snapshot = GetRepoStatus::new(&git)
        .execute()
        .expect("snapshot should load");

    assert_eq!(snapshot.entries.len(), 1);
    assert_eq!(snapshot.entries[0].code, " M");
    assert_eq!(snapshot.entries[0].path, "tracked.txt");
}

#[test]
fn build_clipboard_export_uses_old_path_for_base_side_renamed_file_comments() {
    let repo = TempGitRepo::new();

    repo.write_file("old name.txt", "line 1\nrename target\nline 3\n");
    let base_sha = repo.commit_all("base");
    repo.checkout_new_branch("feature/review");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.rename("old name.txt", "new name.txt");
    let head_sha = repo.commit_all("rename");

    let git = GitCliRepository::new(repo.path());
    let export = BuildClipboardExport::new(&git)
        .execute(BuildClipboardExportRequest {
            head_sha: head_sha.clone(),
            comments: vec![ReviewCommentDraft {
                anchor: CommentAnchor {
                    base_sha: base_sha.clone(),
                    old_path: Some("old name.txt".to_string()),
                    new_path: Some("new name.txt".to_string()),
                    side: AnchorSide::Base,
                    line_number: 2,
                },
                source_head_sha: head_sha.clone(),
                body: "rename comment".to_string(),
            }],
        })
        .expect("clipboard export should build");

    assert!(export.contains(&format!("old name.txt - {head_sha}")));
    assert!(export.contains("old name.txt:2 [base]"));
    assert!(export.contains("rename target"));
}

#[test]
fn build_clipboard_export_groups_comments_by_file_and_head_sha() {
    let repo = TempGitRepo::new();

    repo.write_file("src/lib.rs", "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\n");
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/review");
    repo.write_file("src/lib.rs", "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nhead line\nline 13\n");
    let head_sha = repo.commit_all("head");

    let git = GitCliRepository::new(repo.path());
    let export = BuildClipboardExport::new(&git)
        .execute(BuildClipboardExportRequest {
            head_sha: head_sha.clone(),
            comments: vec![
                ReviewCommentDraft {
                    anchor: CommentAnchor {
                        base_sha: base_sha.clone(),
                        old_path: Some("src/lib.rs".to_string()),
                        new_path: Some("src/lib.rs".to_string()),
                        side: AnchorSide::Head,
                        line_number: 12,
                    },
                    source_head_sha: head_sha.clone(),
                    body: "first".to_string(),
                },
                ReviewCommentDraft {
                    anchor: CommentAnchor {
                        base_sha,
                        old_path: Some("src/lib.rs".to_string()),
                        new_path: Some("src/lib.rs".to_string()),
                        side: AnchorSide::Head,
                        line_number: 3,
                    },
                    source_head_sha: head_sha.clone(),
                    body: "second".to_string(),
                },
            ],
        })
        .expect("clipboard export should build");

    assert!(export.contains(&format!("src/lib.rs - {head_sha}")));
    assert_eq!(export.matches("src/lib.rs - ").count(), 1);
    assert!(export.contains("Comment: first"));
    assert!(export.contains("Comment: second"));
    assert!(export.contains("  2 | line 2"));
    assert!(export.contains(" 13 | line 13"));
}

#[test]
fn load_review_commit_mode_returns_only_selected_commit_changes() {
    let repo = TempGitRepo::new();

    repo.write_file("src/lib.rs", "base\n");
    repo.write_file("docs/readme.md", "base docs\n");
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/review");

    repo.write_file("src/lib.rs", "first feature line\n");
    repo.commit_all("first feature");
    repo.write_file("docs/readme.md", "second feature line\n");
    let selected_sha = repo.commit_all("second feature");

    let git = GitCliRepository::new(repo.path());
    let review = LoadReview::new(&git)
        .execute(LoadReviewRequest {
            review_mode: ReviewMode::Commit,
            selected_base: Some("origin/main".to_string()),
            selected_commit: Some(selected_sha.clone()),
            expanded_paths: vec![],
        })
        .expect("commit review should load");

    assert_eq!(review.review_mode, ReviewMode::Commit);
    assert_eq!(review.base_branch, "origin/main");
    assert_eq!(review.selected_commit, Some(selected_sha.clone()));
    assert_eq!(review.head_sha, selected_sha);
    assert_eq!(review.files.len(), 1);
    assert_eq!(review.files[0].path(), "docs/readme.md");
    assert!(review.files[0]
        .hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .any(|line| line.text == "second feature line"));
}

#[test]
fn load_review_commit_mode_rejects_invalid_selected_commit() {
    let repo = TempGitRepo::new();

    repo.write_file("src/lib.rs", "base\n");
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/review");
    repo.write_file("src/lib.rs", "feature\n");
    repo.commit_all("feature");

    let git = GitCliRepository::new(repo.path());
    let error = LoadReview::new(&git)
        .execute(LoadReviewRequest {
            review_mode: ReviewMode::Commit,
            selected_base: Some("origin/main".to_string()),
            selected_commit: Some("deadbeef".to_string()),
            expanded_paths: vec![],
        })
        .expect_err("invalid commit should fail");

    assert_eq!(error, LoadReviewError::InvalidCommit);
}

#[test]
fn load_review_commit_mode_returns_local_changes_for_sentinel_selection() {
    let repo = TempGitRepo::new();

    repo.write_file("src/lib.rs", "base\n");
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/review");
    repo.write_file("src/lib.rs", "base\nlocal change\n");

    let git = GitCliRepository::new(repo.path());
    let review = LoadReview::new(&git)
        .execute(LoadReviewRequest {
            review_mode: ReviewMode::Commit,
            selected_base: Some("origin/main".to_string()),
            selected_commit: Some("LOCAL_CHANGES".to_string()),
            expanded_paths: vec![],
        })
        .expect("local changes review should load");

    assert_eq!(review.review_mode, ReviewMode::Commit);
    assert_eq!(review.base_branch, "origin/main");
    assert_eq!(review.selected_commit, Some("LOCAL_CHANGES".to_string()));
    assert_eq!(review.head_sha, base_sha);
    assert_eq!(review.files.len(), 1);
    assert!(review.files[0]
        .hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .any(|line| line.text == "local change"));
}

#[test]
fn load_review_commit_mode_includes_untracked_files_for_local_changes() {
    let repo = TempGitRepo::new();

    repo.write_file("src/lib.rs", "base\n");
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/review");
    repo.write_file("notes/new.txt", "untracked file\n");

    let git = GitCliRepository::new(repo.path());
    let review = LoadReview::new(&git)
        .execute(LoadReviewRequest {
            review_mode: ReviewMode::Commit,
            selected_base: Some("origin/main".to_string()),
            selected_commit: Some("LOCAL_CHANGES".to_string()),
            expanded_paths: vec![],
        })
        .expect("local changes review should load");

    assert!(review
        .files
        .iter()
        .any(|file| file.path() == "notes/new.txt"));
}

#[test]
fn build_clipboard_export_uses_worktree_content_for_local_changes_untracked_file_comments() {
    let repo = TempGitRepo::new();

    repo.write_file("src/lib.rs", "base\n");
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/review");
    repo.write_file("notes/new.txt", "line 1\nuntracked body\nline 3\n");

    let git = GitCliRepository::new(repo.path());
    let review = LoadReview::new(&git)
        .execute(LoadReviewRequest {
            review_mode: ReviewMode::Commit,
            selected_base: Some("origin/main".to_string()),
            selected_commit: Some("LOCAL_CHANGES".to_string()),
            expanded_paths: vec![],
        })
        .expect("local changes review should load");
    let file = review
        .files
        .iter()
        .find(|entry| entry.path() == "notes/new.txt")
        .expect("untracked file should be in review");
    let added = file
        .hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .find(|line| line.text == "untracked body")
        .expect("added line should exist");

    let export = BuildClipboardExport::new(&git)
        .execute(BuildClipboardExportRequest {
            head_sha: review.head_sha.clone(),
            comments: vec![ReviewCommentDraft {
                anchor: CommentAnchor {
                    base_sha: review.merge_base_sha.clone(),
                    old_path: file.old_path.clone(),
                    new_path: file.new_path.clone(),
                    side: AnchorSide::Head,
                    line_number: added.new_line_number.expect("new line number should exist"),
                },
                source_head_sha: "WORKTREE".to_string(),
                body: "note on untracked file".to_string(),
            }],
        })
        .expect("clipboard export should build");

    assert!(export.contains("notes/new.txt - "));
    assert!(export.contains("note on untracked file"));
    assert!(export.contains("untracked body"));
}
