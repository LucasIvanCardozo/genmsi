# scripts

Tooling that supports development but is not shipped in the binaries.

- `wine-oracle/` — harness to run the executable of the reference application
  under Wine against a model and capture its outputs. The binaries of that
  application live in a local copy (`Genmsi_20230711/`, and the backup
  `Genmsi_20230711.backup/` that the harness always uses) that is **not
  versioned**: it is in `.gitignore`. Back the folder up before the first use
  and work on the copy. `scripts/wine-oracle/README.md` explains how the
  harness works with those binaries.
- `inventory_blocks.py` — scans the `.gmdl` models of `fixtures/models/` and
  reports which block classes and properties they actually use; its output is
  `docs/BLOCK-INVENTORY.md`.
- `bump-version.sh` — writes the version in the seven copies that follow the
  source and fails when one of them holds another version. Run it when the
  version of a release changes.
- `dev-app.sh` — runs the app in the browser against the real engine, without
  installing the desktop shell. Run it to see a change of the frontend.
- `install-linux.sh` — installs the desktop app, its launcher and its icon for
  the current user; with `--with-mcp` it also installs the CLI and the guide
  of the agent and configures the MCP. Run it when the app has to be on this
  machine.
- `mcp-probe/` — tools that drive the MCP server over stdio and compare what
  it answers. Use them to see what a change of the documents moved.
- `wine-oracle/probes/` — the experiments that settle a question about a
  class: a model run in the reference application and in our engine, with the
  numbers compared and the verdict written down.

Scripts are small, boring and self-contained; when a script grows logic worth
testing, it moves into a crate. The exception is
`scripts/wine-oracle/make-benches.py`: it is a script and it carries its own
tests (`scripts/wine-oracle/test_make_benches.py`).
