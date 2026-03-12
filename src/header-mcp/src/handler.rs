use async_trait::async_trait;
use rmcp::{
    model::*,
    protocol::*,
    server::{RequestContext, RoleServer, ServerHandler, ToolRouter},
    tool_handler, tool_router, ErrorData,
};
use serde_json::json;
use std::sync::Arc;

use crate::state::ServerState;
use crate::tools::{conductor, filesystem, git, project};

pub struct ConductorHandler {
    state: Arc<ServerState>,
    tool_router: ToolRouter,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Router: ลงทะเบียน tool ทั้งหมด
// ─────────────────────────────────────────────────────────────────────────────

#[tool_router]
impl ConductorHandler {

    // ══════════════════════════════════════════════════════════════════════
    // FILESYSTEM TOOLS
    // ══════════════════════════════════════════════════════════════════════

    #[tool(
        name = "fs_file_exists",
        description = "Check whether a file or directory exists at the given path"
    )]
    async fn fs_file_exists(
        &self,
        params: Parameters<filesystem::FileExistsParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = filesystem::file_exists(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "fs_read_file",
        description = "Read the contents of a file. Use head_lines or tail_lines to limit output for large files (>1MB)"
    )]
    async fn fs_read_file(
        &self,
        params: Parameters<filesystem::ReadFileParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = filesystem::read_file(
            &resolved.to_string_lossy(),
            p.head_lines,
            p.tail_lines,
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "fs_write_file",
        description = "Write (or overwrite) content to a file. Parent directories are created automatically"
    )]
    async fn fs_write_file(
        &self,
        params: Parameters<filesystem::WriteFileParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = filesystem::write_file(&resolved.to_string_lossy(), &p.content)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "fs_append_file",
        description = "Append content to the end of a file (creates the file if it does not exist)"
    )]
    async fn fs_append_file(
        &self,
        params: Parameters<filesystem::AppendFileParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = filesystem::append_file(&resolved.to_string_lossy(), &p.content)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "fs_create_directory",
        description = "Create a directory and all parent directories (equivalent to mkdir -p)"
    )]
    async fn fs_create_directory(
        &self,
        params: Parameters<filesystem::CreateDirectoryParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = filesystem::create_directory(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "fs_list_directory",
        description = "List files and directories at a given path. Set recursive=true for deep listing"
    )]
    async fn fs_list_directory(
        &self,
        params: Parameters<filesystem::ListDirectoryParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = filesystem::list_directory(
            &resolved.to_string_lossy(),
            p.recursive.unwrap_or(false),
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "fs_copy_file",
        description = "Copy a file from source to destination. Destination parent directories are created automatically"
    )]
    async fn fs_copy_file(
        &self,
        params: Parameters<filesystem::CopyFileParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let src = self.state.resolve(&p.source);
        let dst = self.state.resolve(&p.destination);
        let result = filesystem::copy_file(
            &src.to_string_lossy(),
            &dst.to_string_lossy(),
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "fs_delete",
        description = "Delete a file or directory (recursive for directories)"
    )]
    async fn fs_delete(
        &self,
        params: Parameters<filesystem::DeleteFileParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = filesystem::delete_file(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    // ══════════════════════════════════════════════════════════════════════
    // GIT TOOLS
    // ══════════════════════════════════════════════════════════════════════

    #[tool(
        name = "git_init",
        description = "Initialize a new git repository in the given directory (safe to call if already initialized)"
    )]
    async fn git_init(
        &self,
        params: Parameters<git::GitInitParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = git::git_init(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "git_status",
        description = "Get the git status of a repository (--porcelain format). Returns list of changed files and whether repo is clean"
    )]
    async fn git_status(
        &self,
        params: Parameters<git::GitStatusParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = git::git_status(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "git_add_commit",
        description = "Stage files and create a git commit. Use files=[\".\"] to stage all changes"
    )]
    async fn git_add_commit(
        &self,
        params: Parameters<git::GitAddCommitParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = git::git_add_commit(
            &resolved.to_string_lossy(),
            &p.files,
            &p.message,
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "git_list_tracked",
        description = "List all files tracked by git (respects .gitignore). Useful for brownfield project analysis"
    )]
    async fn git_list_tracked(
        &self,
        params: Parameters<git::GitListTrackedParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = git::git_list_tracked(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    // ══════════════════════════════════════════════════════════════════════
    // CONDUCTOR STATE TOOLS
    // ══════════════════════════════════════════════════════════════════════

    #[tool(
        name = "conductor_read_state",
        description = "Read the conductor/setup_state.json file to determine setup progress and which step to resume from"
    )]
    async fn conductor_read_state(
        &self,
        params: Parameters<conductor::ReadSetupStateParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.workspace_path);
        let result = conductor::read_setup_state(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "conductor_write_state",
        description = "Write the last_successful_step to conductor/setup_state.json. Call this after each major step completes successfully"
    )]
    async fn conductor_write_state(
        &self,
        params: Parameters<conductor::WriteSetupStateParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.workspace_path);
        let result = conductor::write_setup_state(
            &resolved.to_string_lossy(),
            &p.last_successful_step,
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "conductor_generate_track_id",
        description = "Generate a unique Track ID from a track description using format: shortname_YYYYMMDD"
    )]
    async fn conductor_generate_track_id(
        &self,
        params: Parameters<conductor::GenerateTrackIdParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let result = conductor::generate_track_id(&p.description)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "conductor_create_track_metadata",
        description = "Create metadata.json and index.md for a new track in the tracks directory"
    )]
    async fn conductor_create_track_metadata(
        &self,
        params: Parameters<conductor::CreateTrackMetadataParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.workspace_path);
        let result = conductor::create_track_metadata(
            &resolved.to_string_lossy(),
            &p.track_id,
            &p.track_type,
            &p.description,
            "tracks",
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "conductor_create_index",
        description = "Create conductor/index.md as the main project context index file"
    )]
    async fn conductor_create_index(
        &self,
        params: Parameters<conductor::CreateConductorIndexParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.workspace_path);
        let result = conductor::create_conductor_index(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "conductor_create_tracks_registry",
        description = "Create or update conductor/tracks.md with the initial track entry"
    )]
    async fn conductor_create_tracks_registry(
        &self,
        params: Parameters<conductor::CreateTracksRegistryParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.workspace_path);
        let result = conductor::create_tracks_registry(
            &resolved.to_string_lossy(),
            &p.track_description,
            &p.track_id,
            &p.tracks_dir,
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    // ══════════════════════════════════════════════════════════════════════
    // PROJECT ANALYSIS TOOLS
    // ══════════════════════════════════════════════════════════════════════

    #[tool(
        name = "project_detect_maturity",
        description = "Detect whether a project is Greenfield (new) or Brownfield (existing) by checking for git, manifests, and source directories"
    )]
    async fn project_detect_maturity(
        &self,
        params: Parameters<project::DetectProjectMaturityParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = project::detect_project_maturity(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }

    #[tool(
        name = "project_scan_files",
        description = "Scan project directory for manifest files, source directories, and config files. Skips node_modules, build, target, etc."
    )]
    async fn project_scan_files(
        &self,
        params: Parameters<project::ScanProjectFilesParams>,
    ) -> Result<serde_json::Value, ErrorData> {
        let p = params.inner();
        let resolved = self.state.resolve(&p.path);
        let result = project::scan_project_files(
            &resolved.to_string_lossy(),
            p.max_depth.unwrap_or(3),
            p.respect_ignore.unwrap_or(true),
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(json!(result))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Server Handler (delegate tool_call to tool_router)
// ─────────────────────────────────────────────────────────────────────────────

#[tool_handler]
impl ServerHandler for ConductorHandler {}

impl ConductorHandler {
    pub fn new() -> Self {
        Self {
            state: Arc::new(ServerState::new()),
            tool_router: Self::tool_router(),
        }
    }
}
