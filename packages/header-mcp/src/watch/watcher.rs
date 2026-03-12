use notify::{EventKind, PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, warn};

use super::events::ServerMessage;
use crate::state::WatchFeatureState;

pub fn to_relative_path(absolute_path: &Path, watched_path: &str) -> Option<String> {
    let watched = Path::new(watched_path);
    absolute_path
        .strip_prefix(watched)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

pub fn convert_notify_event(
    event: notify::Event,
    watched_path: &str,
    pending_rename_from: &mut Option<String>,
) -> Vec<ServerMessage> {
    let mut results = Vec::new();

    match event.kind {
        EventKind::Create(_) => {
            for path in event.paths {
                if let Some(relative_path) = to_relative_path(&path, watched_path) {
                    results.push(ServerMessage::Created {
                        path: relative_path,
                    });
                }
            }
        }

        EventKind::Modify(modify_kind) => match modify_kind {
            notify::event::ModifyKind::Name(rename_mode) => match rename_mode {
                notify::event::RenameMode::From => {
                    if let Some(path) = event.paths.first() {
                        *pending_rename_from = to_relative_path(path, watched_path);
                    }
                }
                notify::event::RenameMode::To => {
                    if let Some(to_path) = event.paths.first() {
                        if let Some(to_relative) = to_relative_path(to_path, watched_path) {
                            if let Some(from_relative) = pending_rename_from.take() {
                                results.push(ServerMessage::Renamed {
                                    from: from_relative,
                                    to: to_relative,
                                });
                            } else {
                                results.push(ServerMessage::Created { path: to_relative });
                            }
                        } else {
                            pending_rename_from.take();
                        }
                    }
                }
                notify::event::RenameMode::Both => {
                    if event.paths.len() >= 2 {
                        let from_relative = to_relative_path(&event.paths[0], watched_path);
                        let to_relative = to_relative_path(&event.paths[1], watched_path);
                        match (from_relative, to_relative) {
                            (Some(from), Some(to)) => {
                                results.push(ServerMessage::Renamed { from, to });
                            }
                            (Some(from), None) => {
                                results.push(ServerMessage::Deleted { path: from });
                            }
                            (None, Some(to)) => {
                                results.push(ServerMessage::Created { path: to });
                            }
                            (None, None) => {}
                        }
                    }
                }
                _ => {
                    for path in event.paths {
                        if let Some(relative_path) = to_relative_path(&path, watched_path) {
                            results.push(ServerMessage::Modified {
                                path: relative_path,
                            });
                        }
                    }
                }
            },
            _ => {
                for path in event.paths {
                    if let Some(relative_path) = to_relative_path(&path, watched_path) {
                        results.push(ServerMessage::Modified {
                            path: relative_path,
                        });
                    }
                }
            }
        },

        EventKind::Remove(_) => {
            for path in &event.paths {
                let path_str = path.to_string_lossy().to_string();

                if path_str == watched_path {
                    results.push(ServerMessage::Terminated {
                        reason: "watched path was removed".to_string(),
                    });
                } else if let Some(relative_path) = to_relative_path(path, watched_path) {
                    results.push(ServerMessage::Deleted {
                        path: relative_path,
                    });
                }
            }
        }

        _ => {}
    }

    results
}

pub fn spawn_watcher(state: WatchFeatureState, canonical_path: String, is_wsl: bool) {
    let active_watch = state.get_or_create_watcher(canonical_path.clone());
    let tx = active_watch.tx.clone();

    #[cfg(target_os = "windows")]
    let use_poll_watcher = is_wsl;
    #[cfg(not(target_os = "windows"))]
    let use_poll_watcher = {
        let _ = is_wsl;
        false
    };

    tokio::spawn(async move {
        let (notify_tx, mut notify_rx) =
            tokio::sync::mpsc::channel::<notify::Result<notify::Event>>(256);

        let create_poll_watcher = |notify_tx: tokio::sync::mpsc::Sender<
            notify::Result<notify::Event>,
        >| {
            let poll_config = notify::Config::default().with_poll_interval(Duration::from_secs(2));
            PollWatcher::new(
                move |res| {
                    let _ = notify_tx.blocking_send(res);
                },
                poll_config,
            )
        };

        let mut watcher: Box<dyn Watcher + Send> = if use_poll_watcher {
            match create_poll_watcher(notify_tx.clone()) {
                Ok(w) => Box::new(w),
                Err(e) => {
                    let _ = tx.send(ServerMessage::Terminated {
                        reason: format!("Failed to create poll watcher: {}", e),
                    });
                    state.remove_watcher(&canonical_path);
                    return;
                }
            }
        } else {
            let notify_tx_clone = notify_tx.clone();
            let watcher_result = RecommendedWatcher::new(
                move |res| {
                    let _ = notify_tx_clone.blocking_send(res);
                },
                notify::Config::default(),
            );

            match watcher_result {
                Ok(w) => Box::new(w),
                Err(e) => {
                    warn!(
                        "Native file watcher failed for {}: {}. Falling back to poll watcher.",
                        canonical_path, e
                    );

                    match create_poll_watcher(notify_tx.clone()) {
                        Ok(w) => {
                            let _ = tx.send(ServerMessage::Warning {
                                message: format!(
                                    "Using poll-based file watching (native watcher unavailable: {}). File change detection may be slower.",
                                    e
                                ),
                            });
                            Box::new(w)
                        }
                        Err(poll_err) => {
                            let _ = tx.send(ServerMessage::Terminated {
                                reason: format!(
                                    "Failed to create watcher: {} (poll fallback also failed: {})",
                                    e, poll_err
                                ),
                            });
                            state.remove_watcher(&canonical_path);
                            return;
                        }
                    }
                }
            }
        };

        if let Err(e) = watcher.watch(Path::new(&canonical_path), RecursiveMode::Recursive) {
            let _ = tx.send(ServerMessage::Terminated {
                reason: format!("Failed to watch path: {}", e),
            });
            state.remove_watcher(&canonical_path);
            return;
        }

        let mut pending_rename_from: Option<String> = None;
        let cleanup_check_interval = Duration::from_secs(30);

        loop {
            tokio::select! {
                res = notify_rx.recv() => {
                    match res {
                        Some(Ok(event)) => {
                            let messages = convert_notify_event(
                                event,
                                &canonical_path,
                                &mut pending_rename_from,
                            );

                            for msg in messages {
                                let is_terminated = matches!(&msg, ServerMessage::Terminated { .. });
                                let _ = tx.send(msg);

                                if is_terminated {
                                    state.remove_watcher(&canonical_path);
                                    return;
                                }
                            }
                        }
                        Some(Err(e)) => {
                            let _ = tx.send(ServerMessage::Terminated {
                                reason: format!("Watch error: {}", e),
                            });
                            state.remove_watcher(&canonical_path);
                            return;
                        }
                        None => {
                            state.remove_watcher(&canonical_path);
                            return;
                        }
                    }
                }
                _ = tokio::time::sleep(cleanup_check_interval) => {
                    if tx.receiver_count() == 0 {
                        debug!("No subscribers remaining for watch on {}, cleaning up", canonical_path);
                        state.remove_watcher(&canonical_path);
                        return;
                    }
                }
            }
        }
    });
}
