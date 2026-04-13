mod support;

use git_reviewer::adapters::outbound::git_cli::GitCliRepository;
use git_reviewer::application::use_cases::build_clipboard_export::{
    BuildClipboardExport, BuildClipboardExportRequest,
};
use git_reviewer::application::use_cases::get_repo_status::GetRepoStatus;
use git_reviewer::application::use_cases::load_review::{LoadReview, LoadReviewRequest};
use git_reviewer::domain::review::{AnchorSide, CommentAnchor, ReviewCommentDraft};
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
            selected_base: Some("origin/main".to_string()),
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
