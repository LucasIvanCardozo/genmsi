//! MCP server for the GenMSI engine (`genmsi mcp` over stdio).
//!
//! Planned tool surface (Phase 3): `simulate`, `lint`, `get_trace`,
//! `explain_model`. The engine logic lives in `genmsi-core`; this binary only
//! adapts it to the MCP protocol. `rmcp` is added when this phase starts.

fn main() {
    eprintln!("genmsi-mcp is not implemented yet (planned for Phase 3)");
    eprintln!("see docs/ROADMAP.md for the phase plan");
    std::process::exit(2);
}
