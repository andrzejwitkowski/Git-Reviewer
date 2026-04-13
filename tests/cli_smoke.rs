mod support;

use assert_cmd::Command;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command as StdCommand, Stdio};
use support::git_repo::TempGitRepo;

#[test]
fn help_exits_successfully() {
    let output = Command::cargo_bin("git-reviewer")
        .expect("binary should build")
        .arg("--help")
        .output()
        .expect("help command should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("git-reviewer"));
}

#[test]
fn missing_directory_returns_failure() {
    let output = Command::cargo_bin("git-reviewer")
        .expect("binary should build")
        .arg("/definitely/missing/path")
        .output()
        .expect("binary should run");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("does not exist"));
}

#[test]
fn non_git_directory_returns_failure() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");

    let output = Command::cargo_bin("git-reviewer")
        .expect("binary should build")
        .arg(temp_dir.path())
        .output()
        .expect("binary should run");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("is not a git repository"));
}

#[test]
fn subdirectory_inside_git_repository_starts_server_and_prints_url() {
    let repo = TempGitRepo::new();
    let nested_dir = repo.path().join("nested/path");
    fs::create_dir_all(&nested_dir).expect("nested dir should be created");

    let mut child = StdCommand::new(env!("CARGO_BIN_EXE_git-reviewer"))
        .arg(&nested_dir)
        .arg("--port")
        .arg("0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary should start");

    let stdout = child.stdout.take().expect("stdout should be piped");
    let mut stdout = BufReader::new(stdout);
    let mut line = String::new();
    stdout
        .read_line(&mut line)
        .expect("stdout should be readable");

    assert!(line.contains("http://127.0.0.1:"), "stdout: {line}");

    child.kill().expect("server should stop");
    child.wait().expect("child should exit");
}
