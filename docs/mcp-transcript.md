# An agent through the MCP server

This is the end-to-end transcript of an agent that builds a small model,
validates it, runs it and reads its trace, using nothing but the tools of the
GenMSI MCP server (`docs/mcp.md`).

Everything below is a real exchange: the server was started as
`./target/debug/genmsi-mcp` and driven over stdio with the JSON-RPC messages
shown here, one request at a time. The only abbreviation is the model
document: on the wire it travels as one JSON string, and it is shown formatted
below because it is easier to read.

## The model the agent writes

A workshop with arrivals every ten time units, one queue and a termination
counter of one transaction:

```xml
<gmodel versioncode="1"><gclasses/><modules>
  <module moduleid="0" moduleclass="main" owner="0" caption="Taller" name="Taller" kind="C">
    <links>
      <link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/>
      <link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/>
      <link frommoduleid="3" fromioid="2" tomoduleid="4" toioid="1"/>
    </links>
    <inputs/><outputs/>
  </module>
  <module moduleid="1" moduleclass="Generate" owner="0" caption="Llegadas" name="Llegadas" kind="A">
    <outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Queue"/></outputs>
    <props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props>
  </module>
  <module moduleid="2" moduleclass="Queue" owner="0" caption="Cola" name="Cola" kind="A">
    <inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Queue"/></inputs>
    <outputs><output ioid="2" name="Salida1" caption="" mylabel="A_Depart"/></outputs>
    <props><property name="Queue" hint="" value="cola" type="EntityUse" subType="Queue" visible="1" unique="0"/></props>
  </module>
  <module moduleid="3" moduleclass="Depart" owner="0" caption="Salida" name="Salida" kind="A">
    <inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Depart"/></inputs>
    <outputs><output ioid="2" name="Salida1" caption="" mylabel="A_Terminate"/></outputs>
    <props><property name="Queue" hint="" value="cola" type="EntityUse" subType="Queue" visible="1" unique="0"/></props>
  </module>
  <module moduleid="4" moduleclass="Terminate" owner="0" caption="Fin" name="Fin" kind="A">
    <inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Terminate"/></inputs>
    <props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props>
  </module>
</modules></gmodel>
```

The agent has no file yet, so it passes the document inline as `model`.

## 1. The handshake

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"agente","version":"1"}}}
```

```json
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"rmcp","version":"3.6.0"},"instructions":"GenMSI: a discrete-event simulation engine for GPSS-style models. Use `lint` to validate a model, `simulate` to run it, `get_trace` to read its events and `explain_model` to describe it. Give the model either as `path` (a .gmdl file) or as `model` (the .gmdl document as text)."}}
```

The client answers with the `notifications/initialized` notification and asks
what the server offers.

## 2. The tools

```json
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
```

The server answers with the four tools of D7 and their input schemas:

| Tool | Properties of the input schema |
|---|---|
| `explain_model` | `model` (string or null), `path` (string or null) |
| `get_trace` | `limit` (integer, default 100), `model`, `path`, `seed` (integer, default 1), `until` (number or null) |
| `lint` | `model`, `path` |
| `simulate` | `model`, `path`, `seed`, `until` |

## 3. The agent validates the model

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lint","arguments":{"model":"<the document above>"}}}
```

```json
{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"{\n  \"schema\": \"genmsi.lint/1\",\n  \"path\": \"<memory>\",\n  \"status\": \"ok\",\n  \"title\": \"Taller\",\n  \"blocks\": 4,\n  \"definitions\": {\n    \"functions\": 0,\n    \"variables\": 0,\n    \"random_variables\": 0,\n    \"storages\": 0,\n    \"tables\": 0,\n    \"qtables\": 0\n  },\n  \"diagnostics\": []\n}"}],"isError":false}}
```

The model compiles: four executable statements, no diagnostics. The document is
the `genmsi.lint/1` one the CLI prints with `genmsi lint --json`.

## 4. The agent runs the model

```json
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"simulate","arguments":{"model":"<the document above>","seed":7}}}
```

```json
{"jsonrpc":"2.0","id":4,"result":{"content":[{"type":"text","text":"..."}],"isError":false}}
```

The text is the `genmsi.report/1` document:

