#!/usr/bin/env python3
"""Compares the original's GENERATE rows with ours, model by model.

For every model of `fixtures/models` that has a baseline report, the script:

  1. reads the original's report (horizon and GENERATE rows),
  2. runs our engine with that horizon and reads our GENERATE rows,
  3. matches the rows by their operands (in order, for repeated operands),
  4. prints the counts side by side with the two readings of the first arrival.

The counting model of the original (proved in odd/tasks/notas-primera-llegada.md):

  entries(ours)   = arrivals already executed
  entries(original) = arrivals already executed + 1 transaction pending in the FEC
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODELS = ROOT / "fixtures" / "models"
OUT = ROOT / "fixtures" / "outputs"
CLI = ROOT / "target" / "release" / "genmsi"

HORIZON = re.compile(r"^\s*(\d+)\s+(\d+)\s+\d+\s+\d+\s+\d+\s*$")
ROW = re.compile(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\S*)\s+(\S+)\s+(.*?)\s*$")


def parse_report(text: str):
    horizon = None
    rows = []
    for raw in text.splitlines():
        if "START_TIME" in raw:
            continue
        m = HORIZON.match(raw)
        if m and horizon is None:
            horizon = (int(m.group(1)), int(m.group(2)))
            continue
        m = ROW.match(raw)
        if not m:
            continue
        line, entries, current, retry, loc, kind, rest = m.groups()
        if kind != "GENERATE":
            continue
        rows.append({"line": int(line), "entries": int(entries), "ops": rest.strip()})
    return horizon, rows


def key(ops: str) -> str:
    """The operand list, normalized (the original writes `FN$x`, we write `FN:x`).

    An empty operand and a zero are the same value, so they match.
    """
    parts = [p.strip() for p in ops.split(",")]
    parts += [""] * (5 - len(parts))
    out = []
    for p in parts[:5]:
        p = p.replace("FN:", "FN$")
        if p == "":
            p = "0"
        out.append(p)
    return ",".join(out)


def num(text):
    try:
        return float(text)
    except (TypeError, ValueError):
        return None


def predict(a, c, end, deterministic, limit=None):
    """Arrivals inside [0, end] under both readings (None when not predictable)."""
    media = num(a)
    offset = num(c)
    if limit is not None:
        # a limited generator stops at the limit: both readings agree
        return "limitado", "limitado"
    if not deterministic or media is None or media <= 0:
        return None, None
    start_media = offset if offset is not None else media
    start_cero = offset if offset is not None else 0.0

    def count(start):
        if start > end:
            return 0
        return int((end - start) // media) + 1

    return count(start_media), count(start_cero)


def main():
    seed = sys.argv[1] if len(sys.argv) > 1 else "1"
    print(
        f"{'modelo':<40}{'line':>5}{'operandos':<26}{'orig':>6}{'nuestro':>9}"
        f"{'orig-1':>8}{'media':>7}{'cero':>6}  veredicto"
    )
    for model in sorted(MODELS.glob("*.gmdl")):
        stem = model.stem
        report = OUT / f"{stem.replace(' ', '_')}.report.txt"
        if not report.exists():
            continue
        horizon, orig_rows = parse_report(report.read_text(encoding="utf-8", errors="replace"))
        if horizon is None or not orig_rows:
            continue
        run = subprocess.run(
            [str(CLI), "simulate", str(model), "--seed", seed, "--until", str(horizon[1])],
            capture_output=True,
            text=True,
        )
        if run.returncode != 0:
            print(f"{stem:<40} run error: {run.stderr.strip()[:60]}")
            continue
        _, our_rows = parse_report(run.stdout)
        # match by operands, in order
        buckets = {}
        for row in our_rows:
            buckets.setdefault(key(row["ops"]), []).append(row)
        for row in orig_rows:
            k = key(row["ops"])
            ours = buckets.get(k)
            our = ours.pop(0) if ours else None
            a, b, c, d, e = (row["ops"].split(",") + [""] * 5)[:5]
            deterministic = b.strip() == "" or num(b) == 0.0
            limit = num(d) if d.strip() != "" else None
            media, cero = predict(a, c, horizon[1], deterministic, limit)
            orig = row["entries"]
            our_count = our["entries"] if our else None
            if media == "limitado":
                verdict = "no discrimina (limitado)"
            elif media is None:
                if num(a) in (None, 0.0):
                    verdict = "no discrimina (media vacia: 0 en las dos lecturas)"
                else:
                    verdict = "no discrimina (dispersion)"
            else:
                if orig - 1 == media and orig - 1 == cero:
                    verdict = "no discrimina"
                elif orig - 1 == media:
                    verdict = "MEDIA"
                elif orig - 1 == cero:
                    verdict = "CERO"
                else:
                    verdict = f"ninguna: orig-1={orig - 1} media={media} cero={cero}"
            print(
                f"{stem:<40}{row['line']:>5}{row['ops']:<26}{orig:>6}"
                f"{str(our_count):>9}{orig - 1:>8}{str(media):>7}{str(cero):>6}  {verdict}"
            )


if __name__ == "__main__":
    main()
