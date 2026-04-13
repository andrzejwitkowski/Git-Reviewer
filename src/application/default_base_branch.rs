#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultBaseBranchError {
    MissingRemoteBranches,
}

pub fn resolve_default_base_branch(
    remote_branches: &[String],
) -> Result<String, DefaultBaseBranchError> {
    if remote_branches.iter().any(|branch| branch == "origin/main") {
        return Ok("origin/main".to_string());
    }

    if remote_branches
        .iter()
        .any(|branch| branch == "origin/master")
    {
        return Ok("origin/master".to_string());
    }

    remote_branches
        .first()
        .cloned()
        .ok_or(DefaultBaseBranchError::MissingRemoteBranches)
}
