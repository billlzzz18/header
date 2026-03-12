use anyhow::Result;

pub async fn normalize_path(path: &str, is_wsl: bool) -> Result<String> {
    #[cfg(target_os = "windows")]
    if is_wsl {
        return Ok(path.to_string());
    }

    Ok(path.to_string())
}
