# genmsi-app (placeholder)

Tauri 2 + React 19 desktop application: graphical model editor, live monitor
and reports. This crate joins the Cargo workspace in **Phase 4**, after the
engine and its data format are frozen by tests.

Planned frontend stack: React 19 + TypeScript, `@xyflow/react` (node editor),
Zustand (UI state), Tailwind + shadcn/ui, Recharts (charts). Shared types are
generated from the Rust schema (ts-rs or typify) so the GUI cannot drift from
the engine.

Do not add simulation logic here: the app is a thin client over `genmsi-core`.
