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

The script (`scripts/check-visual-loop.sh`) does these steps; they are
spelled out here. Three helpers come first, because they are what the app's
own interface allows in a browser.

```bash
# Open a model in the browser.
#
# The app has no dropdown of models: it opens them with the dialog of the
# system, and the browser has no dialog. The recents are the way in — the
# browser keeps them in its own storage — so the loop leaves the path there,
# reloads the page and clicks the model in the menu.
open_model() {
  local path="$1"
  local file stem
  file="$(basename "$path")"
  stem="$(basename "$path" .gmdl)"
  agent-browser eval "localStorage.setItem('genmsi.recent-models', JSON.stringify(['$path']))" >/dev/null
  agent-browser reload >/dev/null
  agent-browser wait --text "GenMSI"
  agent-browser find role button click --name "Recientes"
  agent-browser eval "(() => { const button = Array.from(document.querySelectorAll('button')).find((b) => b.textContent.includes('$file')); if (!button) return 'the model is not in the recents'; button.click(); return 'opened'; })()" >/dev/null
  # The header carries the name of the model that is open.
  agent-browser wait --text "$stem"
}

# The centre of an element, in the coordinates the mouse takes.
centre_of() {
  local selector="$1"
  agent-browser eval "(() => { const el = document.querySelector('$selector'); if (!el) return ''; const r = el.getBoundingClientRect(); return [Math.round(r.x + r.width / 2), Math.round(r.y + r.height / 2)].join(','); })()" |
    tr -d '"'
}

# Where a block is on the canvas, as the graph draws it.
node_position() {
  agent-browser eval "(() => { const el = document.querySelector('.react-flow__node[data-id=\"$1\"]'); return el ? el.style.transform : ''; })()" |
    tr -d '"'
}
```

```bash
# The reviewed model: the graph, framed with the model's own geometry.
open_model "fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl"
agent-browser wait --text "Puerta Banco"
agent-browser screenshot /tmp/genmsi-app/01-grafo.png

# Select a block of the graph by its node id (a click by text lands on the
# element that covers the node) and read its properties.
agent-browser click '.react-flow__node[data-id="6"]'
agent-browser screenshot /tmp/genmsi-app/02-propiedades.png

# Run the model and read the report.
agent-browser find role button click --name "Simular"
agent-browser click '[data-monitor-panel="reporte"]'
agent-browser wait --text "Bloques"
agent-browser click '[data-monitor-aside="bloques"]'
agent-browser screenshot /tmp/genmsi-app/03-catalogo.png

agent-browser screenshot /tmp/genmsi-app/04-reporte.png
# The editing below happens on the graph, so the window goes back to it.
agent-browser click '[data-monitor-panel="reproduccion"]'
agent-browser wait --fn "document.querySelectorAll('.react-flow__node').length > 0"
```

### The editing steps

