#!/usr/bin/env python3
"""Build the GUI map assets from the Wikimedia Commons "Diplomacy.svg".

Inputs
  shared/map/source/Diplomacy.commons.svg  (CC BY-SA 3.0, Martin Asal; see shared/map/CREDITS.md)
  shared/map/standard.json                 (province IDs, types, adjacency)

Outputs (all generated; do not edit by hand, re-run this script instead)
  shared/map/standard.svg         base map: one group per province (id="prov-<ID>"),
                                  split-coast areas, SC markers (id="sc-<ID>"), labels
  shared/map/standard.layout.json anchor points for units, labels and SC markers
  shared/map/standard.graph.svg   graph view of the adjacency data (debug/auxiliary view)

Usage: python3 -I tools/map/build_map.py   (run from the repository root; stdlib only)
"""

import hashlib
import json
import math
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAP_DIR = ROOT / "shared" / "map"
SOURCE = MAP_DIR / "source" / "Diplomacy.commons.svg"
SOURCE_SHA256 = "db49bb9fe711b277f91f3da267e5b3a01715820fb6f48564a3969559e4c19307"
SOURCE_URL = "https://commons.wikimedia.org/wiki/File:Diplomacy.svg"

NS = "{http://www.w3.org/2000/svg}"
VIEWBOX = (0, 0, 610, 560)

# Labels in the source that differ from our canonical IDs (docs/en/map.md § 2.9).
ALIASES = {"LYO": "GoL", "MAO": "Mid", "NAO": "NAt", "NWG": "Nrg", "TYS": "Tyn"}
IMPASSABLE = "Swi"

# Split-coast provinces are drawn as two separate areas in the source.
# The rule picks which area is which coast; checked visually (see docs/en/map.md § 7).
SPLIT_COAST_RULES = {
    "Bul": lambda a, b: ("EC", "SC") if a["cx"] > b["cx"] else ("SC", "EC"),  # east part = EC
    "Spa": lambda a, b: ("NC", "SC") if a["cy"] < b["cy"] else ("SC", "NC"),  # north part = NC
    "StP": lambda a, b: ("NC", "SC") if a["cy"] < b["cy"] else ("SC", "NC"),  # north part = NC
}

LABEL_FONT_SIZE = 13
GRAPH_SCALE = 2
GRAPH_COAST_OFFSET = 26
LABEL_CHAR_WIDTH = 7.5  # rough width of one glyph at 13px, for keeping units off labels


# ----- SVG path → polygons ------------------------------------------------------------------------

TOKEN = re.compile(r"[MmLlHhVvCcSsQqTtZz]|-?(?:\d+\.?\d*|\.\d+)(?:e-?\d+)?")
ARGC = {"M": 2, "L": 2, "H": 1, "V": 1, "C": 6, "S": 4, "Q": 4, "T": 2}


def path_polygons(d):
    """Approximate a path by polygons (curve control points are dropped; endpoints kept)."""
    tokens = TOKEN.findall(d)
    polys, cur, x, y, start, cmd, i = [], [], 0.0, 0.0, (0.0, 0.0), None, 0
    while i < len(tokens):
        t = tokens[i]
        if t.isalpha():
            cmd = t
            i += 1
            if cmd in "Zz":
                if cur:
                    polys.append(cur)
                cur, (x, y) = [], start
                continue
        up = cmd.upper()
        n = ARGC[up]
        args = [float(v) for v in tokens[i:i + n]]
        i += n
        rel = cmd.islower()
        if up == "H":
            x = x + args[0] if rel else args[0]
        elif up == "V":
            y = y + args[0] if rel else args[0]
        else:
            ex, ey = args[-2], args[-1]
            x, y = (x + ex, y + ey) if rel else (ex, ey)
        if up == "M":
            if cur:
                polys.append(cur)
            cur, start = [(x, y)], (x, y)
            cmd = "l" if rel else "L"  # implicit lineto after moveto
        else:
            cur.append((x, y))
    if cur:
        polys.append(cur)
    return [p for p in polys if len(p) >= 3]


