use std::path::PathBuf;

/// Root working directory (cwd ของ project ที่ conductor กำลัง setup)
#[derive(Clone, Debug)]
pub struct ServerState {
    pub workspace_root: PathBuf,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            workspace_root: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from(".")),
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
