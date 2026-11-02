#!/usr/bin/env python3
"""A minimal MCP client over stdio, for the measurement of the proposal.

It starts the GenMSI MCP server, speaks the protocol with it and prints the
answer of each tool call. The measurement of the MCP counts the calls an
agent makes: this script is how a call is made from a shell, and the log of
the calls is what the measurement is made of.

    python3 mcp.py <tool> '<json arguments>'

The server is the one this branch builds: `target/release/genmsi-mcp`. The
data directory of the app comes from the environment (`XDG_DATA_HOME`), so
the measurement runs against a directory of its own.
"""

import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
SERVER = os.environ.get("GENMSI_MCP", str(ROOT / "target" / "release" / "genmsi-mcp"))
PROTOCOL = "2025-06-18"


def call(tool, arguments):
    """Call one tool and return the answer of the server."""
    server = subprocess.Popen(
        [SERVER],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=str(ROOT),
    )

    def send(message):
        server.stdin.write(json.dumps(message) + "\n")
        server.stdin.flush()

    def read():
        line = server.stdout.readline()
        if not line:
            raise SystemExit("the server closed the connection")
        return json.loads(line)

    send(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": PROTOCOL,
                "capabilities": {},
                "clientInfo": {"name": "medida", "version": "1"},
            },
        }
    )
    read()
    send({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
    send(
        {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {"name": tool, "arguments": arguments},
        }
    )
    answer = read()
    server.stdin.close()
    server.wait(timeout=10)

    if "error" in answer:
        return json.dumps(answer["error"], indent=2, sort_keys=True)
    result = answer["result"]
    if result.get("isError"):
        return "ISERROR " + result["content"][0]["text"]
    return result["content"][0]["text"]


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        raise SystemExit(1)
    print(call(sys.argv[1], json.loads(sys.argv[2])))


if __name__ == "__main__":
    main()
