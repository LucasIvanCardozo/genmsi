# An agent through the MCP server

A real session, captured against the server: every payload below is what the
server answered, not a sketch of it. The agent builds a small model —a queue
with a resource— without writing a line of XML, runs it, reads its trace, and
fixes a mistake the engine reports.

- The server is `genmsi mcp` (the same binary as the CLI) and speaks JSON-RPC
  over stdin and stdout.
- The model is the neutral `Modelo1`: `Generate` → `Queue` → `Seize` →
  `Advance` → `Depart` → `Terminate`.
- Every run is seeded, so every number below can be reproduced.

## 1. The handshake

The client says hello, and the server answers with what it is and how it
wants to be used. The `instructions` are the part an agent should read before
anything else:

```
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "genmsi-transcript",
      "version": "1"
    }
  }
}
```

The answer carries the `instructions` of the server: the procedure in one
paragraph, which is what an agent should read before anything else:

> GenMSI: a discrete-event simulation engine for GPSS-style models. Call `catalogue` before you write a model: it says which block classes exist and what the XML of a block of each class looks like; ask for every class you need in ONE call, with `classes`. Use `apply_edit` to build or to change a model: it takes a batch of the engine's editing commands (`addBlock`, `connect`, `setProperty`, `moveBlock`, `removeBlock`, ...) and writes the model; the batch is atomic, so a command that fails leaves the model as it was. Use `lint` to validate a model, `simulate` to run it, `get_trace` to read its ev…

## 2. The tools

`tools/list` is the answer: **8 tools** — `app_state`, `apply_edit`, `catalogue`, `explain_model`, `get_trace`, `lint`, `save_model`, `simulate`.

```json
{
  "schema": "genmsi.tools/1",
  "tools": [
    {
      "name": "app_state",
      "required": [],
      "properties": []
    },
    {
      "name": "apply_edit",
      "required": [
        "commands"
      ],
      "properties": [
        "commands",
        "dry_run",
        "layout",
        "model",
        "path",
        "summary",
        "target"
      ]
    },
    {
      "name": "catalogue",
      "required": [],
      "properties": [
        "class",
        "classes"
      ]
    },
    {
      "name": "explain_model",
      "required": [],
      "properties": [
        "model",
        "path"
      ]
    },
    {
      "name": "get_trace",
      "required": [],
      "properties": [
        "classes",
        "from",
        "limit",
        "model",
        "nodes",
        "path",
        "seed",
        "start",
        "summary",
        "to",
        "until",
        "xact"
      ]
    },
    {
      "name": "lint",
      "required": [],
      "properties": [
        "model",
        "path"
      ]
    },
    {
      "name": "save_model",
      "required": [],
      "properties": [
        "model",
        "path",
        "summary",
        "target"
      ]
    },
    {
      "name": "simulate",
      "required": [],
      "properties": [
        "model",
        "path",
        "seed",
        "start",
        "until"
      ]
    }
  ]
}
```


## 3. The vocabulary

Before writing a model, the agent asks what the classes are. One call with
`classes` returns every one it needs — the properties each class declares and
the ports it has (`ioid` is the number a link names):

```json
{
  "schema": "genmsi.classes/1",
  "classes": [
    {
      "class": "Generate",
      "properties": [
        "Cantidad",
        "Media",
        "Modificador",
        "OffSet",
        "Prioridad"
      ],
      "inputs": [],
      "outputs": [
        1
      ]
    },
    {
      "class": "Queue",
      "properties": [
        "Cantidad",
        "Queue"
      ],
      "inputs": [
        1
      ],
      "outputs": [
        2
      ]
    },
    {
      "class": "Seize",
      "properties": [
        "Facility"
      ],
      "inputs": [
        1
      ],
      "outputs": [
        2
      ]
    },
    {
      "class": "Advance",
      "properties": [
        "Media",
        "Modificador"
      ],
      "inputs": [
        1
      ],
      "outputs": [
        2
      ]
    },
    {
      "class": "Depart",
      "properties": [
        "Cantidad",
        "Queue"
      ],
      "inputs": [
        1
      ],
      "outputs": [
        2
      ]
    },
    {
      "class": "Terminate",
      "properties": [
        "Unidades"
      ],
      "inputs": [
        1
      ],
      "outputs": []
    }
  ]
}
```


| Class | Properties | Inputs | Outputs |
|---|---|---|---|
| `Generate` | `Cantidad`, `Media`, `Modificador`, `OffSet`, `Prioridad` | — | 1 |
| `Queue` | `Cantidad`, `Queue` | 1 | 2 |
| `Seize` | `Facility` | 1 | 2 |
| `Advance` | `Media`, `Modificador` | 1 | 2 |
| `Depart` | `Cantidad`, `Queue` | 1 | 2 |
| `Terminate` | `Unidades` | 1 | — |