```bash
# The copy the loop edits.
cp fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl crates/genmsi-app/.devmodels/visual.gmdl
open_model "crates/genmsi-app/.devmodels/visual.gmdl"
agent-browser wait --text "Puerta Banco"

# A new node, created from the catalogue. The classes live in the lateral
# panel, in the tab "Bloques".
agent-browser click '[data-monitor-aside="bloques"]'
agent-browser wait --text "Generar"
# The list of classes scrolls: the class has to be on the picture before it
# can be clicked.
agent-browser scrollintoview '[data-block-class="Generate"]'
agent-browser click '[data-block-class="Generate"]'
agent-browser wait --fn "document.querySelectorAll('.react-flow__node').length > 18"
agent-browser screenshot /tmp/genmsi-app/05-nodo-nuevo.png

# A second node, and the connection between the two: the new Generate feeds
# the new Terminate, so the link does not replace any other one. React Flow
# listens to the pointer events, so the connection is a real mouse drag.
agent-browser scrollintoview '[data-block-class="Terminate"]'
agent-browser click '[data-block-class="Terminate"]'
agent-browser wait --fn "document.querySelectorAll('.react-flow__node').length > 19"
agent-browser find role button click --name "Encuadrar"
agent-browser wait 300

# The drag needs the coordinates of the two handles, read from the picture:
# the blocks move with the model, so fixed coordinates would not land.
from_centre=$(centre_of ".react-flow__node[data-id=\"39\"] .react-flow__handle-bottom")
to_centre=$(centre_of ".react-flow__node[data-id=\"40\"] .react-flow__handle-top")
agent-browser mouse move "${from_centre%,*}" "${from_centre#*,}"
agent-browser mouse down left
agent-browser mouse move "${to_centre%,*}" "${to_centre#*,}" --duration 400 --steps 12
agent-browser mouse up left
agent-browser wait --fn "document.querySelectorAll('.react-flow__edge').length > 9"
agent-browser screenshot /tmp/genmsi-app/06-conectado.png

# The node is dragged: one drag is one undo entry.
POSITION_BEFORE=$(node_position 39)
x=$(centre_of '.react-flow__node[data-id="39"]')
agent-browser mouse move "${x%,*}" "${x#*,}"
agent-browser mouse down left
agent-browser mouse move $(( ${x%,*} - 120 )) $(( ${x#*,} - 60 )) --duration 400 --steps 12
agent-browser mouse up left
agent-browser wait --fn "document.querySelector('.react-flow__node[data-id=\"39\"]').style.transform !== '$POSITION_BEFORE'"
agent-browser screenshot /tmp/genmsi-app/07-arrastrado.png

# Undo takes the drag back.
agent-browser press "Control+z"
agent-browser wait --fn "document.querySelector('.react-flow__node[data-id=\"39\"]').style.transform === '$POSITION_BEFORE'"
agent-browser screenshot /tmp/genmsi-app/08-deshacer.png

# And redo puts it back.
agent-browser press "Control+Shift+z"
agent-browser wait --fn "document.querySelector('.react-flow__node[data-id=\"39\"]').style.transform !== '$POSITION_BEFORE'"
agent-browser screenshot /tmp/genmsi-app/09-rehacer.png

# The re-layout: the button asks the engine to compose the model, and the
# blocks move to the layers of the flow. The composition is the engine's,
# the same one the MCP applies before it writes a model.
POSITION_BEFORE=$(node_position 39)
agent-browser find role button click --name "Reordenar el grafo en capas"
agent-browser wait --fn "document.querySelector('.react-flow__node[data-id=\"39\"]').style.transform !== '$POSITION_BEFORE'"
agent-browser screenshot /tmp/genmsi-app/10-reacomodado.png

# The model is saved, and the app says so.
agent-browser find role button click --name "Guardar"
agent-browser wait --text "Se guardó"
agent-browser screenshot /tmp/genmsi-app/11-guardado.png

# The file is reopened: the change is in the file, not only in the session.
agent-browser open http://127.0.0.1:5173/
agent-browser wait --text "GenMSI"
open_model "crates/genmsi-app/.devmodels/visual.gmdl"
agent-browser wait --text "Puerta Banco"
agent-browser screenshot /tmp/genmsi-app/12-reabierto.png
```

### A model the engine refuses

```bash
open_model "fixtures/models/TP9__tp9.gmdl"
agent-browser wait --text "El modelo no compila"
agent-browser screenshot /tmp/genmsi-app/13-error.png
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
open_model "fixtures/models/TP4__supermercado.gmdl"
agent-browser wait --text "supermercado"

# The run it needs.
agent-browser wait --text "Ejecutar"
agent-browser find role button click --name "Ejecutar"
# The clock of the run advances: the monitor is playing the trace.
agent-browser wait --fn "!document.body.innerText.includes('0:00:00')"
agent-browser screenshot /tmp/genmsi-app/14-monitor.png

# The replay is paused and put at the middle of the run, so the picture is
# the same on every run. React reads the value of the input from its own
# tracker, so the value is set through the native setter and the input event
# is dispatched: the component sees the change.
agent-browser find role button click --name "Pausar la reproducción"
agent-browser eval "(() => { const input = document.querySelector('[aria-label=\"Posición de la reproducción\"]'); const max = Number(input.max); const value = String(Math.floor(max / 2)); const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set; setter.call(input, value); input.dispatchEvent(new Event('input', { bubbles: true })); return value; })()"
agent-browser screenshot /tmp/genmsi-app/15-monitor-reproduccion.png

# The charts of the run: the time series of the entities and the histogram
# of the tables.
agent-browser click '[data-monitor-panel="graficos"]'
agent-browser wait --text "Histograma"
agent-browser screenshot /tmp/genmsi-app/16-monitor-graficos.png

# The report of the run, with the numbers of the instant the replay is at.
agent-browser click '[data-monitor-panel="reporte"]'
agent-browser wait --text "Colas"
agent-browser screenshot /tmp/genmsi-app/17-monitor-reporte.png
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
- the re-layout puts the blocks in layers: that picture is the engine's
  composition, the same one the MCP applies before it writes a model;
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
