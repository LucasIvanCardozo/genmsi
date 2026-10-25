#!/usr/bin/env python3
"""Print what the MCP server answers for the documents a change touches.

The answer is written as a text of sections, so two runs (the one before the
change and the one after it) can be compared with `compare_docs.py`.

    scripts/mcp-probe/mcp_diff.py target/debug/genmsi fixtures/models/TP2__problema1.gmdl
"""

import json
import subprocess
import sys

BINARY = sys.argv[1]
MODEL = sys.argv[2]


class Server:
    def __init__(self, binary):
        self.process = subprocess.Popen(
            [binary],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        self.next_id = 0

    def request(self, method, params=None):
        self.next_id += 1
        message = {"jsonrpc": "2.0", "id": self.next_id, "method": method}
        if params is not None:
            message["params"] = params
        self.process.stdin.write(json.dumps(message) + "\n")
        self.process.stdin.flush()
        while True:
            line = self.process.stdout.readline()
            if not line:
                raise RuntimeError("the server closed the connection")
            answer = json.loads(line)
            if answer.get("id") == self.next_id:
                return answer

    def notify(self, method):
        self.process.stdin.write(json.dumps({"jsonrpc": "2.0", "method": method}) + "\n")
        self.process.stdin.flush()

    def call(self, tool, arguments):
        answer = self.request("tools/call", {"name": tool, "arguments": arguments})
        if "error" in answer:
            return json.dumps(answer["error"], indent=2, sort_keys=True)
        return answer["result"]["content"][0]["text"]

    def close(self):
        self.process.stdin.close()
        self.process.wait(timeout=10)


server = Server(BINARY)
server.request(
    "initialize",
    {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "genmsi-scaffold", "version": "1"},
    },
)
server.notify("notifications/initialized")

print("=== catalogue (list) ===")
print(server.call("catalogue", {}))
print("=== catalogue Generate ===")
print(server.call("catalogue", {"class": "Generate"}))
print("=== catalogue Advance ===")
print(server.call("catalogue", {"class": "Advance"}))
print("=== explain_model ===")
print(server.call("explain_model", {"path": MODEL}))

server.close()
