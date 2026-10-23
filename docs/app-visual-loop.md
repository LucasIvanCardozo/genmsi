# The visual loop of genmsi-app

The UI of this app is reviewed with a browser, not by reading the code: the
frontend runs under Vite, `agent-browser` opens it, the loop clicks, takes a
screenshot in `/tmp` and looks at it before the slice is closed.

The loop is part of the definition of done of every slice. The screenshots
are **not** committed: the repository does not carry images (the phase's
decision `S4`).

The whole loop is one runnable script, `crates/genmsi-app/scripts/
check-visual-loop.sh`; it runs these steps in order, takes the screenshots
and exits non-zero if any step fails. The manual steps below are the same
commands, spelled out one by one — when the script changes, this document
changes with it.

## What it takes

```bash
# The engine the app talks to (the CLI and the app's engine facade).
cargo build -p genmsi-cli -p genmsi-app-engine

# The frontend's dependencies.
cd crates/genmsi-app && bun install
```

The dev bridge spawns `target/debug/genmsi` and `target/debug/genmsi-app-engine`.
Both paths can be changed with `GENMSI_BIN` and `GENMSI_APP_BIN`.

The loop edits a **copy** of a reference model in the development work
directory (`crates/genmsi-app/.devmodels/`): the reference models of
`fixtures/models` are read-only oracle material, and the development path
writes only in that directory.

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
export AGENT_BROWSER_SESSION="genmsi-$(agent-browser session id --scope worktree --prefix fase4b)"
agent-browser open http://127.0.0.1:5173/
agent-browser set viewport 1440 1000   # the graph of a reference model is a tall column
agent-browser wait --text "GenMSI"
agent-browser snapshot -i              # the interactive elements, with their refs
```

## 3. Drive the slice

```bash
# The model picker: the options arrive after the model list, so wait for
# one model's name before selecting. Selecting earlier is a silent no-op.
agent-browser wait --text "TP2__problema1"

# Open a reference model. `agent-browser select` takes a CSS selector and the
# option's value (the model's path, as the model list lists it).
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
```

### The editing steps

```bash
# The copy the loop edits.
cp fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl crates/genmsi-app/.devmodels/visual.gmdl
agent-browser select '[aria-label="Modelo"]' "crates/genmsi-app/.devmodels/visual.gmdl"
agent-browser wait --text "Puerta Banco"

# A new node, created from the catalogue. The list of classes scrolls: the
# class has to be on the picture before it can be clicked.
agent-browser click '[aria-label="Añadir bloque"]'
agent-browser wait --text "Generar"
agent-browser scrollintoview '[data-block-class="Generate"]'
agent-browser click '[data-block-class="Generate"]'
agent-browser wait --fn "document.querySelectorAll('.react-flow__node').length > 18"
agent-browser screenshot /tmp/genmsi-app/04-nodo-nuevo.png

# A second node, and the connection between the two: the new Generate feeds
# the new Terminate, so the link does not replace any other one. React Flow
# listens to the pointer events, so the connection is a real mouse drag.
agent-browser click '[aria-label="Añadir bloque"]'
agent-browser wait --text "Terminar"
agent-browser scrollintoview '[data-block-class="Terminate"]'
agent-browser click '[data-block-class="Terminate"]'
agent-browser click 'button[title="Encuadrar el grafo"]'
agent-browser mouse move 372 807
agent-browser mouse down left
agent-browser mouse move 484 891 --duration 400 --steps 12
agent-browser mouse up left
agent-browser wait --fn "document.querySelectorAll('.react-flow__edge').length > 9"
agent-browser screenshot /tmp/genmsi-app/05-conectado.png

# The node is dragged: one drag is one undo entry.
agent-browser mouse move 368 780
agent-browser mouse down left
agent-browser mouse move 248 720 --duration 400 --steps 12
agent-browser mouse up left
agent-browser screenshot /tmp/genmsi-app/06-arrastrado.png

# Undo takes the drag back, and redo puts it back.
agent-browser press "Control+z"
agent-browser screenshot /tmp/genmsi-app/07-deshacer.png
agent-browser press "Control+Shift+z"
agent-browser screenshot /tmp/genmsi-app/08-rehacer.png

# The model is saved, and the app says so.
agent-browser find role button click --name "Guardar"
agent-browser wait --text "Se guardó"
agent-browser screenshot /tmp/genmsi-app/09-guardado.png

