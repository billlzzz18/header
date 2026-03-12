use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use std::path::Path;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tracing::{debug, error, info};

use super::events::{ClientMessage, ServerMessage};
use crate::state::WatchFeatureState;
use crate::utils::normalize_path;
use crate::watch::watcher::spawn_watcher;

pub async fn handle_watch_connection(
    mut ws: WebSocketStream<TcpStream>,
    state: WatchFeatureState,
) -> Result<()> {
    // ─── Step 1: Receive join message ───────────────────────────────────────
    let join_msg = match ws.next().await {
        Some(Ok(msg)) => msg,
        Some(Err(e)) => {
            error!("WebSocket error on connect: {}", e);
            return Err(anyhow::anyhow!("Connection error: {}", e));
        }
        None => {
            return Err(anyhow::anyhow!("Connection closed before join"));
        }
    };

    let join_text = match join_msg.to_text() {
        Ok(text) => text,
        Err(_) => {
            let _ = ws
                .send(tokio_tungstenite::tungstenite::Message::Text(
                    ServerMessage::Error {
                        message: "Expected text message".to_string(),
                    }
                    .to_json_string(),
                ))
                .await;
            return Err(anyhow::anyhow!("Expected text message"));
        }
    };

    let client_msg: ClientMessage = match serde_json::from_str(join_text) {
        Ok(msg) => msg,
        Err(e) => {
            let _ = ws
                .send(tokio_tungstenite::tungstenite::Message::Text(
                    ServerMessage::Error {
                        message: format!("Invalid JSON: {}", e),
                    }
                    .to_json_string(),
                ))
                .await;
            return Err(anyhow::anyhow!("Invalid JSON: {}", e));
        }
    };

    // ─── Step 2: Extract and validate path ──────────────────────────────────
    let ClientMessage::Join { path, is_wsl } = client_msg;

    let normalized_path = normalize_path(&path, is_wsl)
        .await
        .context("Failed to normalize path")?;

    if !Path::new(&normalized_path).is_absolute() {
        let _ = ws
            .send(tokio_tungstenite::tungstenite::Message::Text(
                ServerMessage::Error {
                    message: "Path must be absolute".to_string(),
                }
                .to_json_string(),
            ))
            .await;
        return Err(anyhow::anyhow!("Path must be absolute"));
    }

    let path_obj = Path::new(&normalized_path);
    if !path_obj.exists() {
        let _ = ws
            .send(tokio_tungstenite::tungstenite::Message::Text(
                ServerMessage::Error {
                    message: "Path does not exist".to_string(),
                }
                .to_json_string(),
            ))
            .await;
        return Err(anyhow::anyhow!("Path does not exist"));
    }

    let canonical_path = path_obj
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_string_lossy()
        .to_string();

    info!("Client joined watch: {}", canonical_path);

    // ─── Step 3: Get or create watcher ─────────────────────────────────────
    let active_watch = state.get_or_create_watcher(canonical_path.clone());
    let mut broadcast_rx = active_watch.tx.subscribe();

    let should_start = active_watch
        .started
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_ok();

    if should_start {
        spawn_watcher(state.clone(), canonical_path.clone(), is_wsl);
    }

    // ─── Step 4: Send ok reply ────────────────────────────────────────────
    let ok_msg = ServerMessage::Ok {
        path: canonical_path.clone(),
    };
    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        ok_msg.to_json_string(),
    ))
    .await?;

    debug!("Sent ok reply to client");

    // ─── Step 5: Forward events from broadcast to WebSocket ────────────────
    loop {
        tokio::select! {
            result = broadcast_rx.recv() => {
                match result {
                    Ok(msg) => {
                        let is_terminated = matches!(&msg, ServerMessage::Terminated { .. });

                        let ws_msg = tokio_tungstenite::tungstenite::Message::Text(msg.to_json_string());
                        if let Err(e) = ws.send(ws_msg).await {
                            error!("Failed to send message to client: {}", e);
                            return Err(anyhow::anyhow!("Send error: {}", e));
                        }

                        if is_terminated {
                            return Ok(());
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        debug!("Broadcast channel closed");
                        return Ok(());
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                }
            }

            msg = ws.next() => {
                if msg.is_none() {
                    debug!("WebSocket connection closed by client");
                    return Ok(());
                }
            }
        }
    }
}
