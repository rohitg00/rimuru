use serde::Serialize;
use std::process::Command;

#[derive(Debug, Serialize, Clone)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub head: String,
}

fn is_valid_branch_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains("..")
        && !name.contains('\0')
        && !name.contains(' ')
        && !name.starts_with('-')
        && !name.starts_with('.')
        && name.len() <= 128
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

pub fn create_worktree(repo_path: &str, branch_name: &str) -> Result<String, String> {
    if !is_valid_branch_name(branch_name) {
        return Err(format!(
            "Invalid branch name '{}'. Use only alphanumeric, dash, underscore, or dot characters.",
            branch_name
        ));
    }

    let repo = std::path::Path::new(repo_path);
    if !repo.is_dir() {
        return Err(format!("Repository path does not exist: {}", repo_path));
    }

    let worktree_path = format!("{}/.worktrees/{}", repo_path, branch_name);
    std::fs::create_dir_all(&format!("{}/.worktrees", repo_path))
        .map_err(|e| format!("Failed to create worktrees dir: {}", e))?;

    let output = Command::new("git")
        .args(["worktree", "add", &worktree_path, "-b", branch_name])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git worktree add failed: {}", stderr));
    }

    Ok(worktree_path)
}

pub fn cleanup_worktree(repo_path: &str, worktree_path: &str) -> Result<(), String> {
    let canonical_repo =
        std::fs::canonicalize(repo_path).map_err(|e| format!("Invalid repo path: {}", e))?;
    let worktrees_dir = canonical_repo.join(".worktrees");

    if let Ok(canonical_wt) = std::fs::canonicalize(worktree_path) {
        if !canonical_wt.starts_with(&worktrees_dir) {
            return Err(
                "Worktree path must be within the repository's .worktrees directory".to_string(),
            );
        }
    } else {
        let wt_path = std::path::Path::new(worktree_path);
        if wt_path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            return Err("Worktree path cannot contain '..' traversal".to_string());
        }
    }

    let output = Command::new("git")
        .args(["worktree", "remove", worktree_path, "--force"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git worktree remove failed: {}", stderr));
    }
    Ok(())
}

pub fn list_worktrees(repo_path: &str) -> Result<Vec<WorktreeInfo>, String> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_path = String::new();
    let mut current_head = String::new();
    let mut current_branch = String::new();

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = path.to_string();
        } else if let Some(head) = line.strip_prefix("HEAD ") {
            current_head = head.to_string();
        } else if let Some(branch) = line.strip_prefix("branch ") {
            current_branch = branch.replace("refs/heads/", "");
        } else if line.is_empty() && !current_path.is_empty() {
            worktrees.push(WorktreeInfo {
                path: current_path.clone(),
                branch: current_branch.clone(),
                head: current_head.clone(),
            });
            current_path.clear();
            current_head.clear();
            current_branch.clear();
        }
    }
    if !current_path.is_empty() {
        worktrees.push(WorktreeInfo {
            path: current_path,
            branch: current_branch,
            head: current_head,
        });
    }

    Ok(worktrees)
}