def inside(pt, polys):
    """Even-odd point-in-polygon over several rings."""
    x, y = pt
    hit = False
    for poly in polys:
        n = len(poly)
        for k in range(n):
            (x1, y1), (x2, y2) = poly[k], poly[(k + 1) % n]
            if (y1 > y) != (y2 > y) and x < (x2 - x1) * (y - y1) / (y2 - y1) + x1:
                hit = not hit
    return hit


def edge_distance(pt, polys):
    x, y = pt
    best = math.inf
    for poly in polys:
        n = len(poly)
        for k in range(n):
            (x1, y1), (x2, y2) = poly[k], poly[(k + 1) % n]
            dx, dy = x2 - x1, y2 - y1
            L = dx * dx + dy * dy
            t = 0 if L == 0 else max(0, min(1, ((x - x1) * dx + (y - y1) * dy) / L))
            best = min(best, math.hypot(x - (x1 + t * dx), y - (y1 + t * dy)))
    return best


def bbox(polys):
    xs = [p[0] for poly in polys for p in poly]
    ys = [p[1] for poly in polys for p in poly]
    return min(xs), min(ys), max(xs), max(ys)


def best_point(polys, holes, avoid_rects, avoid_circles):
    """Point inside `polys` (outside `holes`) farthest from edges, keeping clear of labels/markers."""
    x0, y0, x1, y1 = bbox(polys)
    best, best_score = None, -math.inf
    step = 1.0
    yy = y0 + step / 2
    while yy < y1:
        xx = x0 + step / 2
        while xx < x1:
            pt = (xx, yy)
            if inside(pt, polys) and not inside(pt, holes):
                score = edge_distance(pt, polys + holes)
                for rx0, ry0, rx1, ry1 in avoid_rects:
                    ddx = max(rx0 - xx, 0, xx - rx1)
                    ddy = max(ry0 - yy, 0, yy - ry1)
                    score = min(score, math.hypot(ddx, ddy))
                for cx, cy, r in avoid_circles:
                    score = min(score, math.hypot(xx - cx, yy - cy) - r)
                if score > best_score:
                    best, best_score = pt, score
            xx += step
        yy += step
    if best is None:
        raise SystemExit("no interior point found")
    return [round(best[0], 1), round(best[1], 1)]


# ----- read inputs --------------------------------------------------------------------------------

def load_source():
    raw = SOURCE.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if SOURCE_SHA256 != "PINNED_BELOW" and digest != SOURCE_SHA256:
        raise SystemExit(f"source SVG changed (sha256 {digest}); review it before rebuilding")
    root = ET.fromstring(raw)
    groups = []
    for g in root.findall(NS + "g"):
        areas = [p for p in g if p.tag == NS + "path" and p.get("class") == "s4" or
                 p.tag == NS + "path" and p.get("class") in ("s1", "s3") and not any(
                     q.tag == NS + "path" and q.get("class") == "s4" for q in g)]
        waterways = [p for p in g if p.tag == NS + "path" and p.get("class") == "s1" and any(
            q.tag == NS + "path" and q.get("class") == "s4" for q in g)]
        text = next((t for t in g if t.tag == NS + "text"), None)
        sc = next((s for s in g if s.tag == NS + "g" and s.get("id") == "sc"), None)
        if not areas:
            continue  # standalone coloured SC dots and the starting units: dropped
        label = None
        if text is not None:
            lx, ly = [float(v) for v in re.findall(r"[-\d.]+", text.get("style"))[-2:]]
            label = (text.get("id"), lx, ly)
        sc_center = None
        if sc is not None:
            ring = path_polygons(sc.find(NS + "path").get("d"))[0]
            xs, ys = [p[0] for p in ring], [p[1] for p in ring]
            sc_center = (round((min(xs) + max(xs)) / 2, 1), round((min(ys) + max(ys)) / 2, 1))
        groups.append({
            "areas": [a.get("d") for a in areas],
            "waterways": [w.get("d") for w in waterways],
            "label": label,
            "sc": sc_center,
        })
    backdrop = root.find(NS + "path")
    return digest, backdrop.get("d"), groups


