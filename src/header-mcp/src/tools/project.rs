use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ─── Parameter Schemas ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DetectProjectMaturityParams {
    /// Directory to analyze (workspace root)
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScanProjectFilesParams {
    /// Workspace root to scan
    pub path: String,
    /// Max depth for directory walk (default 3)
    pub max_depth: Option<usize>,
    /// If true, respect .gitignore and .geminiignore patterns
    pub respect_ignore: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFileSizeParams {
    /// File path to check
    pub path: String,
}

// ─── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, JsonSchema)]
pub struct ProjectMaturityResult {
    /// "greenfield" or "brownfield"
    pub maturity: String,
    pub indicators: Vec<String>,
    pub has_git: bool,
    pub has_uncommitted_changes: bool,
    pub detected_manifests: Vec<String>,
    pub detected_source_dirs: Vec<String>,
    pub git_status_output: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ProjectScanResult {
    pub root: String,
    pub manifest_files: Vec<FileInfo>,
    pub source_dirs: Vec<String>,
    pub config_files: Vec<FileInfo>,
    pub total_files_scanned: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct FileInfo {
    pub path: String,
    pub size_bytes: u64,
    pub is_large: bool, // > 1MB
}

// ─── Implementations ────────────────────────────────────────────────────────

pub async fn detect_project_maturity(path: &str) -> Result<ProjectMaturityResult> {
    let root = Path::new(path);
    let mut indicators = Vec::new();
    let mut has_uncommitted_changes = false;
    let mut git_status_output = None;

    // ── ตรวจ version control ──
    let has_git = root.join(".git").exists();
    let has_svn = root.join(".svn").exists();
    let has_hg = root.join(".hg").exists();

    if has_git {
        indicators.push("Found .git directory".into());
        // ตรวจ uncommitted changes
        if let Ok(output) = tokio::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output()
            .await
        {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            has_uncommitted_changes = !s.is_empty();
            git_status_output = Some(s.clone());
            if !s.is_empty() {
                indicators.push("Dirty git repository (uncommitted changes)".into());
            }
        }
    }
    if has_svn { indicators.push("Found .svn directory".into()); }
    if has_hg  { indicators.push("Found .hg directory".into()); }

    // ── ตรวจ manifest files ──
    let manifest_names = [
        "package.json", "pom.xml", "requirements.txt", "go.mod",
        "Cargo.toml", "build.gradle", "setup.py", "pyproject.toml",
        "Gemfile", "composer.json",
    ];
    let mut detected_manifests = Vec::new();
    for name in &manifest_names {
        if root.join(name).exists() {
            detected_manifests.push(name.to_string());
            indicators.push(format!("Found manifest: {}", name));
        }
    }

    // ── ตรวจ source code directories ──
    let src_dirs = ["src", "app", "lib", "cmd", "pkg", "internal"];
    let mut detected_source_dirs = Vec::new();
    for dir in &src_dirs {
        let dir_path = root.join(dir);
        if dir_path.is_dir() {
            // ตรวจว่ามี code files จริงๆ ไหม
            let has_code = walkdir::WalkDir::new(&dir_path)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| {
                    let ext = e.path().extension()
                        .and_then(|x| x.to_str())
                        .unwrap_or("");
                    matches!(ext, "rs"|"ts"|"js"|"py"|"go"|"java"|"kt"|"swift"|"cs"|"rb")
                });

            if has_code {
                detected_source_dirs.push(dir.to_string());
                indicators.push(format!("Found source directory: {}/", dir));
            }
        }
    }

    // ── ตัดสิน maturity ──
    let is_brownfield = has_git || has_svn || has_hg
        || !detected_manifests.is_empty()
        || !detected_source_dirs.is_empty();

    // Greenfield: ไดเรกทอรีว่าง หรือมีแค่ README.md
    let maturity = if is_brownfield {
        "brownfield".to_string()
    } else {
        "greenfield".to_string()
    };

    Ok(ProjectMaturityResult {
        maturity,
        indicators,
        has_git,
        has_uncommitted_changes,
        detected_manifests,
        detected_source_dirs,
        git_status_output,
    })
}

pub async fn scan_project_files(
    path: &str,
    max_depth: usize,
    respect_ignore: bool,
) -> Result<ProjectScanResult> {
    let root = Path::new(path);
    let mut manifest_files = Vec::new();
    let mut config_files = Vec::new();
    let mut source_dirs = std::collections::HashSet::new();
    let mut total = 0usize;

    let manifest_names: std::collections::HashSet<&str> = [
        "package.json", "pom.xml", "requirements.txt", "go.mod",
        "Cargo.toml", "build.gradle", "setup.py", "pyproject.toml",
        "Gemfile", "composer.json",
    ].iter().copied().collect();

    let config_names: std::collections::HashSet<&str> = [
        ".env", ".env.example", "docker-compose.yml", "Dockerfile",
        ".gitignore", ".geminiignore", "README.md", "CHANGELOG.md",
    ].iter().copied().collect();

    // Directories to skip
    let skip_dirs: std::collections::HashSet<&str> = [
        "node_modules", ".m2", "build", "dist", "bin", "target",
        ".git", ".idea", ".vscode", "__pycache__", ".pytest_cache",
    ].iter().copied().collect();

    let walker = walkdir::WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !skip_dirs.contains(name.as_ref())
        });

    for entry in walker.filter_map(|e| e.ok()) {
        total += 1;
        let fname = entry.file_name().to_string_lossy().to_string();
        let fpath = entry.path().to_string_lossy().to_string();
        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let is_large = size > 1_000_000;

        if manifest_names.contains(fname.as_str()) {
            manifest_files.push(FileInfo { path: fpath.clone(), size_bytes: size, is_large });
        } else if config_names.contains(fname.as_str()) {
            config_files.push(FileInfo { path: fpath, size_bytes: size, is_large });
        }

        // Track source dirs
        if entry.file_type().is_file() {
            let ext = entry.path().extension()
                .and_then(|x| x.to_str()).unwrap_or("");
            if matches!(ext, "rs"|"ts"|"js"|"py"|"go"|"java"|"kt"|"swift"|"cs"|"rb") {
                if let Some(parent) = entry.path().parent() {
                    source_dirs.insert(parent.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(ProjectScanResult {
        root: path.to_string(),
        manifest_files,
        source_dirs: source_dirs.into_iter().collect(),
        config_files,
        total_files_scanned: total,
    })
}
