mod support;

use git_reviewer::adapters::outbound::git_cli::GitCliRepository;
use git_reviewer::application::ports::git_repository::{GitRepository, GitRepositoryError};
use git_reviewer::application::use_cases::load_review::{LoadReview, LoadReviewRequest};
use git_reviewer::domain::review::{DiffChange, ReviewMode};
use support::git_repo::TempGitRepo;

#[test]
fn git_cli_repository_lists_branch_commits_newest_first() {
    let repo = TempGitRepo::new();

    repo.write_file("notes.txt", "base\n");
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/review");

    repo.write_file("notes.txt", "base\nfirst\n");
    let first_sha = repo.commit_all("first feature commit");
    repo.write_file("notes.txt", "base\nfirst\nsecond\n");
    let second_sha = repo.commit_all("second feature commit");

    let repository = GitCliRepository::new(repo.path());
    let commits = repository
        .commit_summaries("origin/main")
        .expect("commit summaries should load");

    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].sha, second_sha);
    assert_eq!(commits[0].short_sha, second_sha[..7].to_string());
    assert_eq!(commits[0].subject, "second feature commit");
    assert_eq!(commits[1].sha, first_sha);
    assert_eq!(commits[1].subject, "first feature commit");
}

#[test]
fn git_cli_repository_returns_single_commit_diff() {
    let repo = TempGitRepo::new();

    repo.write_file("notes.txt", "base\n");
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/review");

    repo.write_file("notes.txt", "base\nfirst\n");
    repo.commit_all("first feature commit");
    repo.write_file("notes.txt", "base\nfirst\nsecond\n");
    let selected_sha = repo.commit_all("second feature commit");

    let repository = GitCliRepository::new(repo.path());
    let diff = repository
        .raw_commit_diff(&selected_sha)
        .expect("single commit diff should load");

    assert_eq!(diff.base_branch, "commit");
    assert_eq!(diff.head_sha, selected_sha);
    assert!(!diff.merge_base_sha.is_empty());
    assert!(diff.diff.contains("+second"));
    assert!(!diff.diff.contains("+first"));
}

#[test]
fn git_cli_repository_rejects_unknown_commit_for_single_commit_diff() {
    let repo = TempGitRepo::new();

    repo.write_file("notes.txt", "base\n");
    repo.commit_all("base");

    let repository = GitCliRepository::new(repo.path());
    let error = repository
        .raw_commit_diff("deadbeef")
        .expect_err("unknown commit should fail");

    assert_eq!(error, GitRepositoryError::InvalidCommit);
}

#[test]
fn git_cli_repository_returns_local_changes_diff() {
    let repo = TempGitRepo::new();

    repo.write_file("notes.txt", "base\n");
    let head_sha = repo.commit_all("base");
    repo.write_file("notes.txt", "base\nworktree\n");

    let repository = GitCliRepository::new(repo.path());
    let diff = repository
        .raw_local_changes_diff()
        .expect("local changes diff should load");

    assert_eq!(diff.base_branch, "local");
    assert_eq!(diff.merge_base_sha, head_sha);
    assert_eq!(diff.head_sha, head_sha);
    assert!(diff.diff.contains("+worktree"));
}

#[test]
fn load_review_uses_merge_base_and_parses_rename_delete_and_binary_changes() {
    let repo = TempGitRepo::new();

    repo.write_file("notes.txt", "base\ncommon\n");
    repo.write_file("rename_me.txt", "rename me\n");
    repo.write_file("gone.txt", "remove me\n");
    repo.write_binary_file("image.bin", &[0, 1, 2, 3]);
    let base_sha = repo.commit_all("base");

    repo.checkout_new_branch("feature/review");
    repo.rename("rename_me.txt", "renamed.txt");
    repo.write_file("notes.txt", "base\nfeature committed\ncommon\n");
    repo.commit_all("feature commit");

    repo.checkout("main");
    repo.write_file("notes.txt", "base\nmain only\ncommon\n");
    let main_sha = repo.commit_all("main commit");

    repo.checkout("feature/review");
    repo.create_remote_tracking_branch("origin/main", &main_sha);
    repo.write_file(
        "notes.txt",
        "base\nfeature committed\nfeature worktree\ncommon\n",
    );
    repo.remove_file("gone.txt");
    repo.write_binary_file("image.bin", &[9, 8, 7, 6]);

    let repository = GitCliRepository::new(repo.path());
    let review = LoadReview::new(&repository)
        .execute(LoadReviewRequest {
            review_mode: ReviewMode::Branch,
            selected_base: Some("origin/main".to_string()),
            selected_commit: None,
            expanded_paths: vec![],
        })
        .expect("review should load");

    assert_eq!(review.base_branch, "origin/main");
    assert_eq!(review.merge_base_sha, base_sha);

    let notes = review
        .files
        .iter()
        .find(|file| file.new_path.as_deref() == Some("notes.txt"))
        .expect("notes diff should exist");

    assert_eq!(notes.change, DiffChange::Modified);
    assert!(notes
        .hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .any(|line| line.text == "feature committed"));
    assert!(notes
        .hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .any(|line| line.text == "feature worktree"));
    assert!(!notes
        .hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .any(|line| line.text == "main only"));

    let renamed = review
        .files
        .iter()
        .find(|file| file.new_path.as_deref() == Some("renamed.txt"))
        .expect("rename diff should exist");
    assert_eq!(renamed.change, DiffChange::Renamed);
    assert_eq!(renamed.old_path.as_deref(), Some("rename_me.txt"));

    let deleted = review
        .files
        .iter()
        .find(|file| file.old_path.as_deref() == Some("gone.txt"))
        .expect("delete diff should exist");
    assert_eq!(deleted.change, DiffChange::Deleted);
    assert_eq!(deleted.new_path, None);

    let binary = review
        .files
        .iter()
        .find(|file| file.new_path.as_deref() == Some("image.bin"))
        .expect("binary diff should exist");
    assert!(binary.is_binary);
    assert!(binary.hunks.is_empty());
}

