# fixtures

Reference inputs and expected outputs used by tests and the Wine oracle.

- `models/` — the 25 reference `.gmdl` models. The tests read them:
  `crates/genmsi-cli/tests/cli.rs` lints and simulates them, and
  `crates/genmsi-app/tests/replay.test.ts` replays two of them in the app.
- `outputs/` — the outputs captured for the models of `models/` under the Wine
  harness (`scripts/wine-oracle/`). Three files per model:
  - `<model>.gpss.txt` — the listing the model compiles into;
  - `<model>.report.txt` — the report of the run. This is the **baseline**:
    `crates/genmsi-core/tests/oracle.rs` compares the run of the engine against
    it;
  - `<model>.wine.log` — the log of the Wine session that captured them.

Layout is intentionally boring: a test names the model and the expected file
it compares against.

The name of a model file carries the coursework it comes from and the model
itself, separated by `__` (`Nombre__modelo.gmdl`), and the three outputs carry
the same name with the spaces turned into underscores:
`Nombre__modelo.gpss.txt`, `Nombre__modelo.report.txt` and
`Nombre__modelo.wine.log`. (`TP2__problema1` has no `wine.log` in the tree.)
