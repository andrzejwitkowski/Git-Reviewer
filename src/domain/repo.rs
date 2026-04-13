#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoContext {
    pub head_sha: String,
    pub current_branch: String,
    pub remote_branches: Vec<String>,
    pub default_base_branch: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoStatusEntry {
    pub code: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCommitSummary {
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
    pub is_local_changes: bool,
}

impl ReviewCommitSummary {
    pub fn local_changes() -> Self {
        Self {
            sha: "LOCAL_CHANGES".to_string(),
            short_sha: String::new(),
            subject: "LOCAL CHANGES".to_string(),
            is_local_changes: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSnapshot {
    pub head_sha: String,
    pub current_branch: String,
    pub local_changes_sha: String,
    pub entries: Vec<RepoStatusEntry>,
}

impl RepoSnapshot {
    pub fn new(
        head_sha: String,
        current_branch: String,
        local_changes_sha: String,
        entries: Vec<RepoStatusEntry>,
    ) -> Self {
        Self {
            head_sha,
            current_branch,
            local_changes_sha,
            entries,
        }
    }
}

impl RepoContext {
    pub fn new(
        head_sha: String,
        current_branch: String,
        remote_branches: Vec<String>,
        default_base_branch: String,
    ) -> Self {
        Self {
            head_sha,
            current_branch,
            remote_branches,
            default_base_branch,
        }
    }
}