#[test]
fn git_cli_repository_returns_snapshot_and_file_contents() {
    let repo = TempGitRepo::new();

    repo.write_file("tracked.txt", "base\n");
    let head_sha = repo.commit_all("base");
    repo.write_file("tracked.txt", "worktree change\n");
    repo.write_file("untracked.txt", "new file\n");

    let repository = GitCliRepository::new(repo.path());
    let snapshot = repository.repo_snapshot().expect("snapshot should load");

    assert_eq!(snapshot.head_sha, head_sha);
    assert!(snapshot
        .entries
        .iter()
        .any(|entry| entry.code == " M" && entry.path == "tracked.txt"));
    assert!(snapshot
        .entries
        .iter()
        .any(|entry| entry.code == "??" && entry.path == "untracked.txt"));

    let base_content = repository
        .file_content_at_revision(&head_sha, "tracked.txt")
        .expect("base content should load");
    assert_eq!(base_content.as_deref(), Some("base\n"));

    let missing = repository
        .file_content_at_revision(&head_sha, "missing.txt")
        .expect("missing content should not error");
    assert_eq!(missing, None);
}

#[test]
fn repo_snapshot_local_changes_sha_changes_when_modified_content_changes() {
    let repo = TempGitRepo::new();

    repo.write_file("tracked.txt", "base\n");
    repo.commit_all("base");
    repo.write_file("tracked.txt", "first worktree change\n");

    let repository = GitCliRepository::new(repo.path());
    let first = repository
        .repo_snapshot()
        .expect("first snapshot should load");

    repo.write_file("tracked.txt", "second worktree change\n");

    let second = repository
        .repo_snapshot()
        .expect("second snapshot should load");

    assert_ne!(first.local_changes_sha, second.local_changes_sha);
}

#[test]
fn load_review_parses_paths_with_spaces_and_trailing_tabs_from_git_output() {
    let repo = TempGitRepo::new();

    repo.write_file("old name.txt", "before\n");
    let base_sha = repo.commit_all("base");
    repo.checkout_new_branch("feature/review");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.rename("old name.txt", "new name.txt");
    repo.write_file("new name.txt", "after\n");
    repo.commit_all("rename and edit");

    let repository = GitCliRepository::new(repo.path());
    let review = LoadReview::new(&repository)
        .execute(LoadReviewRequest {
            review_mode: ReviewMode::Branch,
            selected_base: Some("origin/main".to_string()),
            selected_commit: None,
            expanded_paths: vec![],
        })
        .expect("review should load");

    let added = review
        .files
        .iter()
        .find(|file| file.new_path.as_deref() == Some("new name.txt"))
        .expect("added diff should exist");
    let deleted = review
        .files
        .iter()
        .find(|file| file.old_path.as_deref() == Some("old name.txt"))
        .expect("deleted diff should exist");

    assert_eq!(added.old_path, None);
    assert_eq!(deleted.new_path, None);
    assert!(added
        .hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .any(|line| line.text == "after"));
    assert!(deleted
        .hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .any(|line| line.text == "before"));
}

#[test]
fn repo_snapshot_parses_renames_and_quoted_paths() {
    let repo = TempGitRepo::new();

    repo.write_file("before name.txt", "base\n");
    repo.commit_all("base");
    repo.rename("before name.txt", "after name.txt");
    repo.run(&["add", "-A"]);
    repo.write_file("spaced name.txt", "new file\n");

    let repository = GitCliRepository::new(repo.path());
    let snapshot = repository.repo_snapshot().expect("snapshot should load");

    assert!(snapshot
        .entries
        .iter()
        .any(|entry| entry.code == "R " && entry.path == "before name.txt -> after name.txt"));
    assert!(snapshot
        .entries
        .iter()
        .any(|entry| entry.code == "??" && entry.path == "spaced name.txt"));
}
