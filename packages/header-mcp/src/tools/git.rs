use anyhow::{bail, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

// ─── Parameter Schemas ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitInitParams {
    /// Directory to initialize git in (defaults to current workspace)
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitStatusParams {
    /// Repository path
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitAddCommitParams {
    /// Repository path
    pub path: String,
    /// Files to stage, e.g. [".", "src/main.rs"] (use ["."] for all)
    pub files: Vec<String>,
    /// Commit message
    pub message: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitCheckIgnoreParams {
    /// Repository path
    pub path: String,
    /// File/directory to check
    pub target: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitListTrackedParams {
    /// Repository path  
    pub path: String,
}

// ─── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, JsonSchema)]
pub struct GitInitResult {
    pub success: bool,
    pub path: String,
    pub message: String,
    pub already_existed: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GitStatusResult {
    pub path: String,
    pub is_clean: bool,
    pub has_uncommitted_changes: bool,
    pub status_output: String,
    pub changed_files: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GitCommitResult {
    pub success: bool,
    pub commit_hash: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GitListTrackedResult {
    pub files: Vec<String>,
    pub directories: Vec<String>,
}

// ─── Implementations ────────────────────────────────────────────────────────

async fn run_git(args: &[&str], cwd: &str) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .await
        .context("Failed to execute git")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args[0], stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn git_init(path: &str) -> Result<GitInitResult> {
    let git_dir = std::path::Path::new(path).join(".git");
    let already_existed = git_dir.exists();

    if !already_existed {
        run_git(&["init"], path).await?;
    }

    Ok(GitInitResult {
        success: true,
        path: path.to_string(),
        message: if already_existed {
            "Git repository already initialized".into()
        } else {
            "Initialized new Git repository".into()
        },
        already_existed,
    })
}

pub async fn git_status(path: &str) -> Result<GitStatusResult> {
    let output = run_git(&["status", "--porcelain"], path).await?;
    let changed_files: Vec<String> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l[3..].trim().to_string())
        .collect();

    Ok(GitStatusResult {
        path: path.to_string(),
        is_clean: output.is_empty(),
        has_uncommitted_changes: !output.is_empty(),
        status_output: output,
        changed_files,
    })
}

pub async fn git_add_commit(
    path: &str,
    files: &[String],
    message: &str,
) -> Result<GitCommitResult> {
    // Stage files
    let mut add_args = vec!["add"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    add_args.extend_from_slice(&file_refs);
    run_git(&add_args, path).await?;

    // Commit
    run_git(&["commit", "-m", message], path).await?;

    // Get commit hash
    let hash = run_git(&["rev-parse", "--short", "HEAD"], path).await.ok();

    Ok(GitCommitResult {
        success: true,
        commit_hash: hash,
        message: format!("Committed: {}", message),
    })
}

pub async fn git_list_tracked(path: &str) -> Result<GitListTrackedResult> {
    let output = run_git(&["ls-files", "--exclude-standard", "-co"], path).await?;

    let files: Vec<String> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    // Unique parent dirs
    let mut dirs: std::collections::HashSet<String> = std::collections::HashSet::new();
    for f in &files {
        if let Some(parent) = std::path::Path::new(f).parent() {
            let d = parent.to_string_lossy().to_string();
            if !d.is_empty() {
                dirs.insert(d);
            }
        }
    }

    let mut directories: Vec<String> = dirs.into_iter().collect();
    directories.sort();

    Ok(GitListTrackedResult { files, directories })
}