## 4. The agent writes the model

`apply_edit` takes a batch of the engine's own editing commands and applies
it atomically: if one fails, none is applied. The `container` is the id of the
block that holds the new one, and `0` is the root of the model.

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "apply_edit",
    "arguments": {
      "commands": [
        {"op": "addBlock", "class": "Generate", "container": 0, "x": 80, "y": 80},
        {"op": "addBlock", "class": "Queue", "container": 0, "x": 240, "y": 80},
        {"op": "addBlock", "class": "Seize", "container": 0, "x": 400, "y": 80},
        {"op": "addBlock", "class": "Advance", "container": 0, "x": 560, "y": 80},
        {"op": "addBlock", "class": "Depart", "container": 0, "x": 720, "y": 80},
        {"op": "addBlock", "class": "Terminate", "container": 0, "x": 880, "y": 80},
        {"op": "connect", "from": 1, "fromIoid": 1, "to": 2, "toIoid": 1},
        {"op": "connect", "from": 2, "fromIoid": 2, "to": 3, "toIoid": 1},
        {"op": "connect", "from": 3, "fromIoid": 2, "to": 4, "toIoid": 1},
        {"op": "connect", "from": 4, "fromIoid": 2, "to": 5, "toIoid": 1},
        {"op": "connect", "from": 5, "fromIoid": 2, "to": 6, "toIoid": 1},
        {"op": "setProperty", "id": 1, "property": "Media", "value": "10"},
        {"op": "setProperty", "id": 2, "property": "Queue", "value": "Cola"},
        {"op": "setProperty", "id": 3, "property": "Facility", "value": "Recurso"},
        {"op": "setProperty", "id": 4, "property": "Media", "value": "5"},
        {"op": "setProperty", "id": 5, "property": "Queue", "value": "Cola"},
        {"op": "setProperty", "id": 6, "property": "Unidades", "value": "1"},
        {"op": "setCaption", "id": 1, "caption": "Llegadas"},
        {"op": "setCaption", "id": 2, "caption": "Cola de espera"},
        {"op": "setCaption", "id": 3, "caption": "Recurso"},
        {"op": "setCaption", "id": 4, "caption": "Servicio"}
      ]
    }
  }
}
```


The answer says what happened: how many commands were applied, which blocks
were created, and whether the model was written to the disk. A first call with
`dry_run: true` answers the same thing with `"written": false` and carries the
`source` the batch would have written —the model, composed by the engine.

## 5. The agent validates the model

```json
{
  "schema": "genmsi.lint/1",
  "path": "/tmp/taller.gmdl",
  "status": "ok",
  "title": "Modelo1",
  "blocks": 6,
  "definitions": {
    "functions": 0,
    "variables": 0,
    "random_variables": 0,
    "storages": 0,
    "tables": 0,
    "qtables": 0
  },
  "diagnostics": []
}
```


## 6. The agent runs the model

```json
{
  "schema": "genmsi.report/1",
  "title": "Modelo1",
  "seed": 42,
  "start_time": 0.0,
  "end_time": 12.0,
  "end_reason": "terminated",
  "transactions": 1,
  "events": 7,
  "blocks": [
    {
      "line": 1,
      "node": 1,
      "class": "Generate",
      "label": null,
      "operands": "10,30,,,",
      "entries": 2,
      "current": 1,
      "retries": 0,
      "counts_block": true
    },
    {
      "line": 2,
      "node": 2,
      "class": "Queue",
      "label": "A_Queue",
      "operands": "Cola",
      "entries": 1,
      "current": 0,
      "retries": 0,
      "counts_block": true
    },
    {
      "line": 3,
      "node": 3,
      "class": "Seize",
      "label": "A_Seize",
      "operands": "Recurso",
      "entries": 1,
      "current": 0,
      "retries": 0,
      "counts_block": true
    },
    {
      "line": 4,
      "node": 4,
      "class": "Advance",
      "label": "A_Advance",
      "operands": "5,5",
      "entries": 1,
      "current": 0,
      "retries": 0,
      "counts_block": true
    },
    {
      "line": 5,
      "node": 5,
      "class": "Depart",
      "label": "A_Depart",
      "operands": "Cola",
      "entries": 1,
      "current": 0,
      "retries": 0,
      "counts_block": true
    },
    {
      "line": 6,
      "node": 6,
      "class": "Terminate",
      "label": "A_Terminate",
      "operands": "1",
      "entries": 1,
      "current": 0,
      "retries": 0,
      "counts_block": true
    }
  ],
  "entities": {
    "facilities": {
      "Recurso": {
        "name": "Recurso",
        "entries": 1,
        "busy_time": 4.0,
        "busy": true,
        "available": true,
        "retry": 0,
        "inter": 0
      }
    },
    "storages": {},
    "queues": {
      "Cola": {
        "name": "Cola",
        "content": 0,
        "max": 1,
        "entries": 1,
        "zero_entries": 0,
        "total_time": 4.0,
        "area": 4.0,
        "retry": 0
      }
    },
    "chains": {},
    "logic": {},
    "savevalues": {},
    "tables": {},
    "qtables": {}
  },
  "diagnostics": []
}
```


The report carries the seed it used, so the run can be repeated: `simulate`
with the same `seed` gives the same numbers.

## 7. The agent reads the trace

The trace can be filtered, and the filter is part of the answer —so the agent
knows what it is looking at:

```json
{
  "schema": "genmsi.trace/1",
  "title": "Modelo1",
  "seed": 42,
  "start_time": 0.0,
  "end_time": 12.0,
  "end_reason": "terminated",
  "limit": 3,
  "records": 2,
  "truncated": false,
  "filter": {
    "classes": [
      "Terminate"
    ],
    "nodes": [],
    "xact": null,
    "from": null,
    "to": null
  },
  "events": [
    {
      "time": 12.0,
      "xact": 1,
      "block": 5,
      "node": 6,
      "class": "Terminate",
      "label": "A_Terminate",
      "event": "enter",
      "detail": ""
    },
    {
      "time": 12.0,
      "xact": 1,
      "block": 5,
      "node": 6,
      "class": "Terminate",
      "label": "A_Terminate",
      "event": "terminate",
      "detail": ""
    }
  ]
}
```


## 8. The agent makes a mistake, and the engine says how to fix it

The agent writes a model that uses a storage nobody declared:

```json
{
  "name": "apply_edit",
  "arguments": {
    "commands": [
      {"op": "addBlock", "class": "Generate", "container": 0, "x": 80, "y": 80},
      {"op": "addBlock", "class": "StorageUse", "container": 0, "x": 240, "y": 80},
      {"op": "addBlock", "class": "Terminate", "container": 0, "x": 400, "y": 80},
      {"op": "connect", "from": 1, "fromIoid": 1, "to": 2, "toIoid": 1},
      {"op": "connect", "from": 2, "fromIoid": 2, "to": 3, "toIoid": 1},
      {"op": "setProperty", "id": 1, "property": "Media", "value": "10"},
      {"op": "setProperty", "id": 2, "property": "Storage", "value": "Piezas"},
      {"op": "setProperty", "id": 3, "property": "Unidades", "value": "1"}
    ]
  }
}
```


The model is written, but the engine does not consider it sound. `lint` says
what is missing and what to do about it:

```json
{
  "schema": "genmsi.lint/1",
  "path": "/tmp/taller-sin-declarar.gmdl",
  "status": "diagnostics",
  "title": "Modelo1",
  "blocks": 7,
  "definitions": {
    "functions": 0,
    "variables": 0,
    "random_variables": 0,
    "storages": 0,
    "tables": 0,
    "qtables": 0
  },
  "diagnostics": [
    "the model uses the Storage 'Piezas' in the StorageUse block 2, and no StorageDef block declares 'Piezas' (the model declares no Storage at all). Declare 'Piezas' with a StorageDef block, or use one of the declared names. The storage has no capacity: the run stops when a transaction reaches it."
  ]
}
```


The fix is one more block —a `StorageDef` that declares the storage— and the
model goes through:

```json
{
  "name": "apply_edit",
  "arguments": {
    "commands": [
      {"op": "addBlock", "class": "StorageDef", "container": 0, "x": 240, "y": 200},
      {"op": "setProperty", "id": 4, "property": "Storage", "value": "Piezas"},
      {"op": "setProperty", "id": 4, "property": "Tamaño", "value": "20"}
    ]
  }
}
```


```json
{
  "schema": "genmsi.lint/1",
  "path": "/tmp/taller-sin-declarar.gmdl",
  "status": "ok",
  "title": "Modelo1",
  "blocks": 7,
  "definitions": {
    "functions": 0,
    "variables": 0,
    "random_variables": 0,
    "storages": 1,
    "tables": 0,
    "qtables": 0
  },
  "diagnostics": []
}
```


## Reproducing it

The server is the `mcp` subcommand of the CLI:

```sh
cargo build --release
./target/release/genmsi mcp
```

It speaks JSON-RPC over stdin and stdout, one message per line, so it can be
driven without any client library. The probe of the repository asks a single
question and prints the answer:

```sh
scripts/mcp-probe/mcp_call.py target/release/genmsi-mcp
```

The exchange of this document was captured the same way: `initialize`,
`notifications/initialized`, `tools/list`, and then one `tools/call` per tool.

## See also

- [`docs/mcp.md`](mcp.md): the tools, the documents and the errors.
- [`skill/genmsi-modeling/SKILL.md`](../skill/genmsi-modeling/SKILL.md): the
  procedure and the vocabulary for building a model.
