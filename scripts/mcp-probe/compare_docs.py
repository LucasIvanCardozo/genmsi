#!/usr/bin/env python3
"""Compare the two answers of the MCP server: only the intended changes may show up.

    scripts/mcp-probe/compare_docs.py before.txt after.txt

The two texts are the ones `mcp_diff.py` prints. The comparison knows what
the change was meant to change: it fails on anything else.
"""

import json
import re
import sys


def sections(path):
    text = open(path, encoding="utf-8").read()
    parts = re.split(r"^=== (.+) ===$", text, flags=re.M)
    out = {}
    for name, body in zip(parts[1::2], parts[2::2]):
        out[name.strip()] = body.strip()
    return out


base = sections(sys.argv[1])
new = sections(sys.argv[2])

problems = []

# (1) The list: the same classes, the same titles; the new one adds the
#     description and nothing else.
base_list = json.loads(base["catalogue (list)"])
new_list = json.loads(new["catalogue (list)"])
if base_list["schema"] != new_list["schema"]:
    problems.append("the schema of the list changed")
if base_list["skeleton"] != new_list["skeleton"]:
    problems.append("the skeleton changed")
if len(base_list["classes"]) != len(new_list["classes"]):
    problems.append("the number of classes changed")
described = 0
for before, after in zip(base_list["classes"], new_list["classes"]):
    if before["class"] != after["class"] or before["title"] != after["title"]:
        problems.append(f"the class {before} changed into {after}")
    if after["description"]:
        described += 1
    if set(after) != {"class", "title", "description"}:
        problems.append(f"unexpected fields in {after}")
print(f"classes: {len(new_list['classes'])}, with a description: {described}")

# (2) The details: the same shape, the same ports, the same template; the
#     tooltip is decoded.
for name in ("catalogue Generate", "catalogue Advance"):
    before = json.loads(base[name])
    after = json.loads(new[name])
    for field in ("schema", "class", "title", "inputs", "outputs", "template"):
        if before[field] != after[field]:
            problems.append(f"{name}: the field {field} changed")
    if set(after) != {"schema", "class", "title", "description", "inputs", "outputs", "properties", "template"}:
        problems.append(f"{name}: unexpected fields {sorted(after)}")
    for b, a in zip(before["properties"], after["properties"]):
        for field in ("name", "ty", "subtype", "unique", "options"):
            if b[field] != a[field]:
                problems.append(f"{name}: the property {b['name']} changed its {field}")
        if b["hint"] == a["hint"]:
            problems.append(f"{name}: the tooltip of {b['name']} was not decoded")
        if "%" in a["hint"]:
            problems.append(f"{name}: the tooltip of {b['name']} is still encoded: {a['hint']}")
        if a["hint"] != re.sub(r"%([0-9A-Fa-f]{2})", lambda m: chr(int(m.group(1), 16)), b["hint"]):
            problems.append(f"{name}: the tooltip of {b['name']} is not the decoded one")

# (3) The explain document: the glossary moved to the core, the answer must
#     not change.
if base["explain_model"] != new["explain_model"]:
    problems.append("the explain document changed")
else:
    print("explain_model: identical")

if problems:
    print("PROBLEMS:")
    for problem in problems:
        print(" -", problem)
    sys.exit(1)
print("the only differences are the description and the decoded tooltips")
