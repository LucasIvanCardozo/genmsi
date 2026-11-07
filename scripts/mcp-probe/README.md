# mcp-probe

The tools that drive the GenMSI MCP server over stdio and compare what it
answers.

The server speaks JSON-RPC over stdin/stdout, one message per line, so a
probe is a process of its own: no MCP client library is needed to ask the
server a question, and the answer is a text that two runs can be compared
with.

## `mcp_call.py` — what the catalogue answers

```bash
scripts/mcp-probe/mcp_call.py target/debug/genmsi-mcp
```

It asks the server for the list of the classes and for the document of one
class, and prints both. The first argument is the binary of the server: the
script starts it without arguments, so it has to be the server itself
(`target/debug/genmsi-mcp`, the binary of `cargo build -p genmsi-mcp`). The
CLI binary (`target/debug/genmsi`) asks for a subcommand and would not serve
the protocol; the same server is reachable as `genmsi mcp`.

## `mcp_diff.py` — what the documents a change touches answer

```bash
scripts/mcp-probe/mcp_diff.py target/debug/genmsi-mcp fixtures/models/TP2__problema1.gmdl
```

It asks for the same documents a second time, plus the explain document of a
model, and prints them as sections. The answer is meant to be kept: the one
of the commit before the change and the one of the commit after it. The
first argument is the same server binary as in `mcp_call.py`.

## `compare_docs.py` — the change that published the class descriptions

```bash
scripts/mcp-probe/compare_docs.py before.txt after.txt
```

It compares the two answers and fails when something changed that the change
was not meant to change. It is written for one change — the one that gave
every class its description, decoded the tooltips and moved the glossary to
the core — and its rules are the rules of that change: the list of the
classes may only gain descriptions, the details of a class may only gain
decoded tooltips, and the explain document must not change at all.

## Reproducing the measurement of the agent

The evidence of the measurements is in five directories, one per run:

- `odd/tasks/medida/` — the first run: the model of the dining hall, written
  by hand and saved with `save_model`, with the assumptions of the model, the
  classes it uses and the findings about their behaviour.
- `odd/tasks/medida2/` — the second run: the same scenario in a fresh
  session that had not read the previous one, with its own model (written
  with `save_model`) and its own notes.
- `odd/tasks/medida3/` — the third run: the same model built with the
  `apply_edit` verb instead of XML written by hand.
- `odd/tasks/medida4/` — the fourth run: the agent renaming a block and
  changing a property of a model that a person has open with unsaved
  changes, and leaving the change as a proposal.
- `odd/tasks/medida5/` — the fifth run: the same, from a cold agent that had
  not read the code of the product, over the model of the vaccination
  centre.

Each of them keeps the transcript of the calls (`mcp-calls.log`), the model
the agent built and the notes. The transcript was written by these tools:

```bash
# the binary the probe drives
cargo build -p genmsi-mcp

# the transcript of the calls of the agent (the session drives the tools)
#   → odd/tasks/medida/mcp-calls.log

# the answers of the server, before and after the change
scripts/mcp-probe/mcp_diff.py target/debug/genmsi-mcp fixtures/models/TP2__problema1.gmdl > after.txt
#   (the "before" is the same command on the commit before the change)

# the comparison that says only the intended changes showed up
scripts/mcp-probe/compare_docs.py before.txt after.txt
```

The two probes carry their own copy of the JSON-RPC client: they are two
instruments of one measurement and they grew apart, so the duplication is
left as it is instead of hiding it behind a module that only the two of them
use.
