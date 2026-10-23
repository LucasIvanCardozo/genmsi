#!/usr/bin/env python3
"""Evidence table for the first-arrival rule of a GENERATE block.

Reads the original's baseline reports (fixtures/outputs/*.report.txt) and prints,
per GENERATE row: horizon, the original's entry count, and what each reading
predicts.

Counting model of the original (proved in odd/tasks/notas-primera-llegada.md):

  entries = arrivals already executed  +  1 transaction pending in the FEC

The pending one is proved by TP7_1/TP7_2 (`GENERATE 60,,600` -> 52 entries while
the downstream block only has 51) and by TP6_1 (`GENERATE 90,30,2700` -> 14
entries while the queue only has 13).

Readings:
  media  : the first increment is A (the mean) -> first arrival at t = A
  cero   : the first arrival is at t = 0 (offset ignored unless given)
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "fixtures" / "outputs"

HORIZON = re.compile(r"^\s*(\d+)\s+(\d+)\s+\d+\s+\d+\s+\d+\s*$")
ROW = re.compile(
    r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\S*)\s+(\S+)\s+(.*?)\s*$"
)


def parse(report: Path):
    lines = report.read_text(encoding="utf-8", errors="replace").splitlines()
    horizon = None
    rows = []
    for raw in lines:
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
        rows.append(
            {
                "line": int(line),
                "entries": int(entries),
                "current": int(current),
                "loc": loc,
                "ops": rest.strip(),
            }
        )
    return horizon, rows


def operands(ops: str):
    """Split the operand list `A,B,C,D,E` into its five slots."""
    parts = [p.strip() for p in ops.split(",")]
    parts += [""] * (5 - len(parts))
    return parts[:5]


def num(text: str):
    try:
        return float(text)
    except (TypeError, ValueError):
        return None


def predictions(a, c, horizon_end, limit, deterministic):
    """Return (arrivals_media, arrivals_cero) inside [0, horizon_end].

    Only deterministic generators (no dispersion) are predicted exactly; a
    generator with a dispersion is reported as a range.
    """
    if not deterministic:
        return None, None
    media = num(a)
    offset = num(c)
    limit_n = num(limit)
    if limit_n is not None:
        # A limited generator stops at the limit: both readings agree on the
        # number of arrivals only when the limit is reached inside the horizon.
        return "limitado", "limitado"
    if media is None:
        return None, None
    start_media = offset if offset is not None else media
    start_cero = offset if offset is not None else 0.0
    if media <= 0:
        return None, None

    def count(start):
        if start > horizon_end:
            return 0
        return int((horizon_end - start) // media) + 1

    return count(start_media), count(start_cero)


def main():
    only = sys.argv[1:] if len(sys.argv) > 1 else None
    print(
        f"{'modelo':<42}{'lin':>4}{'horizonte':>10}{'entradas orig':>14}"
        f"{'llega media':>13}{'llega 0':>9}  veredicto"
    )
    for report in sorted(OUT.glob("*.report.txt")):
        stem = report.name[: -len(".report.txt")]
        if only and not any(o in stem for o in only):
            continue
        horizon, rows = parse(report)
        if horizon is None or not rows:
            continue
        start, end = horizon
        for row in rows:
            a, b, c, d, e = operands(row["ops"])
            # a dispersion is a number (deviation) or a function (modifier);
            # empty or zero means the interarrival is exactly A
            deterministic = b == "" or num(b) == 0.0
            media, cero = predictions(a, c, end, d, deterministic)
            orig = row["entries"]
            if media is None:
                verdict = "no discrimina (con dispersion)" if not deterministic else "?"
            elif media == "limitado":
                verdict = "no discrimina (limitado)"
            else:
                # the original counts arrivals + 1 pending
                exp_media = media + 1
                exp_cero = cero + 1
                if orig == exp_media and orig == exp_cero:
                    verdict = "no discrimina"
                elif orig == exp_media:
                    verdict = "MEDIA"
                elif orig == exp_cero:
                    verdict = "CERO"
                else:
                    verdict = f"NINGUNA (orig={orig}, media={exp_media}, cero={exp_cero})"
            print(
                f"{stem:<42}{row['line']:>4}{str(end):>10}{orig:>14}"
                f"{str(media):>13}{str(cero):>9}  {verdict}   [{row['ops']}]"
            )


if __name__ == "__main__":
    main()
