# The visual loop of genmsi-app

The UI of this app is reviewed with a browser, not by reading the code: the
frontend runs under Vite, `agent-browser` opens it, the loop clicks, takes a
screenshot in `/tmp` and looks at it before the slice is closed.

The loop is part of the definition of done of every slice. The screenshots are
**not** committed: the repository does not carry images (the phase's decision
`S4`).

The whole loop is one runnable script, `crates/genmsi-app/scripts/
check-visual-loop.sh`; it runs these steps in order, takes the four
screenshots and exits non-zero if any step fails. The manual steps below are
the same commands, spelled out one by one — when the script changes, this
document changes with it.

## What it takes

```bash
# The engine the app talks to (the CLI and the app's engine facade).
cargo build -p genmsi-cli -p genmsi-app-engine

# The frontend's dependencies.
cd crates/genmsi-app && bun install
```

The dev bridge spawns `target/debug/genmsi` and `target/debug/genmsi-app-engine`.
Both paths can be changed with `GENMSI_BIN` and `GENMSI_APP_BIN`.

## 1. Start the dev server in the background

```bash
cd crates/genmsi-app
bun run dev > /tmp/genmsi-vite.log 2>&1 &
echo $! > /tmp/genmsi-vite.pid
```

Wait until it answers (the loop must not race the server):

```bash
until curl -sf http://127.0.0.1:5173/ > /dev/null; do sleep 0.3; done
```

The port is fixed (`5173`, `strictPort`), so a second server fails loudly
instead of moving the address.

## 2. Open it and look at it

```bash
export AGENT_BROWSER_SESSION="genmsi-$(agent-browser session id --scope worktree --prefix fase4)"
agent-browser open http://127.0.0.1:5173/
agent-browser wait --text "GenMSI"
agent-browser snapshot -i          # the interactive elements, with their refs
```

## 3. Drive the slice

```bash
# The model picker: the options arrive after `GET /api/models`, so wait for
# one model's name before selecting. Selecting earlier is a silent no-op.
agent-browser wait --text "TP2__problema1"

# Open a reference model. `agent-browser select` takes a CSS selector and the
# option's value (the model's path, as `GET /api/models` lists it); the older
# `find role combobox select` form does not exist in agent-browser 0.38.
agent-browser select '[aria-label="Modelo"]' "fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl"
agent-browser wait --text "Puerta Banco"
agent-browser screenshot /tmp/genmsi-app/01-grafo.png

# Select a block of the graph by its node id (a click by text lands on the
# element that covers the node) and read its properties.
agent-browser click '.react-flow__node[data-id="6"]'
agent-browser screenshot /tmp/genmsi-app/02-propiedades.png

# Run the model and read the report.
agent-browser find role button click --name "Simular"
agent-browser wait --text "Bloques"
agent-browser screenshot /tmp/genmsi-app/03-reporte.png

# A model the engine refuses: the engine's message, not an empty graph.
agent-browser select '[aria-label="Modelo"]' "fixtures/models/TP9__tp9.gmdl"
agent-browser wait --text "Unlink"
agent-browser screenshot /tmp/genmsi-app/04-error.png
```

`agent-browser wait --text ...` is the wait to use after an action: the app is a
single page that renders when the engine's document arrives, so the text of
the thing that appeared is the condition to wait for.

## 4. Look, fix, repeat

Read the screenshot (`agent-browser screenshot` prints the path; open the file
with the image reader of the session). The things to look at:

- the graph is placed with the model's own geometry, and the containers count
  in the frame: no block and no container is cut off;
- the selected block shows its properties, decoded;
- the report's tables carry the engine's numbers, with the original's comma
  decimals;
- a model the engine refuses shows the engine's message, not an empty graph;
- no text is in another language than Spanish.

When something is wrong: fix it, reload (`agent-browser open` again or
`agent-browser reload`), and take the screenshot again. The loop ends when the
picture is what the slice asks for.

## 5. Close the loop

```bash
agent-browser close
kill "$(cat /tmp/genmsi-vite.pid)"
```

The server's log stays in `/tmp/genmsi-vite.log`; it is where a failed request
shows up.

## The checklist of a slice

1. `cargo build -p genmsi-cli -p genmsi-app-engine` (the engine did not move,
   but the build must be there).
2. `crates/genmsi-app/scripts/check-visual-loop.sh`, or the manual steps
   above: start the dev server, wait for it, open it, take the screenshots of
   the flow the slice adds, look at them, fix what they show.
3. `node --test tests/*.test.ts` (the dev bridge's bytes against
   `genmsi simulate --json`; that is the `test` script of `package.json`).
4. `bunx tsc --noEmit`.
5. Close the browser and the server.

## The desktop shell

The shell (`crates/genmsi-app/src-tauri/`) runs the same frontend and the same
adapter; the loop above is the one the phase reviews, because a browser is
what `agent-browser` drives. To try the shell:

```bash
cd crates/genmsi-app
bunx @tauri-apps/cli dev
```

The shell's crate is outside the repository's workspace: its dependency tree
is heavy and the workspace's build must stay as it was.
