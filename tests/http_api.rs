mod support;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use git_reviewer::bootstrap::http_server::build_app;
use serde_json::json;
use serde_json::Value;
use support::git_repo::TempGitRepo;
use tower::util::ServiceExt;

#[tokio::test]
async fn serves_embedded_shell_at_root() {
    let repo = sample_repo();
    let app = build_app(repo.path());

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let html = String::from_utf8(body.to_vec()).expect("html should be utf8");

    assert!(html.contains("data-testid=\"repo-path\""));
    assert!(html.contains("/assets/app.js"));
}

#[tokio::test]
async fn returns_repo_context_review_and_status_from_http_api() {
    let repo = sample_repo();
    let app = build_app(repo.path());

    let repo_context = read_json(&app, "/api/repo-context").await;
    assert_eq!(repo_context["currentBranch"], "feature/http-shell");
    assert_eq!(repo_context["defaultBaseBranch"], "origin/main");
    assert_eq!(
        repo_context["remoteBranches"],
        serde_json::json!(["origin/main"])
    );
    assert_eq!(repo_context["repoPath"], repo.path().display().to_string());

    let review = read_json(&app, "/api/review?base=origin/main").await;
    assert_eq!(review["baseBranch"], "origin/main");
    assert_eq!(review["reviewMode"], "branch");
    assert_eq!(review["selectedCommit"], Value::Null);
    assert_eq!(review["files"].as_array().map(Vec::len), Some(2));
    assert_eq!(review["files"][0]["path"], "notes/guide.md");
    assert_eq!(review["files"][1]["path"], "src/lib.rs");
    assert!(review["files"][0]["hunks"][0]["lines"]
        .as_array()
        .expect("lines should be an array")
        .iter()
        .any(|line| line["text"] == "updated note"));

    let status = read_json(&app, "/api/status").await;
    assert_eq!(status["currentBranch"], "feature/http-shell");
    assert_eq!(status["headSha"], repo.head_sha());
    assert!(status["localChangesSha"].as_str().is_some());
    assert!(status["entries"]
        .as_array()
        .expect("status entries should be an array")
        .iter()
        .any(|entry| entry["path"] == "scratch.txt"));
}

#[tokio::test]
async fn returns_large_file_placeholder_until_full_file_is_requested() {
    let repo = TempGitRepo::new();
    repo.write_file("src/huge.rs", &large_source(820, "feature marker"));
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/large-file");
    repo.write_file("src/huge.rs", &large_source(820, "feature marker updated"));
    repo.commit_all("head");

    let app = build_app(repo.path());

    let review = read_json(&app, "/api/review?base=origin/main").await;
    let file = &review["files"][0];

    assert_eq!(file["path"], "src/huge.rs");
    assert_eq!(file["isLarge"], true);
    assert_eq!(file["isLoaded"], false);
    assert_eq!(file["lineCount"], 820);
    assert!(file["hunks"]
        .as_array()
        .expect("hunks should be an array")
        .is_empty());

    let expanded = read_json(&app, "/api/review?base=origin/main&expand=src%2Fhuge.rs").await;
    let expanded_file = &expanded["files"][0];

    assert_eq!(expanded_file["isLarge"], true);
    assert_eq!(expanded_file["isLoaded"], true);
    assert!(expanded_file["hunks"]
        .as_array()
        .expect("hunks should be an array")
        .iter()
        .any(|hunk| hunk["lines"]
            .as_array()
            .expect("lines should be an array")
            .iter()
            .any(|line| line["text"] == "feature marker updated")));
}

#[tokio::test]
async fn returns_error_payload_for_unknown_base_branch() {
    let repo = sample_repo();
    let app = build_app(repo.path());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/review?base=origin/missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let error: Value = serde_json::from_slice(&body).expect("error should be valid json");

    assert_eq!(error["error"], "invalid base branch");
}

#[tokio::test]
async fn returns_commits_for_selected_base_branch() {
    let repo = sample_repo();
    let app = build_app(repo.path());

    let commits = read_json(&app, "/api/commits?base=origin/main").await;

    assert_eq!(commits.as_array().map(Vec::len), Some(2));
    assert_eq!(commits[0]["subject"], "LOCAL CHANGES");
    assert_eq!(commits[0]["shortSha"], "");
    assert_eq!(commits[1]["subject"], "head");
    assert_eq!(commits[1]["shortSha"].as_str().map(str::len), Some(7));
}

