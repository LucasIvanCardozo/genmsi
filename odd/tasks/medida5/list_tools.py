#!/usr/bin/env python3
"""List the tools the MCP server publishes (schema of every tool).

The client of odd/tasks/medida4/mcp.py only does tools/call; a real agent
gets the schemas at connect time, so this helper prints them once. It does
not call any tool.
"""
import json
import os
import pathlib
import subprocess

ROOT = pathlib.Path("/home/lucas/Documentos/Workspace/Genmsi")
SERVER = os.environ.get("GENMSI_MCP", str(ROOT / "target" / "release" / "genmsi-mcp"))

server = subprocess.Popen([SERVER], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, text=True, cwd=str(ROOT))


def send(message):
    server.stdin.write(json.dumps(message) + "\n")
    server.stdin.flush()


def read():
    return json.loads(server.stdout.readline())


send({"jsonrpc": "2.0", "id": 1, "method": "initialize",
      "params": {"protocolVersion": "2025-06-18", "capabilities": {},
                 "clientInfo": {"name": "medida", "version": "1"}}})
read()
send({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
send({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
answer = read()
server.stdin.close()
server.wait(timeout=10)
for tool in answer["result"]["tools"]:
    print("=" * 70)
    print(tool["name"], "-", tool.get("description", ""))
    print(json.dumps(tool.get("inputSchema", {}), indent=2, ensure_ascii=False))