# The file is reopened: the change is in the file, not only in the session.
agent-browser open http://127.0.0.1:5173/
agent-browser wait --text "GenMSI"
agent-browser select '[aria-label="Modelo"]' "crates/genmsi-app/.devmodels/visual.gmdl"
agent-browser wait --text "Puerta Banco"
agent-browser screenshot /tmp/genmsi-app/10-reabierto.png
```

### A model the engine refuses

```bash
agent-browser select '[aria-label="Modelo"]' "fixtures/models/TP9__tp9.gmdl"
agent-browser wait --text "El modelo no compila"
agent-browser screenshot /tmp/genmsi-app/11-error.png
```

`agent-browser wait --text ...` is the wait to use after an action: the app
is a single page that renders when the engine's document arrives, so the
text of the thing that appeared is the condition to wait for.

### The monitor

The window replays a finished run: the engine publishes the trace of the run
next to the report, and the app rebuilds the state of the run by walking it.
The run lives on the graph itself, so the reproduction is the picture of the
model with the run on it.

```bash
# The model of the monitor: it has queues, facilities and tables, so the
# charts have something to draw.
agent-browser select '[aria-label="Modelo"]' "fixtures/models/TP4__supermercado.gmdl"
agent-browser wait --text "supermercado"

# The run it needs.
agent-browser wait --text "Ejecutar y reproducir"
agent-browser find role button click --name "Ejecutar y reproducir"
agent-browser wait --text "Registros"
agent-browser screenshot /tmp/genmsi-app/12-monitor.png

# The replay is paused and put at the middle of the run, so the picture is
# the same on every run. React reads the value of the input from its own
# tracker, so the value is set through the native setter and the input event
# is dispatched: the component sees the change.
agent-browser find role button click --name "Pausar"
agent-browser eval "(() => { const input = document.querySelector('[aria-label=\"Posición de la reproducción\"]'); const max = Number(input.max); const value = String(Math.floor(max / 2)); const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set; setter.call(input, value); input.dispatchEvent(new Event('input', { bubbles: true })); return value; })()"
agent-browser screenshot /tmp/genmsi-app/13-monitor-reproduccion.png

# The charts of the run: the time series of the entities and the histogram
# of the tables.
agent-browser click '[data-monitor-panel="graficos"]'
agent-browser wait --text "Histograma"
agent-browser screenshot /tmp/genmsi-app/14-monitor-graficos.png

# The report of the run, with the numbers of the instant the replay is at.
agent-browser click '[data-monitor-panel="reporte"]'
agent-browser wait --text "Colas"
agent-browser screenshot /tmp/genmsi-app/15-monitor-reporte.png
```

## 4. Look, fix, repeat

Read the screenshot (`agent-browser screenshot` prints the path; open the file
with the image reader of the session). The things to look at:

- the graph is placed with the model's own geometry, and the flow is
  framed: no block is cut off;
- the selected block shows its properties, decoded, with one editor per
  declared type;
- the new node is on the picture, connected to the other one, and the
  labels of the link are the original editor's (`A_<Class>`);
- the undo takes the drag back and the redo puts it back;
- the diagnostics say what the model compiles to, after every change;
- the report's tables carry the engine's numbers, with the original's comma
  decimals;
- the monitor's clock moves with the replay, the blocks that hold
  transactions are marked, and the panel shows the content of the queues and
  the owner of the facilities at that instant;
- the charts show the time series of the entities and the histogram of the
  tables of the model, with the frequencies the report publishes;
- a model the engine refuses shows the engine's message, not an empty graph;
- no text is in another language than Spanish.

When something is wrong: fix it, reload (`agent-browser open` again or
`agent-browser reload`), and take the screenshot again. The loop ends when
the picture is what the slice asks for.

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
   `genmsi simulate --json`, and the pure logic of the editor; that is the
   `test` script of `package.json`).
4. `bunx tsc --noEmit`.
5. Close the browser and the server.

## The desktop shell

The shell (`crates/genmsi-app/src-tauri/`) runs the same frontend and the same
adapter; the loop above is the one the phase reviews, because a browser is
what `agent-browser` drives. Its own smoke check is
`crates/genmsi-app/scripts/check-tauri-shell.sh`: the window opens under
`Xvfb`, the picture can be captured and the log proves that the frontend
reaches the facade through `invoke`.

The debug build of the shell loads the frontend from the dev server, so the
server has to run; and the webview needs the software renderer
(`LIBGL_ALWAYS_SOFTWARE=1`) and the disabled compositing mode
(`WEBKIT_DISABLE_COMPOSITING_MODE=1`), or it paints nothing under `Xvfb`.
