# AGENTS.md

The rules of this repository, for any agent that works on it.

## Who owns the project

The project belongs to Lucas Cardozo (`lucasivancardozo27@gmail.com`). Every commit
in the history is authored and committed by that person. No bot, no co-author and no
second author appear in it, and none is added later.

## Commits

- One commit per work unit, in the style of the repository: Conventional Commits,
  with a message in Spanish that says what changed and why.
- **No commit carries an attribution footer.** No `Co-Authored-By:`, no
  `Signed-off-by:`, no `Generated with ...`. The message says what changed and why,
  and nothing else. The project belongs to the reader, not to the tool that wrote it.
- The author and the committer are always Lucas Cardozo. Do not change the identity
  of a commit to make it look like someone else's, and do not add a second author to
  one.

The workflow `Comprobaciones` has a job that fails when a commit breaks the second
or the third rule, so the rule is checked, not only written down.

## Checks

- The engine: `cargo test --workspace`.
- The app: `bun install --frozen-lockfile`, `bun run test` and `bun run typecheck`,
  in `crates/genmsi-app`.
- The desktop shell: `cargo test`, in `crates/genmsi-app/src-tauri`.

## The documents of the work

The work of a feature is tracked in `odd/tasks/`: a document with the specs, the
tasks and the log. The document of the feature is the source of truth, and the log
is where the evidence goes.
