use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::broadcast;

// ─── Watch State ───────────────────────────────────────────────────────────

pub struct ActiveWatch {
    pub tx: broadcast::Sender<crate::watch::events::ServerMessage>,
    pub started: AtomicBool,
}

#[derive(Clone, Default)]
pub struct WatchFeatureState {
    pub watchers: Arc<DashMap<String, Arc<ActiveWatch>>>,
}

impl WatchFeatureState {
    pub fn new() -> Self {
        Self {
            watchers: Arc::new(DashMap::new()),
        }
    }

    pub fn get_or_create_watcher(&self, canonical_path: String) -> Arc<ActiveWatch> {
        self.watchers
            .entry(canonical_path)
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(256);
                Arc::new(ActiveWatch {
                    tx,
                    started: AtomicBool::new(false),
                })
            })
            .clone()
    }

    pub fn remove_watcher(&self, canonical_path: &str) {
        self.watchers.remove(canonical_path);
    }
}

// ─── MCP Server State ──────────────────────────────────────────────────────

/// Root working directory (cwd ของ project ที่ header กำลัง setup)
#[derive(Clone, Debug)]
pub struct ServerState {
    pub workspace_root: PathBuf,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Resolve a relative path from workspace root
    pub fn resolve(&self, path: &str) -> PathBuf {
        if std::path::Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.workspace_root.join(path)
        }
    }
}

// ─── Unified Global State ─────────────────────────────────────────────────

pub struct GlobalState {
    pub watch: Arc<WatchFeatureState>,
    pub mcp: Arc<ServerState>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            watch: Arc::new(WatchFeatureState::new()),
            mcp: Arc::new(ServerState::new()),
        }
    }
}
