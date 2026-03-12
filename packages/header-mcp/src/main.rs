use anyhow::Result;
use rmcp::{serve_server, transport::stdio};
use rocket::routes;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tracing::info;

mod handler;
mod state;
mod tools;
mod utils;
mod watch;

use handler::HeaderHandler;
use state::GlobalState;
use watch::handle_watch_connection;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    info!("Starting header Unified Server");

    let global_state = GlobalState::new();
    let watch_state = global_state.watch.clone();
    let mcp_state = global_state.mcp.clone();

    let rocket_handle = {
        let config = rocket::Config {
            address: "127.0.0.1".parse().unwrap(),
            port: 8000,
            ..rocket::Config::default()
        };

        let rocket = rocket::custom(config).mount("/", routes![http_index, http_health]);

        tokio::spawn(async move {
            let _ = rocket.launch().await;
        })
    };

    info!("HTTP server listening on http://127.0.0.1:8000");

    let ws_listener = TcpListener::bind("127.0.0.1:8001").await?;
    info!("WebSocket server listening on ws://127.0.0.1:8001");

    let ws_handle = {
        let watch_state = watch_state.clone();
        tokio::spawn(async move {
            loop {
                match ws_listener.accept().await {
                    Ok((stream, addr)) => {
                        let watch_state = watch_state.clone();
                        tokio::spawn(async move {
                            match accept_async(stream).await {
                                Ok(ws) => {
                                    info!("WebSocket client connected: {}", addr);
                                    if let Err(e) =
                                        handle_watch_connection(ws, (*watch_state).clone()).await
                                    {
                                        tracing::error!("Watch connection error: {}", e);
                                    }
                                    info!("WebSocket client disconnected: {}", addr);
                                }
                                Err(e) => {
                                    tracing::error!("WebSocket upgrade error: {}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Accept error: {}", e);
                    }
                }
            }
        })
    };

    let mcp_handle = {
        let handler = HeaderHandler::new(mcp_state);
        let transport = stdio();

        let server = serve_server(handler, transport).await?;
        info!("MCP server ready (stdio)");
        server
    };

    info!("All servers running. Press Ctrl+C to stop.");

    tokio::signal::ctrl_c().await?;
    info!("Stop signal received; shutting down server");

    let quit_reason = mcp_handle.cancel().await?;
    info!(?quit_reason, "MCP server shutdown complete");

    let _ = rocket_handle;
    let _ = ws_handle;

    Ok(())
}

#[rocket::get("/")]
fn http_index() -> &'static str {
    r#"
header Unified Server

Services:
- HTTP: http://127.0.0.1:8000
- WebSocket: ws://127.0.0.1:8001
- MCP: stdio (for LLM integration)

Endpoints:
- GET / - This page
- GET /health - Health check
- WS /watch/<ref> - File watch connection

Example WebSocket:
  ws://127.0.0.1:8001/watch/header_main
  Send: {\"type\":\"join\",\"path\":\"/home/user/project\",\"is_wsl\":false}
"#
}

#[rocket::get("/health")]
fn http_health() -> &'static str {
    "OK"
}