#[tokio::test]
async fn returns_single_commit_review_from_http_api() {
    let repo = sample_repo();
    let app = build_app(repo.path());
    let commits = read_json(&app, "/api/commits?base=origin/main").await;
    let selected_commit = commits[1]["sha"].as_str().expect("commit sha should exist");

    let review = read_json(
        &app,
        &format!("/api/review?mode=commit&base=origin/main&commit={selected_commit}"),
    )
    .await;

    assert_eq!(review["reviewMode"], "commit");
    assert_eq!(review["baseBranch"], "origin/main");
    assert_eq!(review["selectedCommit"], selected_commit);
    assert_eq!(review["headSha"], selected_commit);
}

#[tokio::test]
async fn returns_local_changes_review_from_http_api() {
    let repo = sample_repo();
    repo.write_file("src/lib.rs", "base\nupdated in worktree\n");
    let app = build_app(repo.path());

    let review = read_json(
        &app,
        "/api/review?mode=commit&base=origin/main&commit=LOCAL_CHANGES",
    )
    .await;

    assert_eq!(review["reviewMode"], "commit");
    assert_eq!(review["selectedCommit"], "LOCAL_CHANGES");
    assert!(review["files"]
        .as_array()
        .expect("files should be an array")
        .iter()
        .any(|file| file["path"] == "src/lib.rs"));
}

#[tokio::test]
async fn returns_error_payload_for_unknown_commit_review() {
    let repo = sample_repo();
    let app = build_app(repo.path());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/review?mode=commit&base=origin/main&commit=deadbeef")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let error: Value = serde_json::from_slice(&body).expect("error should be valid json");

    assert_eq!(error["error"], "invalid commit");
}

#[tokio::test]
async fn returns_error_payload_for_invalid_review_mode() {
    let repo = sample_repo();
    let app = build_app(repo.path());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/review?mode=bogus&base=origin/main")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let error: Value = serde_json::from_slice(&body).expect("error should be valid json");

    assert_eq!(error["error"], "invalid review mode");
}

#[tokio::test]
async fn returns_error_payload_for_unknown_base_branch_commit_list() {
    let repo = sample_repo();
    let app = build_app(repo.path());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/commits?base=origin/missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let error: Value = serde_json::from_slice(&body).expect("error should be valid json");

    assert_eq!(error["error"], "invalid base branch");
}

#[tokio::test]
async fn rejects_clipboard_export_requests_with_invalid_side() {
    let repo = sample_repo();
    let app = build_app(repo.path());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/clipboard-export")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "headSha": repo.head_sha(),
                        "comments": [{
                            "sourceHeadSha": repo.head_sha(),
                            "body": "note",
                            "anchor": {
                                "baseSha": repo.head_sha(),
                                "oldPath": "src/lib.rs",
                                "newPath": "src/lib.rs",
                                "side": "bogus",
                                "lineNumber": 2
                            }
                        }]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let error: Value = serde_json::from_slice(&body).expect("error should be valid json");

    assert_eq!(error["error"], "invalid clipboard export request");
}

async fn read_json(app: &axum::Router, uri: &str) -> Value {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK, "uri {uri}");

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    serde_json::from_slice(&body).expect("response should be valid json")
}

fn sample_repo() -> TempGitRepo {
    let repo = TempGitRepo::new();

    repo.write_file(
        "src/lib.rs",
        "pub fn value() -> &'static str {\n    \"base\"\n}\n",
    );
    repo.write_file("notes/guide.md", "base note\n");
    let base_sha = repo.commit_all("base");
    repo.create_remote_tracking_branch("origin/main", &base_sha);
    repo.checkout_new_branch("feature/http-shell");

    repo.write_file(
        "src/lib.rs",
        "pub fn value() -> &'static str {\n    \"head\"\n}\n",
    );
    repo.write_file("notes/guide.md", "updated note\n");
    repo.commit_all("head");
    repo.write_file("scratch.txt", "worktree change\n");

    repo
}

fn large_source(total_lines: usize, marker: &str) -> String {
    let mut lines = Vec::with_capacity(total_lines);
    for index in 0..total_lines {
        if index == 410 {
            lines.push(marker.to_string());
        } else {
            lines.push(format!("line {}", index + 1));
        }
    }
    format!("{}\n", lines.join("\n"))
}