def main():
    digest, backdrop_d, groups = load_source()
    data = json.loads((MAP_DIR / "standard.json").read_text())
    provinces = {p["id"]: p for p in data["provinces"]}
    canon = {pid.upper(): pid for pid in provinces}
    canon[IMPASSABLE.upper()] = IMPASSABLE
    for alias, pid in ALIASES.items():
        canon[alias] = pid

    # Attach each group to a province.
    by_prov = {}
    unlabeled = []
    for g in groups:
        polys = [poly for d in g["areas"] for poly in path_polygons(d)]
        x0, y0, x1, y1 = bbox(polys)
        g["polys"], g["cx"], g["cy"] = polys, (x0 + x1) / 2, (y0 + y1) / 2
        if g["label"] is None:
            unlabeled.append(g)
            continue
        pid = canon.get(g["label"][0].upper())
        if pid is None:
            raise SystemExit(f"unknown label {g['label'][0]!r}")
        if pid in by_prov:
            raise SystemExit(f"duplicate province {pid}")
        by_prov[pid] = [g]
    for g in unlabeled:
        pid = min(SPLIT_COAST_RULES, key=lambda p: math.hypot(by_prov[p][0]["cx"] - g["cx"],
                                                                by_prov[p][0]["cy"] - g["cy"]))
        by_prov[pid].append(g)
    missing = sorted(set(provinces) - set(by_prov))
    if missing:
        raise SystemExit(f"provinces missing from the source SVG: {missing}")
    for pid in SPLIT_COAST_RULES:
        if len(by_prov[pid]) != 2:
            raise SystemExit(f"{pid} should have two coastal areas, has {len(by_prov[pid])}")
    for pid, parts in by_prov.items():
        if pid not in SPLIT_COAST_RULES and len(parts) != 1:
            raise SystemExit(f"{pid} has {len(parts)} areas")

    # Coasts of split provinces.
    coast_of = {}
    for pid, rule in SPLIT_COAST_RULES.items():
        a, b = by_prov[pid]
        ca, cb = rule(a, b)
        expected = sorted(provinces[pid].get("coasts", []))
        if sorted([ca, cb]) != expected:
            raise SystemExit(f"{pid}: coasts {ca},{cb} do not match standard.json {expected}")
        coast_of[id(a)], coast_of[id(b)] = ca, cb

    # Labels, SC markers and anchors.
    label_pos, sc_pos, avoid_rects, avoid_circles = {}, {}, {}, {}
    for pid, parts in by_prov.items():
        lab = next(p["label"] for p in parts if p["label"])
        text = pid
        lx, ly = lab[1], lab[2]
        label_pos[pid] = [lx, ly]
        w = LABEL_CHAR_WIDTH * len(text)
        avoid_rects[pid] = [(lx - 2, ly - LABEL_FONT_SIZE, lx + w + 2, ly + 3)]
        scs = [p["sc"] for p in parts if p["sc"]]
        if len(scs) > 1:
            raise SystemExit(f"{pid} has {len(scs)} SC markers")
        if pid != IMPASSABLE:
            if bool(scs) != provinces[pid]["supplyCenter"]:
                raise SystemExit(f"{pid}: SC marker does not match standard.json")
        if scs:
            sc_pos[pid] = list(scs[0])
        avoid_circles[pid] = [(scs[0][0], scs[0][1], 4 + 4)] if scs else []

    layout_provinces, layout_coasts, coast_center = {}, {}, {}
    for pid in provinces:
        parts = by_prov[pid]
        polys = [poly for p in parts for poly in p["polys"]]
        holes = [poly for p in parts for d in p["waterways"] for poly in path_polygons(d)]
        unit = best_point(polys, holes, avoid_rects[pid], avoid_circles[pid])
        layout_provinces[pid] = {"unit": unit, "label": label_pos[pid], "sc": sc_pos.get(pid)}
        for p in parts:
            if id(p) in coast_of:
                c = coast_of[id(p)]
                layout_coasts[f"{pid}/{c}"] = {
                    "unit": best_point(p["polys"], [], avoid_rects[pid], avoid_circles[pid])
                }
                coast_center[f"{pid}/{c}"] = (p["cx"], p["cy"])

    # ----- standard.svg ---------------------------------------------------------------------------
    out = []
    w = out.append
    w('<?xml version="1.0" encoding="UTF-8"?>')
    w("<!--")
    w("  Diplomacy standard map for diplomacy-gm (generated by tools/map/build_map.py).")
    w(f"  Adapted from \"Diplomacy.svg\" by Martin Asal, {SOURCE_URL}")
    w("  Licensed under CC BY-SA 3.0, https://creativecommons.org/licenses/by-sa/3.0/")
    w("  Changes: element IDs and classes renamed to the project's province IDs, split coasts and")
    w("  supply centres given IDs, starting units and colours removed, labels set to project IDs.")
    w("  This adapted map is licensed under CC BY-SA 3.0 as well. See shared/map/CREDITS.md.")
    w("  Unofficial fan tool. \"Diplomacy\" is a trademark of Hasbro.")
    w("-->")
    vb = " ".join(str(v) for v in VIEWBOX)
    w(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}" width="{VIEWBOX[2]}" height="{VIEWBOX[3]}">')
    w("  <style>")
    w("    .backdrop, .land, .coast { fill: #ffffdd; }")
    w("    .sea, .waterway { fill: #99ccff; }")
    w("    .impassable { fill: #dddddd; }")
    w("    .area, .waterway { stroke: #000000; stroke-linejoin: round; }")
    w("    .supply-center { fill: #ffffdd; stroke: #000000; }")
    w(f'    .label {{ font-size: {LABEL_FONT_SIZE}px; font-family: "Liberation Sans", Arial, sans-serif; fill: #000000; pointer-events: none; }}')
    w("  </style>")
    w(f'  <path class="backdrop" d="{backdrop_d}"/>')
    w('  <g id="provinces">')
    order = sorted(by_prov, key=lambda p: (p != IMPASSABLE, p))
    for pid in order:
        ptype = "impassable" if pid == IMPASSABLE else provinces[pid]["type"]
        attrs = f'id="prov-{pid}" class="province {ptype}"'
        if pid != IMPASSABLE:
            attrs += f' data-province="{pid}"'
        w(f"    <g {attrs}>")
        for p in by_prov[pid]:
            for d in p["areas"]:
                if id(p) in coast_of:
                    c = coast_of[id(p)]
                    w(f'      <path id="coast-{pid}-{c}" class="area coast-area" data-coast="{c}" fill-rule="evenodd" d="{d}"/>')
                else:
                    w(f'      <path class="area" fill-rule="evenodd" d="{d}"/>')
            for d in p["waterways"]:
                w(f'      <path class="waterway" d="{d}"/>')
        w("    </g>")
    w("  </g>")
    w('  <g id="supply-centers">')
    for pid in sorted(sc_pos):
        x, y = sc_pos[pid]
        w(f'    <circle id="sc-{pid}" class="supply-center" data-province="{pid}" cx="{x}" cy="{y}" r="4"/>')
    w("  </g>")
    w('  <g id="labels">')
    for pid in order:
        x, y = label_pos[pid]
        w(f'    <text id="label-{pid}" class="label" x="{x}" y="{y}">{pid}</text>')
    w("  </g>")
    w("</svg>")
    (MAP_DIR / "standard.svg").write_text("\n".join(out) + "\n")

    # ----- standard.layout.json -------------------------------------------------------------------
    layout = {
        "map": "standard",
        "svg": "standard.svg",
        "viewBox": list(VIEWBOX),
        "source": {"file": "source/Diplomacy.commons.svg", "url": SOURCE_URL,
                   "author": "Martin Asal", "license": "CC BY-SA 3.0", "sha256": digest},
        "provinces": {pid: layout_provinces[pid] for pid in sorted(layout_provinces)},
        "coasts": {k: layout_coasts[k] for k in sorted(layout_coasts)},
    }
    (MAP_DIR / "standard.layout.json").write_text(json.dumps(layout, indent=2) + "\n")

    # ----- standard.graph.svg (auxiliary view D) --------------------------------------------------
    # Geographic layout scaled up so nodes do not overlap; split coasts sit next to their province.
    def node(loc):
        if "/" not in loc:
            x, y = layout_provinces[loc]["unit"]
            return [round(x * GRAPH_SCALE, 1), round(y * GRAPH_SCALE, 1)]
        pid = loc.split("/")[0]
        px, py = node(pid)
        # Direction: from the middle of the province towards the middle of this coast's area.
        others = [coast_center[k] for k in coast_center if k.startswith(pid + "/")]
        mx = sum(c[0] for c in others) / len(others)
        my = sum(c[1] for c in others) / len(others)
        dx, dy = coast_center[loc][0] - mx, coast_center[loc][1] - my
        n = math.hypot(dx, dy) or 1.0
        return [round(px + dx / n * GRAPH_COAST_OFFSET, 1), round(py + dy / n * GRAPH_COAST_OFFSET, 1)]

    gvb = f"0 0 {VIEWBOX[2] * GRAPH_SCALE} {VIEWBOX[3] * GRAPH_SCALE}"

    army = data["adjacency"]["army"]
    fleet = data["adjacency"]["fleet"]
    edges = {}
    for kind, table in (("army", army), ("fleet", fleet)):
        for a, bs in table.items():
            for b in bs:
                key = tuple(sorted((a, b)))
                edges.setdefault(key, set()).add(kind)
    g = []
    w = g.append
    w('<?xml version="1.0" encoding="UTF-8"?>')
    w("<!-- Adjacency graph of the standard map (generated by tools/map/build_map.py from")
    w("     shared/map/standard.json and standard.layout.json). Brown: army only, blue: fleet only,")
    w("     purple: both. Split coasts are separate nodes linked to their province. -->")
    w(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{gvb}" width="{VIEWBOX[2] * GRAPH_SCALE}" height="{VIEWBOX[3] * GRAPH_SCALE}">')
    w("  <style>")
    w("    .edge { stroke-width: 1.5; }")
    w("    .army { stroke: #8b5a2b; } .fleet { stroke: #1f6fd1; } .both { stroke: #7b3fa0; }")
    w("    .coast-link { stroke: #999999; stroke-dasharray: 2 2; }")
    w("    .node { stroke: #000000; } .node.land, .node.coast { fill: #ffffdd; } .node.sea { fill: #99ccff; }")
    w("    .node.split-coast { fill: #ffffff; }")
    w("    .node.sc { stroke-width: 2; }")
    w('    text { font-size: 9px; font-family: "Liberation Sans", Arial, sans-serif; text-anchor: middle; dominant-baseline: central; pointer-events: none; }')
    w("  </style>")
    w('  <rect x="0" y="0" width="100%" height="100%" fill="#ffffff"/>')
    w('  <g id="edges">')
    for (a, b), kinds in sorted(edges.items()):
        cls = "both" if len(kinds) == 2 else next(iter(kinds))
        (x1, y1), (x2, y2) = node(a), node(b)
        w(f'    <line class="edge {cls}" data-from="{a}" data-to="{b}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}"/>')
    for loc in sorted(layout_coasts):
        pid = loc.split("/")[0]
        (x1, y1), (x2, y2) = node(pid), node(loc)
        w(f'    <line class="coast-link" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}"/>')
    w("  </g>")
    w('  <g id="nodes">')
    for pid in sorted(layout_provinces):
        x, y = node(pid)
        sc = " sc" if provinces[pid]["supplyCenter"] else ""
        w(f'    <circle id="node-{pid}" class="node {provinces[pid]["type"]}{sc}" cx="{x}" cy="{y}" r="12"/>')
        w(f'    <text x="{x}" y="{y}">{pid}</text>')
    for loc in sorted(layout_coasts):
        x, y = node(loc)
        w(f'    <circle id="node-{loc.replace("/", "-")}" class="node split-coast" cx="{x}" cy="{y}" r="8"/>')
        w(f'    <text x="{x}" y="{y}">{loc.split("/")[1]}</text>')
    w("  </g>")
    w("</svg>")
    (MAP_DIR / "standard.graph.svg").write_text("\n".join(g) + "\n")

    print(f"source sha256 {digest}")
    print(f"{len(layout_provinces)} provinces, {len(layout_coasts)} coasts, {len(sc_pos)} SCs, {len(edges)} edges")


if __name__ == "__main__":
    sys.exit(main())
