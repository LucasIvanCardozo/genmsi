# fixtures

Reference inputs and expected outputs used by tests and the Wine oracle.

- `models/` — `.gmdl` example models. The user's original models are not
  available yet; Phase 0 starts with hand-written models covering the block
  subset of the inventory.
- `expected/` — outputs produced by the original `genmsi.exe` (via the Wine
  harness, `scripts/`) for the models in `models/`, used as the semantic
  oracle baseline.

Layout is intentionally boring: a test names the model and the expected file
it compares against.
