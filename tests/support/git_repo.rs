use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[allow(dead_code)]
pub struct TempGitRepo {
    dir: TempDir,
}

#[allow(dead_code)]
impl TempGitRepo {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let repo = Self { dir };

        repo.run(&["init", "-b", "main"]);
        repo.run(&["config", "user.name", "Git Reviewer"]);
        repo.run(&["config", "user.email", "git-reviewer@example.com"]);

        repo
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write_file(&self, path: &str, content: &str) {
        let file_path = self.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("parent dirs should be created");
        }
        fs::write(file_path, content).expect("file should be written");
    }

    pub fn write_binary_file(&self, path: &str, content: &[u8]) {
        let file_path = self.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("parent dirs should be created");
        }
        fs::write(file_path, content).expect("binary file should be written");
    }

    pub fn remove_file(&self, path: &str) {
        fs::remove_file(self.path().join(path)).expect("file should be removed");
    }

    pub fn rename(&self, from: &str, to: &str) {
        let destination = self.path().join(to);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).expect("parent dirs should be created");
        }
        fs::rename(self.path().join(from), destination).expect("file should be renamed");
    }

    pub fn checkout_new_branch(&self, name: &str) {
        self.run(&["checkout", "-b", name]);
    }

    pub fn checkout(&self, name: &str) {
        self.run(&["checkout", name]);
    }

    pub fn commit_all(&self, message: &str) -> String {
        self.run(&["add", "-A"]);
        self.run(&["commit", "-m", message]);
        self.run(&["rev-parse", "HEAD"]).trim().to_string()
    }

    pub fn create_remote_tracking_branch(&self, name: &str, target: &str) {
        self.run(&["update-ref", &format!("refs/remotes/{name}"), target]);
    }

    pub fn head_sha(&self) -> String {
        self.run(&["rev-parse", "HEAD"]).trim().to_string()
    }

    pub fn run(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(self.path())
            .output()
            .expect("git command should execute");

        if !output.status.success() {
            panic!(
                "git command failed: git {}\nstdout:\n{}\nstderr:\n{}",
                args.join(" "),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }

        String::from_utf8(output.stdout).expect("stdout should be utf8")
    }
}
