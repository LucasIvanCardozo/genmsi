//! The GenMSI MCP server: the engine exposed to AI agents over stdio.
//!
//! Layout:
//! - [`server`]: the rmcp handler and the tools it serves.
//! - [`engine`]: the only place that calls the engine crates.
//! - [`errors`]: the failure a tool returns when a model does not work.
//!
//! The crate carries no simulation logic (decision D2): every tool is a thin
//! adapter over the same API the CLI uses, and the documents the tools return
//! are the ones the CLI publishes (decision D13).

pub mod engine;
pub mod errors;
pub mod glossary;
pub mod server;

use rmcp::ServiceExt;

pub use server::GenmsiServer;

/// Serve the MCP protocol over stdio until the client disconnects.
///
/// Standard output belongs to the protocol: nothing else may be written there.
pub async fn serve_stdio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = GenmsiServer::new().serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// Serve the MCP protocol over stdio from synchronous code, blocking until the
/// client disconnects.
///
/// The CLI is synchronous and must not acquire a runtime dependency of its
/// own, so this builds the runtime it needs (a current-thread one: the server
/// only waits on stdio and has no CPU parallelism) and blocks on
/// [`serve_stdio`] inside it.
pub fn serve_stdio_blocking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(serve_stdio())
}
