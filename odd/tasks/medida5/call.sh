#!/usr/bin/env bash
# Wrapper for the measurement: runs one MCP tool call through the client of
# odd/tasks/medida4/mcp.py and appends the raw answer to mcp-calls.log.
set -u
cd /home/lucas/Documentos/Workspace/Genmsi
TOOL="$1"; shift
ARGS="$1"; shift || true
{
  echo "=== CALL: $TOOL $ARGS"
} >> odd/tasks/medida5/mcp-calls.log
# NOTE: the XDG_DATA_HOME has to be ABSOLUTE: with a relative path the server
# ignores it and reads the real user's data dir (the real desktop app).
OUT="$(XDG_DATA_HOME=/home/lucas/Documentos/Workspace/Genmsi/odd/tasks/medida5/home python3 odd/tasks/medida4/mcp.py "$TOOL" "$ARGS" 2>&1)"
printf '%s\n' "$OUT" >> odd/tasks/medida5/mcp-calls.log
echo "--- $TOOL $ARGS"
# the log keeps the whole answer; the console shows a truncated one
printf '%s\n' "$OUT" | head -c 3000
