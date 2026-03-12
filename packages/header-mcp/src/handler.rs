use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_handler, tool_router, ServerHandler,
};
use serde_json::json;
use std::sync::Arc;

use crate::state::ServerState;
use crate::tools::{fs, git, header, project};

#[derive(Clone)]
pub struct HeaderHandler {
    state: Arc<ServerState>,
    tool_router: ToolRouter<HeaderHandler>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Router: ลงทะเบียน tool ทั้งหมด
// ─────────────────────────────────────────────────────────────────────────────

#[tool_router]
impl HeaderHandler {
    // ══════════════════════════════════════════════════════════════════════
    // FILESYSTEM TOOLS
    // ══════════════════════════════════════════════════════════════════════

    #[tool(
        name = "fs_file_exists",
        description = "Check whether a file or directory exists at the given path"
    )]
    async fn fs_file_exists(
        &self,
        params: Parameters<fs::FileExistsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = fs::file_exists(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "fs_read_file",
        description = "Read the contents of a file. Use head_lines or tail_lines to limit output for large files (>1MB)"
    )]
    async fn fs_read_file(
        &self,
        params: Parameters<fs::ReadFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = fs::read_file(&resolved.to_string_lossy(), p.head_lines, p.tail_lines)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "fs_write_file",
        description = "Write (or overwrite) content to a file. Parent directories are created automatically"
    )]
    async fn fs_write_file(
        &self,
        params: Parameters<fs::WriteFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = fs::write_file(&resolved.to_string_lossy(), &p.content)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "fs_append_file",
        description = "Append content to the end of a file (creates the file if it does not exist)"
    )]
    async fn fs_append_file(
        &self,
        params: Parameters<fs::AppendFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = fs::append_file(&resolved.to_string_lossy(), &p.content)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "fs_create_directory",
        description = "Create a directory and all parent directories (equivalent to mkdir -p)"
    )]
    async fn fs_create_directory(
        &self,
        params: Parameters<fs::CreateDirectoryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = fs::create_directory(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "fs_list_directory",
        description = "List files and directories at a given path. Set recursive=true for deep listing"
    )]
    async fn fs_list_directory(
        &self,
        params: Parameters<fs::ListDirectoryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = fs::list_directory(&resolved.to_string_lossy(), p.recursive.unwrap_or(false))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "fs_copy_file",
        description = "Copy a file from source to destination. Destination parent directories are created automatically"
    )]
    async fn fs_copy_file(
        &self,
        params: Parameters<fs::CopyFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let src = self.state.resolve(&p.source);
        let dst = self.state.resolve(&p.destination);
        let result = fs::copy_file(&src.to_string_lossy(), &dst.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "fs_delete",
        description = "Delete a file or directory (recursive for directories)"
    )]
    async fn fs_delete(
        &self,
        params: Parameters<fs::DeleteFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = fs::delete_file(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
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
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = git::git_init(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "git_status",
        description = "Get the git status of a repository (--porcelain format). Returns list of changed files and whether repo is clean"
    )]
    async fn git_status(
        &self,
        params: Parameters<git::GitStatusParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = git::git_status(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "git_add_commit",
        description = "Stage files and create a git commit. Use files=[\".\"] to stage all changes"
    )]
    async fn git_add_commit(
        &self,
        params: Parameters<git::GitAddCommitParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = git::git_add_commit(&resolved.to_string_lossy(), &p.files, &p.message)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "git_list_tracked",
        description = "List all files tracked by git (respects .gitignore). Useful for brownfield project analysis"
    )]
    async fn git_list_tracked(
        &self,
        params: Parameters<git::GitListTrackedParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = git::git_list_tracked(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    // ══════════════════════════════════════════════════════════════════════
    // header STATE TOOLS
    // ══════════════════════════════════════════════════════════════════════

    #[tool(
        name = "header_read_state",
        description = "Read the header/setup_state.json file to determine setup progress and which step to resume from"
    )]
    async fn header_read_state(
        &self,
        params: Parameters<header::ReadSetupStateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.workspace_path);
        let result = header::read_setup_state(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "header_write_state",
        description = "Write the last_successful_step to header/setup_state.json. Call this after each major step completes successfully"
    )]
    async fn header_write_state(
        &self,
        params: Parameters<header::WriteSetupStateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.workspace_path);
        let result =
            header::write_setup_state(&resolved.to_string_lossy(), &p.last_successful_step)
                .await
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "header_generate_track_id",
        description = "Generate a unique Track ID from a track description using format: shortname_YYYYMMDD"
    )]
    async fn header_generate_track_id(
        &self,
        params: Parameters<header::GenerateTrackIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let result = header::generate_track_id(&p.description)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "header_create_track_metadata",
        description = "Create metadata.json and index.md for a new track in the tracks directory"
    )]
    async fn header_create_track_metadata(
        &self,
        params: Parameters<header::CreateTrackMetadataParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.workspace_path);
        let result = header::create_track_metadata(
            &resolved.to_string_lossy(),
            &p.track_id,
            &p.track_type,
            &p.description,
            "tracks",
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "header_create_index",
        description = "Create header/index.md as the main project context index file"
    )]
    async fn header_create_index(
        &self,
        params: Parameters<header::CreateHeaderIndexParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.workspace_path);
        let result = header::create_header_index(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "header_create_tracks_registry",
        description = "Create or update header/tracks.md with the initial track entry"
    )]
    async fn header_create_tracks_registry(
        &self,
        params: Parameters<header::CreateTracksRegistryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.workspace_path);
        let result = header::create_tracks_registry(
            &resolved.to_string_lossy(),
            &p.track_description,
            &p.track_id,
            &p.tracks_dir,
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
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
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = project::detect_project_maturity(&resolved.to_string_lossy())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }

    #[tool(
        name = "project_scan_files",
        description = "Scan project directory for manifest files, source directories, and config files. Skips node_modules, build, target, etc."
    )]
    async fn project_scan_files(
        &self,
        params: Parameters<project::ScanProjectFilesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let resolved = self.state.resolve(&p.path);
        let result = project::scan_project_files(
            &resolved.to_string_lossy(),
            p.max_depth.unwrap_or(3),
            p.respect_ignore.unwrap_or(true),
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        respond_json(json!(result))
    }
}

fn respond_json(value: serde_json::Value) -> Result<CallToolResult, ErrorData> {
    let content =
        Content::json(value).map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
    Ok(CallToolResult::success(vec![content]))
}

// ─────────────────────────────────────────────────────────────────────────────
// Server Handler (delegate tool_call to tool_router)
// ─────────────────────────────────────────────────────────────────────────────

#[tool_handler]
impl ServerHandler for HeaderHandler {}

impl HeaderHandler {
    pub fn new(state: Arc<ServerState>) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}
