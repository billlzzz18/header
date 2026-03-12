use anyhow::{bail, Context, Result};
use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ─── Parameter Schemas ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadSetupStateParams {
    /// Base directory of conductor folder (usually workspace root)
    pub workspace_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteSetupStateParams {
    /// Workspace root
    pub workspace_path: String,
    /// The step name, e.g. "2.1_product_guide"
    pub last_successful_step: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateTrackIdParams {
    /// Track description used to create a slug
    pub description: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateTrackMetadataParams {
    /// Workspace root
    pub workspace_path: String,
    /// Track ID (from generate_track_id)
    pub track_id: String,
    /// Track type: "feature" | "bug" | "chore"
    pub track_type: String,
    /// Track description
    pub description: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateConductorIndexParams {
    /// Workspace root
    pub workspace_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateTracksRegistryParams {
    /// Workspace root
    pub workspace_path: String,
    /// Track description
    pub track_description: String,
    /// Track ID
    pub track_id: String,
    /// Tracks directory name (e.g. "tracks")
    pub tracks_dir: String,
}

// ─── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, JsonSchema)]
pub struct SetupStateResult {
    pub exists: bool,
    pub last_successful_step: Option<String>,
    pub raw_content: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SimpleResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TrackIdResult {
    pub track_id: String,
    pub created_at: String,
}

// ─── Implementations ────────────────────────────────────────────────────────

pub async fn read_setup_state(workspace_path: &str) -> Result<SetupStateResult> {
    let state_path = format!("{}/conductor/setup_state.json", workspace_path);
    let p = std::path::Path::new(&state_path);

    if !p.exists() {
        return Ok(SetupStateResult {
            exists: false,
            last_successful_step: None,
            raw_content: None,
        });
    }

    let content = tokio::fs::read_to_string(p)
        .await
        .context("Cannot read setup_state.json")?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .context("setup_state.json is invalid JSON")?;

    let step = json
        .get("last_successful_step")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(SetupStateResult {
        exists: true,
        last_successful_step: step,
        raw_content: Some(content),
    })
}

pub async fn write_setup_state(workspace_path: &str, step: &str) -> Result<SimpleResult> {
    let state_path = format!("{}/conductor/setup_state.json", workspace_path);
    let content = serde_json::json!({ "last_successful_step": step }).to_string();

    tokio::fs::create_dir_all(format!("{}/conductor", workspace_path)).await?;
    tokio::fs::write(&state_path, &content).await?;

    Ok(SimpleResult {
        success: true,
        message: format!("State updated to '{}'", step),
    })
}

pub async fn generate_track_id(description: &str) -> Result<TrackIdResult> {
    // สร้าง slug จาก description
    let slug: String = description
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .take(4) // สูงสุด 4 คำ
        .collect::<Vec<_>>()
        .join("_");

    let date = Utc::now().format("%Y%m%d").to_string();
    let track_id = format!("{}_{}", slug, date);

    Ok(TrackIdResult {
        track_id,
        created_at: Utc::now().to_rfc3339(),
    })
}

pub async fn create_track_metadata(
    workspace_path: &str,
    track_id: &str,
    track_type: &str,
    description: &str,
    tracks_dir: &str,
) -> Result<SimpleResult> {
    let now = Utc::now().to_rfc3339();
    let metadata = serde_json::json!({
        "track_id": track_id,
        "type": track_type,
        "status": "new",
        "created_at": now,
        "updated_at": now,
        "description": description
    });

    let dir = format!("{}/conductor/{}/{}", workspace_path, tracks_dir, track_id);
    tokio::fs::create_dir_all(&dir).await?;

    let meta_path = format!("{}/metadata.json", dir);
    tokio::fs::write(&meta_path, serde_json::to_string_pretty(&metadata)?).await?;

    // สร้าง index.md ด้วย
    let index_content = format!(
        "# Track {} Context\n\n- [Specification](./spec.md)\n- [Implementation Plan](./plan.md)\n- [Metadata](./metadata.json)\n",
        track_id
    );
    tokio::fs::write(format!("{}/index.md", dir), &index_content).await?;

    Ok(SimpleResult {
        success: true,
        message: format!("Track metadata created at '{}'", dir),
    })
}

pub async fn create_conductor_index(workspace_path: &str) -> Result<SimpleResult> {
    let content = r#"# Project Context

## Definition
- [Product Definition](./product.md)
- [Product Guidelines](./product-guidelines.md)
- [Tech Stack](./tech-stack.md)

## Workflow
- [Workflow](./workflow.md)
- [Code Style Guides](./code_styleguides/)

## Management
- [Tracks Registry](./tracks.md)
- [Tracks Directory](./tracks/)
"#;

    let path = format!("{}/conductor/index.md", workspace_path);
    tokio::fs::write(&path, content).await?;

    Ok(SimpleResult {
        success: true,
        message: "Created conductor/index.md".into(),
    })
}

pub async fn create_tracks_registry(
    workspace_path: &str,
    track_description: &str,
    track_id: &str,
    tracks_dir: &str,
) -> Result<SimpleResult> {
    let content = format!(
        "# Project Tracks\n\nThis file tracks all major tracks for the project. Each track has its own detailed plan in its respective folder.\n\n---\n\n- [ ] **Track: {description}**\n  *Link: [./{dir}/{id}/](./{dir}/{id}/)*\n",
        description = track_description,
        dir = tracks_dir,
        id = track_id,
    );

    let path = format!("{}/conductor/tracks.md", workspace_path);
    tokio::fs::write(&path, &content).await?;

    Ok(SimpleResult {
        success: true,
        message: format!("Created conductor/tracks.md with track '{}'", track_id),
    })
}
