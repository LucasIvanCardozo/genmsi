//! The `genmsi-mcp` binary: the GenMSI MCP server over stdio.
//!
//! The server speaks MCP on standard input and output; standard error is
//! free for the failure that stopped it.

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match genmsi_mcp::serve_stdio().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
