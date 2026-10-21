# scripts

Tooling that supports development but is not shipped in the binaries.

- `wine-oracle/` (Phase 0) — harness to run the original `genmsi.exe` under
  Wine against a model and capture its outputs. Before first use, back up the
  original `Genmsi_20230711` folder and work on the copy.
- `inventory/` (Phase 0) — scans `.gmdl` models and reports which block
  classes and properties they actually use; defines the Phase 1 block subset.

Scripts are small, boring and self-contained; when a script grows logic worth
testing, it moves into a crate.
