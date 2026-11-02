#!/usr/bin/env python3
"""Logging wrapper around mcp.py: one MCP call, appended to mcp-calls.log.

    python3 call.py <tool> '<json arguments>'

Prints the answer of the server (same text mcp.py prints) and appends a
record of the call to mcp-calls.log.
"""

import datetime
import json
import os
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import mcp  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
LOG = HERE / "mcp-calls.log"


def list_tools():
    """Ask the server which tools it publishes (not a tool call: a query)."""
    server = subprocess.Popen(
        [mcp.SERVER],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=str(mcp.ROOT),
    )

    def send(message):
        server.stdin.write(json.dumps(message) + "\n")
        server.stdin.flush()

    def read():
        return json.loads(server.stdout.readline())

    send({"jsonrpc": "2.0", "id": 1, "method": "initialize",
          "params": {"protocolVersion": mcp.PROTOCOL, "capabilities": {},
                     "clientInfo": {"name": "medida", "version": "1"}}})
    read()
    send({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
    send({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
    answer = read()
    server.stdin.close()
    server.wait(timeout=10)
    return json.dumps(answer.get("result", answer), indent=2, sort_keys=True)


def main():
    if os.environ.get("GENMSI_CALL_CWD"):
        mcp.ROOT = pathlib.Path(os.environ["GENMSI_CALL_CWD"])
    if sys.argv[1] == "__list__":
        answer = list_tools()
        with LOG.open("a", encoding="utf-8") as fh:
            fh.write("\n===== QUERY tools/list =====\n")
            fh.write(answer.rstrip("\n") + "\n")
        print(answer)
        return
    tool = sys.argv[1]
    arguments = json.loads(sys.argv[2])
    answer = mcp.call(tool, arguments)
    stamp = datetime.datetime.now().isoformat(timespec="seconds")
    with LOG.open("a", encoding="utf-8") as fh:
        fh.write(f"\n===== CALL #{tool} at {stamp} =====\n")
        fh.write(f"tool: {tool}\n")
        fh.write(f"arguments: {json.dumps(arguments, ensure_ascii=False)}\n")
        fh.write(f"env XDG_DATA_HOME: {os.environ.get('XDG_DATA_HOME')}\n")
        fh.write("----- answer -----\n")
        fh.write(answer.rstrip("\n") + "\n")
    print(answer)


if __name__ == "__main__":
    main()
