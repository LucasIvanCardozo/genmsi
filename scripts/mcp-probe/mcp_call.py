#!/usr/bin/env python3
"""Drive the GenMSI MCP server over stdio and print what `catalogue` answers.

The server speaks JSON-RPC over stdin/stdout, one message per line, so the
probe is a process of its own: no MCP client library is needed to ask the
server a question.

    scripts/mcp-probe/mcp_call.py target/debug/genmsi
"""

import json
import subprocess
import sys

BINARY = sys.argv[1]


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

    def send(self, message):
        self.process.stdin.write(json.dumps(message) + "\n")
        self.process.stdin.flush()

    def request(self, method, params=None):
        self.next_id += 1
        message = {"jsonrpc": "2.0", "id": self.next_id, "method": method}
        if params is not None:
            message["params"] = params
        self.send(message)
        while True:
            line = self.process.stdout.readline()
            if not line:
                raise RuntimeError("the server closed the connection")
            answer = json.loads(line)
            if answer.get("id") == self.next_id:
                return answer

    def notify(self, method, params=None):
        message = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            message["params"] = params
        self.send(message)

    def call(self, tool, arguments):
        answer = self.request(
            "tools/call", {"name": tool, "arguments": arguments}
        )
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

listed = server.call("catalogue", {})
print("=== catalogue (the list of the 66 classes) ===")
print(listed)

detail = server.call("catalogue", {"class": "Generate"})
print("=== catalogue { class: 'Generate' } ===")
print(detail)

server.close()
