# mcp-probe

The tools that drive the GenMSI MCP server over stdio and compare what it
answers.

The server speaks JSON-RPC over stdin/stdout, one message per line, so a
probe is a process of its own: no MCP client library is needed to ask the
server a question, and the answer is a text that two runs can be compared
with.

## `mcp_call.py` — what the catalogue answers

```bash
scripts/mcp-probe/mcp_call.py target/debug/genmsi
```

It asks the server for the list of the classes and for the document of one
class, and prints both. The first argument is the binary of the server.

## `mcp_diff.py` — what the documents a change touches answer

```bash
scripts/mcp-probe/mcp_diff.py target/debug/genmsi fixtures/models/TP2__problema1.gmdl
```

It asks for the same documents a second time, plus the explain document of a
model, and prints them as sections. The answer is meant to be kept: the one
of the commit before the change and the one of the commit after it.

## `compare_docs.py` — only the intended changes may show up

```bash
scripts/mcp-probe/compare_docs.py before.txt after.txt
```

It compares the two answers and fails when something changed that the change
was not meant to change. The comparison knows what the change was for: it is
written for the change it guards, and its rules are the rules of that
change.

## Reproducing the measurement of the agent

The evidence of the phase that published the batch catalogue is in
`odd/tasks/medida/` and `odd/tasks/medida2/`: the transcript of the calls
(`mcp-calls.log`), the model the agent built and the notes. The transcript was
written by these tools:

```bash
# the binary the probe drives
cargo build -p genmsi-cli

# the transcript of the calls of the agent (the session drives the tools)
#   → odd/tasks/medida/mcp-calls.log

# the answers of the server, before and after the change
scripts/mcp-probe/mcp_diff.py target/debug/genmsi fixtures/models/TP2__problema1.gmdl > after.txt
#   (the "before" is the same command on the commit before the change)

# the comparison that says only the intended changes showed up
scripts/mcp-probe/compare_docs.py before.txt after.txt
```

The two probes carry their own copy of the JSON-RPC client: they are two
instruments of one measurement and they grew apart, so the duplication is
left as it is instead of hiding it behind a module that only the two of them
use.