```json
{
  "schema": "genmsi.report/1",
  "title": "Taller",
  "seed": 7,
  "start_time": 0.0,
  "end_time": 10.0,
  "end_reason": "terminated",
  "transactions": 1,
  "events": 4,
  "blocks": [
    { "line": 1, "node": 1, "class": "Generate", "label": null, "operands": "10,,,,", "entries": 1, "current": 1, "retries": 0 },
    { "line": 2, "node": 2, "class": "Queue", "label": "A_Queue", "operands": "cola", "entries": 1, "current": 0, "retries": 0 },
    { "line": 3, "node": 3, "class": "Depart", "label": "A_Depart", "operands": "cola", "entries": 1, "current": 0, "retries": 0 },
    { "line": 4, "node": 4, "class": "Terminate", "label": "A_Terminate", "operands": "1", "entries": 1, "current": 0, "retries": 0 }
  ],
  "entities": {
    "facilities": {},
    "storages": {},
    "queues": {
      "cola": {
        "name": "cola",
        "content": 0,
        "max": 1,
        "entries": 1,
        "zero_entries": 1,
        "total_time": 0.0,
        "area": 0.0,
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

(The block rows and the entity maps are shown compacted; the server sends the
document with the same field order and the same values the CLI sends.)

## 5. The agent reads the trace

```json
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"get_trace","arguments":{"model":"<the document above>","seed":7,"limit":4}}}
```

```json
{
  "schema": "genmsi.trace/1",
  "title": "Taller",
  "seed": 7,
  "start_time": 0.0,
  "end_time": 10.0,
  "end_reason": "terminated",
  "limit": 4,
  "records": 4,
  "truncated": true,
  "events": [
    { "time": 10.0, "xact": 1, "block": 0, "class": "Generate", "label": null, "event": "generate", "detail": "" },
    { "time": 10.0, "xact": 1, "block": 1, "class": "Queue", "label": "A_Queue", "event": "enter", "detail": "" },
    { "time": 10.0, "xact": 1, "block": 1, "class": "Queue", "label": "A_Queue", "event": "leave", "detail": "-> 2" },
    { "time": 10.0, "xact": 1, "block": 2, "class": "Depart", "label": "A_Depart", "event": "enter", "detail": "" }
  ]
}
```

The transaction arrives at time 10, joins the queue, leaves it and goes to the
departure. `truncated: true` says the bound of four records may have cut the
trace.

## 6. The agent reads what the model says

```json
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"explain_model","arguments":{"model":"<the document above>"}}}
```

```json
{
  "schema": "genmsi.explain/1",
  "program": {
    "blocks": [
      { "class": "Generate", "label": null, "node": 1, "operands": "10,,,,", "ports": ["Salida1"] },
      { "class": "Queue", "label": "A_Queue", "node": 2, "operands": "cola", "ports": ["Salida1"] },
      { "class": "Depart", "label": "A_Depart", "node": 3, "operands": "cola", "ports": ["Salida1"] },
      { "class": "Terminate", "label": "A_Terminate", "node": 4, "operands": "1", "ports": [] }
    ],
    "diagnostics": [],
    "functions": [],
    "qtables": [],
    "random_variables": [],
    "start_counter": 1.0,
    "storages": {},
    "tables": [],
    "title": "Taller",
    "variables": []
  },
  "glossary": {
    "Depart": "Removes Cantidad transactions from the queue Queue, without using a facility.",
    "Generate": "Creates transactions: Media is the mean time between them, Cantidad how many to create (0 means forever), with the optional Modificador, OffSet and Prioridad.",
    "Queue": "The transaction joins the queue Queue; Cantidad units leave it.",
    "Terminate": "Ends the run when Unidades transactions reach the statement (zero means the statement never fires)."
  }
}
```

The agent now knows what each statement of the model does and which operands
it takes, which is what it needs to write the next model.

## 7. The agent makes a mistake and fixes it

The agent forgets the name of the queue. The server does not crash and does
not return an empty report: it returns a tool error that says which block is
wrong and where it stands in the document.

```json
{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"lint","arguments":{"model":"<the same document with value=\"\" instead of value=\"cola\">"}}}
```

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"error\": \"invalid_block\",\n  \"message\": \"Queue is required\",\n  \"path\": \"<memory>\",\n  \"line\": 1,\n  \"column\": 662,\n  \"node\": 2,\n  \"class\": \"Queue\"\n}"
      }
    ],
    "isError": true
  }
}
```

The agent reads `node: 2` and `class: "Queue"`, opens the document at line 1,
column 662, finds the second `<module>` element, gives the queue its name and
calls `lint` again.

## Reproducing it

```bash
cargo build -p genmsi-mcp
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"agente","version":"1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lint","arguments":{"path":"model.gmdl"}}}' \
  | ./target/debug/genmsi-mcp
```

`crates/genmsi-mcp/tests/tools.rs` runs the same exchange in process, over a
duplex transport: it lists the four tools and runs every reference model
through them.
