#!/usr/bin/env python3
"""Print the JSON of the last call in mcp-calls.log (summarized)."""
import json, sys
log = open("/home/lucas/Documentos/Workspace/Genmsi/odd/tasks/medida5/mcp-calls.log").read()
chunks = log.split("=== CALL: ")[1:]
last = chunks[-1]
head, _, body = last.partition("\n")
print("CALL:", head)
try:
    d = json.loads(body)
except Exception:
    print(body[:4000]); raise SystemExit
if isinstance(d, dict) and "source" in d:
    d["source"] = "<%d chars>" % len(d["source"])
print(json.dumps(d, ensure_ascii=False, indent=1))
