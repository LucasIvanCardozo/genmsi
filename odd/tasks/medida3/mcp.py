#!/usr/bin/env python3
"""A minimal MCP client over stdio.

It starts the GenMSI MCP server, speaks the protocol with it and prints the
answer of one tool call. The measurement of the MCP counts the calls an agent
makes: this script is how a call is made from a shell, and the log of the
calls is what the measurement is made of.

    python3 mcp.py <tool> '<json arguments>'

The server is the one this branch builds: `target/debug/genmsi-mcp`.
"""

import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
SERVER = ROOT / "target" / "debug" / "genmsi-mcp"
PROTOCOL = "2025-06-18"


def call(tool, arguments):
    """Call one tool and return the answer of the server."""
    server = subprocess.Popen(
        [str(SERVER)],
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
    return answer


def main():
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    tool = sys.argv[1]
    arguments = json.loads(sys.argv[2]) if len(sys.argv) > 2 else {}
    print(json.dumps(call(tool, arguments), indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
