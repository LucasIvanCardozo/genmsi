#!/usr/bin/env python3
"""Build the model that shows every block class, unconnected.

The model is the bench of the review of the shapes: one block of every
class of the catalogue, in a grid, with no links between them, so that
every silhouette can be looked at with the same eye.

The blocks are made by the engine itself, through its own editing
session, so the model is a valid model by construction and every block
carries the footprint its class declares.

Usage:
  crates/genmsi-app/scripts/make-catalogue-model.py [OUTPUT]

The output is `crates/genmsi-app/fixtures/catalogo-clases.gmdl` by
default. The engine binaries must be built:
  cargo build -p genmsi-cli -p genmsi-app-engine
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
ENGINE = REPO_ROOT / "target" / "debug" / "genmsi-app-engine"
DEFAULT_OUTPUT = REPO_ROOT / "crates" / "genmsi-app" / "fixtures" / "catalogo-clases.gmdl"

# The grid: how many classes in a row, and the room each one gets. The
# room is bigger than the biggest footprint, so no block touches another
# and every silhouette is seen whole.
COLUMNS = 6
CELL_WIDTH = 190
CELL_HEIGHT = 130

# The properties a block needs to stand on its own. A block with no value
# of its own does not compile: the engine refuses the model, and a model
# the engine refuses is not the bench the review needs. The name of the
# variable and the expression it carries are the values of the class.
REQUIRED = {
    "Variable": {"Expresion": "1", "NombreVar": "Var1"},
}


def catalogue() -> list[dict]:
    """The classes the engine offers, with their footprint."""
    out = subprocess.run(
        [str(ENGINE), "catalogue"], capture_output=True, text=True, check=True
    ).stdout
    return json.loads(out)["classes"]


def build(classes: list[dict], output: Path) -> None:
    """Create the model: one block of every class, and save it."""
    requests: list[dict] = [{"op": "new"}]
    for index, entry in enumerate(classes):
        column = index % COLUMNS
        row = index // COLUMNS
        requests.append(
            {
                "op": "apply",
                "command": {
                    "op": "addBlock",
                    "class": entry["class"],
                    "container": 0,
                    "x": 40 + column * CELL_WIDTH,
                    "y": 40 + row * CELL_HEIGHT,
                },
            }
        )
        for name, value in REQUIRED.get(entry["class"], {}).items():
            requests.append(
                {
                    "op": "apply",
                    "command": {
                        "op": "setProperty",
                        "id": index + 1,
                        "property": name,
                        "value": value,
                    },
                }
            )
    requests.append({"op": "save", "path": str(output)})

    payload = "".join(json.dumps(request) + "\n" for request in requests)
    out = subprocess.run(
        [str(ENGINE), "session"], input=payload, capture_output=True, text=True
    )
    if out.returncode != 0:
        raise SystemExit(f"the engine failed:\n{out.stderr}")

    answers = [json.loads(line) for line in out.stdout.splitlines() if line.strip()]
    errors = [answer for answer in answers if answer.get("ok") is False]
    if errors:
        raise SystemExit(f"the engine refused:\n{json.dumps(errors[:3], indent=2)}")


def patch_unlink_label(text: str) -> str:
    """Give the "unlinked transactions" output of every Unlink its label.

    The engine refuses a model whose UNLINK sends the unlinked
    transactions out without a label, and the editing protocol has no
    command to put a label on a port. The model is written by the engine
    and this is the one attribute the protocol cannot reach.
    """
    return re.sub(
        r'(<output ioid="3" name="SalidaAlt" caption="[^"]*" mylabel=")[^"]*(")',
        r"\1Libre\2",
        text,
    )


def main() -> int:
    output = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else DEFAULT_OUTPUT
    if not ENGINE.exists():
        raise SystemExit(
            f"{ENGINE} is missing; run cargo build -p genmsi-cli -p genmsi-app-engine"
        )
    classes = catalogue()
    output.parent.mkdir(parents=True, exist_ok=True)
    build(classes, output)
    output.write_text(patch_unlink_label(output.read_text(encoding="utf-8")))
    print(f"==> {len(classes)} classes in {output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
