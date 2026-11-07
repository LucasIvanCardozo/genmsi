# CLAUDE.md

The rules of this repository are in `AGENTS.md`. The short version, because it is the
one that matters most:

- The project belongs to Lucas Cardozo. Every commit is authored and committed by
  him: no bot, no co-author, no second author.
- **No commit carries an attribution footer.** No `Co-Authored-By:`, no
  `Signed-off-by:`, no `Generated with ...`. The message says what changed and why,
  and nothing else.
- One commit per work unit, Conventional Commits, message in Spanish.
- Before finishing: `cargo test --workspace`; in `crates/genmsi-app`,
  `bun run test` and `bun run typecheck`.

`AGENTS.md` has the whole thing, including the checks of the desktop shell.
