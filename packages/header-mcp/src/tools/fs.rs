use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ─── Parameter Schemas ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileExistsParams {
    /// Path to check (relative to workspace root)
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadFileParams {
    /// Path of file to read (relative to workspace root)
    pub path: String,
    /// Optional: read only first N lines (for large files)
    pub head_lines: Option<usize>,
    /// Optional: read only last N lines (for large files)
    pub tail_lines: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteFileParams {
    /// Path of file to write (relative to workspace root)
    pub path: String,
    /// Content to write (overwrites existing content)
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AppendFileParams {
    /// Path of file to append to (relative to workspace root)
    pub path: String,
    /// Content to append
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateDirectoryParams {
    /// Directory path to create, including all parents (mkdir -p)
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDirectoryParams {
    /// Directory to list
    pub path: String,
    /// If true, list recursively (default false)
    pub recursive: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CopyFileParams {
    /// Source file path
    pub source: String,
    /// Destination file path (will create parent dirs if needed)
    pub destination: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteFileParams {
    /// File path to delete
    pub path: String,
}

// ─── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, JsonSchema)]
pub struct FileExistsResult {
    pub exists: bool,
    pub path: String,
    pub is_file: bool,
    pub is_directory: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ReadFileResult {
    pub path: String,
    pub content: String,
    pub size_bytes: u64,
    pub line_count: usize,
    pub truncated: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct WriteFileResult {
    pub path: String,
    pub bytes_written: usize,
    pub created_new: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListDirectoryResult {
    pub path: String,
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub entry_type: String, // "file" | "directory"
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SimpleResult {
    pub success: bool,
    pub message: String,
}

// ─── Implementations ────────────────────────────────────────────────────────

pub async fn file_exists(path: &str) -> Result<FileExistsResult> {
    let p = Path::new(path);
    match tokio::fs::metadata(p).await {
        Ok(meta) => Ok(FileExistsResult {
            exists: true,
            path: path.to_string(),
            is_file: meta.is_file(),
            is_directory: meta.is_dir(),
        }),
        Err(_) => Ok(FileExistsResult {
            exists: false,
            path: path.to_string(),
            is_file: false,
            is_directory: false,
        }),
    }
}

pub async fn read_file(
    path: &str,
    head_lines: Option<usize>,
    tail_lines: Option<usize>,
) -> Result<ReadFileResult> {
    let p = Path::new(path);
    let metadata = tokio::fs::metadata(p)
        .await
        .with_context(|| format!("Cannot stat '{}'", path))?;

    let raw = tokio::fs::read_to_string(p)
        .await
        .with_context(|| format!("Cannot read '{}'", path))?;

    let all_lines: Vec<&str> = raw.lines().collect();
    let total_lines = all_lines.len();
    let mut truncated = false;

    let content = if let Some(n) = head_lines {
        truncated = n < total_lines;
        all_lines[..n.min(total_lines)].join("\n")
    } else if let Some(n) = tail_lines {
        truncated = n < total_lines;
        let start = total_lines.saturating_sub(n);
        all_lines[start..].join("\n")
    } else {
        raw
    };

    Ok(ReadFileResult {
        path: path.to_string(),
        content,
        size_bytes: metadata.len(),
        line_count: total_lines,
        truncated,
    })
}

pub async fn write_file(path: &str, content: &str) -> Result<WriteFileResult> {
    let p = Path::new(path);
    let created_new = !p.exists();

    // สร้าง parent directory อัตโนมัติ
    if let Some(parent) = p.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Cannot create directories for '{}'", path))?;
    }

    tokio::fs::write(p, content)
        .await
        .with_context(|| format!("Cannot write '{}'", path))?;

    Ok(WriteFileResult {
        path: path.to_string(),
        bytes_written: content.len(),
        created_new,
    })
}

pub async fn append_file(path: &str, content: &str) -> Result<SimpleResult> {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(p)
        .await
        .with_context(|| format!("Cannot open '{}' for append", path))?;

    file.write_all(content.as_bytes()).await?;

    Ok(SimpleResult {
        success: true,
        message: format!("Appended {} bytes to '{}'", content.len(), path),
    })
}

pub async fn create_directory(path: &str) -> Result<SimpleResult> {
    tokio::fs::create_dir_all(path)
        .await
        .with_context(|| format!("Cannot create directory '{}'", path))?;

    Ok(SimpleResult {
        success: true,
        message: format!("Directory '{}' created (including parents)", path),
    })
}

pub async fn list_directory(path: &str, recursive: bool) -> Result<ListDirectoryResult> {
    let p = Path::new(path);
    let mut entries = Vec::new();

    if recursive {
        for entry in walkdir::WalkDir::new(p)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let meta = entry.metadata().ok();
            entries.push(DirectoryEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                entry_type: if entry.file_type().is_dir() {
                    "directory".into()
                } else {
                    "file".into()
                },
                size_bytes: meta.as_ref().map(|m| m.len()),
            });
        }
    } else {
        let mut rd = tokio::fs::read_dir(p)
            .await
            .with_context(|| format!("Cannot list '{}'", path))?;

        while let Some(entry) = rd.next_entry().await? {
            let meta = entry.metadata().await.ok();
            let ft = meta
                .as_ref()
                .map(|m| if m.is_dir() { "directory" } else { "file" })
                .unwrap_or("unknown");

            entries.push(DirectoryEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                entry_type: ft.into(),
                size_bytes: meta.as_ref().map(|m| m.len()),
            });
        }
    }

    Ok(ListDirectoryResult {
        path: path.to_string(),
        entries,
    })
}

pub async fn copy_file(source: &str, destination: &str) -> Result<SimpleResult> {
    let dest = Path::new(destination);
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::copy(source, destination)
        .await
        .with_context(|| format!("Cannot copy '{}' -> '{}'", source, destination))?;

    Ok(SimpleResult {
        success: true,
        message: format!("Copied '{}' to '{}'", source, destination),
    })
}

pub async fn delete_file(path: &str) -> Result<SimpleResult> {
    let p = Path::new(path);
    if p.is_dir() {
        tokio::fs::remove_dir_all(p).await?;
    } else {
        tokio::fs::remove_file(p).await?;
    }
    Ok(SimpleResult {
        success: true,
        message: format!("Deleted '{}'", path),
    })
}
