use anyhow::Result;
use rmcp::{protocol::ServerCapabilities, server::Server, transport::StdioTransport};
use tokio::signal;

mod handler;
mod state;
mod tools;

use handler::ConductorHandler;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Conductor MCP Server");

    let handler = ConductorHandler::new();
    let transport = StdioTransport::new();

    let server = Server::builder()
        .with_handler(handler)
        .with_capabilities(ServerCapabilities {
            tools: Some(Default::default()),
            ..Default::default()
        })
        .build(transport)?;

    tracing::info!("Server ready");
    server.run(signal::ctrl_c()).await?;

    Ok(())
}
