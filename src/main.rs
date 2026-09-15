//! # Tandoor MCP Server
//!
//! A Model Context Protocol (MCP) server that provides tools for interacting with Tandoor,
//! a recipe management system. This server allows AI assistants to search recipes, create
//! new recipes, manage shopping lists, and more through a standardized protocol.
//!
//! ## Environment Variables
//!
//! - `TANDOOR_BASE_URL`: Tandoor server URL (default: http://localhost:8080)
//! - `TANDOOR_AUTH_TOKEN`: Pre-existing Tandoor API token (preferred; avoids rate-limited login)
//! - `TANDOOR_USERNAME`: Tandoor username (used only when TANDOOR_AUTH_TOKEN is not set)
//! - `TANDOOR_PASSWORD`: Tandoor password (used only when TANDOOR_AUTH_TOKEN is not set)
//! - `BIND_ADDR`: Address to bind the MCP server (default: 0.0.0.0:3001)
//! - `RUST_LOG`: Logging level (info, debug, trace, etc.)

use mcp_tandoor::server::TandoorMcpServer;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::LocalSessionManager,
};
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let base_url =
        env::var("TANDOOR_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".to_string());

    // Authenticate: prefer TANDOOR_AUTH_TOKEN to avoid the 10-req/day rate limit on
    // the /api-token-auth/ login endpoint.
    tracing::info!("Validating Tandoor credentials...");
    let startup_server = if let Ok(token) = env::var("TANDOOR_AUTH_TOKEN") {
        tracing::info!("Using pre-existing auth token from TANDOOR_AUTH_TOKEN");
        let server = TandoorMcpServer::new(base_url.clone());
        server.set_global_auth_token(token).await?;
        server
    } else {
        let username = env::var("TANDOOR_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let password = env::var("TANDOOR_PASSWORD").unwrap_or_else(|_| "admin".to_string());
        let server = TandoorMcpServer::new_with_credentials(
            base_url.clone(),
            username.clone(),
            password.clone(),
        );
        if let Err(e) = server.authenticate(username.clone(), password.clone()).await {
            tracing::error!("Authentication failed: {}", e);
            tracing::error!("  - TANDOOR_BASE_URL: {}", base_url);
            tracing::error!("  - TANDOOR_USERNAME: {}", username);
            tracing::error!("  - Set TANDOOR_AUTH_TOKEN to bypass login rate limiting");
            std::process::exit(1);
        }
        server
    };

    tracing::info!("Testing API access with token...");
    match startup_server.test_api_access().await {
        Ok(_) => tracing::info!("API access test passed"),
        Err(e) => {
            tracing::warn!("API access test failed: {e}");
            tracing::warn!("The server will continue, but some tools may not work.");
        }
    }

    tracing::info!("Successfully authenticated with Tandoor");

    let base_url_clone = base_url.clone();

    let service = StreamableHttpService::new(
        move || Ok(TandoorMcpServer::new(base_url_clone.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Tandoor MCP Server listening on {bind_addr} (Streamable HTTP at /mcp)");

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.unwrap();
            tracing::info!("Shutting down...");
        })
        .await?;

    Ok(())
}
