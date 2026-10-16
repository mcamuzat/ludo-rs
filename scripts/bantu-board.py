#!/usr/bin/env python3
"""Extracts the Bantu board graph from the vector pages of the rules PDF.

Usage: bantu-board.py RULES.pdf OUT_DIR

The PDF (Summit Games rules put online by François Haffner) is not part of
the repository. Pages 4 to 7 hold the board in four quarters, drawn as
vectors: circles, links, rows and start lines are read from their colours.
Writes OUT_DIR/board_data.rs (the engine's table) and OUT_DIR/board.svg (a
drawing of the graph, to check it against the scan).
"""

import json
import math
import os
import re
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET

NS = "{http://www.w3.org/2000/svg}"
PAGES = (4, 5, 6, 7)

CIRCLE = (60, 80, 100)
ROW = (50, 0, 0)
START = (100, 0, 0)
ARROW = (20, 40, 100)
DISC = (60, 20, 0)


def numbers(d):
    vals = [float(x) for x in re.findall(r"-?\d+(?:\.\d+)?", d)]
    return list(zip(vals[0::2], vals[1::2]))


def colour(fill):
    m = re.match(r"rgb\(([\d.]+)%, ([\d.]+)%, ([\d.]+)%\)", fill or "")
    return tuple(round(float(v)) for v in m.groups()) if m else None


def corners(points):
    """Distinct vertices of a straight-edged path (the closing point dropped)."""
    out = []
    for p in points:
        if not any(math.dist(p, q) < 0.01 for q in out):
            out.append(p)
    return out


def page_shapes(pdf, page, tmp):
    svg = os.path.join(tmp, f"p{page}.svg")
    subprocess.run(["pdftocairo", "-svg", "-f", str(page), "-l", str(page), pdf, svg], check=True)
    shapes = []
    disc = None
    for el in ET.parse(svg).getroot().iter(NS + "path"):
        c = colour(el.get("fill"))
        if c is None:
            continue
        d = el.get("d", "")
        pts = numbers(d)
        curved = "C" in d
        if c == DISC and curved:
            xs = [x for x, _ in pts]
            ys = [y for _, y in pts]
            disc = ((min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2)
        shapes.append((c, curved, pts))
    # Same scale on every page: only the offset differs. Centre on the disc.
    return [(c, curved, [(x - disc[0], y - disc[1]) for x, y in pts]) for c, curved, pts in shapes]


def extract(pdf):
    circles, bars, rows, starts, arrows, triangles = [], [], [], [], [], []
    with tempfile.TemporaryDirectory() as tmp:
        for page in PAGES:
            for c, curved, pts in page_shapes(pdf, page, tmp):
                if c == CIRCLE and curved:
                    xs = [x for x, _ in pts]
                    ys = [y for _, y in pts]
                    centre = ((min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2)
                    r = (max(xs) - min(xs)) / 2
                    if r < 40 and not any(math.dist(centre, o[0]) < 5 for o in circles):
                        circles.append((centre, r))
                elif c == CIRCLE:
                    v = corners(pts)
                    if len(v) == 4:
                        add_unique(bars, bar_axis(v))
                    elif len(v) == 3:
                        add_unique(triangles, tuple(v), key=centroid)
                elif c == ROW and not curved:
                    add_unique(rows, tuple(corners(pts)), key=centroid)
                elif c == START and not curved:
                    add_unique(starts, tuple(corners(pts)), key=centroid)
                elif c == ARROW and not curved:
                    v = corners(pts)
                    if 6 <= len(v) <= 8:
                        add_unique(arrows, tuple(v), key=centroid)
    return circles, bars, rows, starts, arrows, triangles


def centroid(v):
    return (sum(x for x, _ in v) / len(v), sum(y for _, y in v) / len(v))


def add_unique(items, item, key=None):
    k = key(item) if key else (item[0], item[1])
    for other in items:
        ok = key(other) if key else (other[0], other[1])
        if isinstance(k[0], tuple):
            same = all(math.dist(a, b) < 5 for a, b in zip(sorted(k), sorted(ok)))
        else:
            same = math.dist(k, ok) < 5
        if same:
            return
    items.append(item)


def bar_axis(v):
    """The two ends of a thin rectangle: the middles of its short sides."""
    sides = [(v[i], v[(i + 1) % 4]) for i in range(4)]
    short = sorted(sides, key=lambda s: math.dist(*s))[:2]
    ends = [((a[0] + b[0]) / 2, (a[1] + b[1]) / 2) for a, b in short]
    return tuple(sorted(ends))


def inside(p, poly):
    x, y = p
    n = len(poly)
    hit = False
    for i in range(n):
        (x1, y1), (x2, y2) = poly[i], poly[(i + 1) % n]
        if (y1 > y) != (y2 > y) and x < x1 + (y - y1) * (x2 - x1) / (y2 - y1):
            hit = not hit
    return hit


def near_rect(p, poly, margin=4):
    xs = [x for x, _ in poly]
    ys = [y for _, y in poly]
    return min(xs) - margin <= p[0] <= max(xs) + margin and min(ys) - margin <= p[1] <= max(ys) + margin


def grown(poly, factor=1.08):
    c = centroid(poly)
    return [(c[0] + (x - c[0]) * factor, c[1] + (y - c[1]) * factor) for x, y in poly]


def along(poly):
    """Unit vector of a band's long axis."""
    sides = [(poly[i], poly[(i + 1) % len(poly)]) for i in range(len(poly))]
    (a, b) = max(sides, key=lambda s: math.dist(*s))
    n = math.dist(a, b)
    return ((b[0] - a[0]) / n, (b[1] - a[1]) / n)


def build(raw):
    circles, bars, rows, starts, arrows = raw
    centres = [c for c, _ in circles]
    radius = sum(r for _, r in circles) / len(circles)

    def circle_at(p):
        best = min(range(len(centres)), key=lambda i: math.dist(p, centres[i]))
        return best if math.dist(p, centres[best]) <= radius + 3 else None

    groups = []  # (kind, cells in order)
    owner = {}
    for kind, polys, test in (("start", starts, near_rect), ("row", rows, lambda p, q: inside(p, grown(q)))):
        for poly in polys:
            members = [i for i, c in enumerate(centres) if test(c, poly)]
            axis = along(poly)
            members.sort(key=lambda i: centres[i][0] * axis[0] + centres[i][1] * axis[1])
            for i in members:
                if i in owner:
                    print(f"warning: circle {i} in two groups", file=sys.stderr)
                owner[i] = len(groups)
            groups.append((kind, members))
    links, loose = [], []
    for a, b in bars:
        ca, cb = circle_at(a), circle_at(b)
        if ca is None or cb is None:
            loose.append((a, b))
            continue
        if ca == cb or (owner.get(ca) is not None and owner.get(ca) == owner.get(cb)):
            continue
        if (ca, cb) not in links and (cb, ca) not in links:
            links.append((ca, cb))
    return centres, radius, groups, links, loose


def debug_svg(centres, radius, groups, links, loose, arrows, path):
    lo = min(min(x, y) for x, y in centres) - 60
    hi = max(max(x, y) for x, y in centres) + 60
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{lo} {lo} {hi - lo} {hi - lo}" width="1400" height="1400">',
           f'<rect x="{lo}" y="{lo}" width="{hi - lo}" height="{hi - lo}" fill="#cfc"/>']
    for kind, members in groups:
        pts = " ".join(f"{centres[i][0]},{centres[i][1]}" for i in members)
        colour = "#e00" if kind == "start" else "#800"
        out.append(f'<polyline points="{pts}" stroke="{colour}" stroke-width="{radius * 1.4}" fill="none" stroke-linecap="round"/>')
    out.append('<defs><marker id="h" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="5" markerHeight="5" orient="auto"><path d="M0,0 L10,5 L0,10 z" fill="#06c"/></marker></defs>')
    for a, b in links:
        (x1, y1), (x2, y2) = centres[a], centres[b]
        k = 1 - radius * 0.9 / math.dist(centres[a], centres[b])
        out.append(f'<line x1="{x1}" y1="{y1}" x2="{x1 + (x2 - x1) * k}" y2="{y1 + (y2 - y1) * k}" stroke="#39f" stroke-width="5" marker-end="url(#h)"/>')
    for a, b in loose:
        out.append(f'<line x1="{a[0]}" y1="{a[1]}" x2="{b[0]}" y2="{b[1]}" stroke="#f0f" stroke-width="8"/>')
    for v in arrows:
        pts = " ".join(f"{x},{y}" for x, y in v)
        out.append(f'<polygon points="{pts}" fill="#03f"/>')
    for i, (x, y) in enumerate(centres):
        out.append(f'<circle cx="{x}" cy="{y}" r="{radius * 0.8}" fill="#9cf" fill-opacity="0.85"/>')
        out.append(f'<text x="{x}" y="{y + 6}" font-size="18" text-anchor="middle">{i}</text>')
    out.append("</svg>")
    with open(path, "w") as f:
        f.write("\n".join(out))


def angle(p):
    """Clockwise angle on screen (y down), 0 to the right, in degrees."""
    return math.degrees(math.atan2(p[1], p[0])) % 360


# Clockwise from each start line: the quarter it opens and where it leads.
# Line 4 (left, 180°) -> line 3 (top, 270°) -> line 2 (right, 0°) -> line 1
# (bottom, 90°) -> finish (bottom-left quarter).
LINE_AT = {180: 4, 270: 3, 0: 2, 90: 1}
QUARTER = {4: (180, 270), 3: (270, 360), 2: (0, 90), 1: (90, 180)}
NEXT = {4: 3, 3: 2, 2: 1, 1: None}

# Links the distance rule cannot orient (both ends equally far from the next
# line), as extracted cell ids -> direction. 37-52 joins two parallel rows of
# the top-right quarter: like 18->22, 94->98 and 63->78 in the other quarters,
# it leads from the row reached first (35-38, which feeds 50-52 through
# 35->50) to the next one.
BY_HAND = {frozenset((37, 52)): (37, 52)}


def seg_dist(p, a, b):
    ax, ay = a
    bx, by = b
    dx, dy = bx - ax, by - ay
    t = max(0, min(1, ((p[0] - ax) * dx + (p[1] - ay) * dy) / (dx * dx + dy * dy)))
    return math.dist(p, (ax + t * dx, ay + t * dy))


def orient(centres, radius, groups, links, arrows):
    lines = {}
    for g, (kind, members) in enumerate(groups):
        if kind == "start":
            a = angle(centroid([centres[i] for i in members]))
            n = LINE_AT[min(LINE_AT, key=lambda k: min(abs(a - k), 360 - abs(a - k)))]
            lines[n] = members
    line_of = {i: n for n, members in lines.items() for i in members}
    # Arrows: one in a cell marks a finish cell; one on a link makes it one-way.
    arrow_cells, forced = set(), {}
    for v in arrows:
        c = centroid(v)
        # The PDF draws each arrow from its tip, between its two barbs.
        tip = v[0]
        assert abs(math.dist(tip, v[1]) - math.dist(tip, v[-1])) < 1.5, f"arrow not drawn from its tip: {v}"
        direction = (tip[0] - c[0], tip[1] - c[1])
        cell = min(range(len(centres)), key=lambda i: math.dist(c, centres[i]))
        link = min(links, key=lambda l: seg_dist(c, centres[l[0]], centres[l[1]]))
        if seg_dist(c, centres[link[0]], centres[link[1]]) < math.dist(c, centres[cell]) - radius * 0.5:
            a, b = link
            ab = (centres[b][0] - centres[a][0], centres[b][1] - centres[a][1])
            forced[frozenset(link)] = (a, b) if ab[0] * direction[0] + ab[1] * direction[1] > 0 else (b, a)
        else:
            arrow_cells.add(cell)

    def quarter(i):
        if i in line_of:
            return None
        a = angle(centres[i])
        for n, (lo, hi) in QUARTER.items():
            if lo <= a < hi:
                return n
        raise AssertionError

    adjacency = {i: set() for i in range(len(centres))}
    for a, b in links:
        adjacency[a].add(b)
        adjacency[b].add(a)
    for kind, members in groups:
        for a, b in zip(members, members[1:]):
            adjacency[a].add(b)
            adjacency[b].add(a)
    directed, problems = [], []
    for a, b in links:
        la, lb = line_of.get(a), line_of.get(b)
        if la is not None and lb is not None:
            problems.append(f"link between two start lines: {a}-{b}")
            continue
        if la is not None or lb is not None:
            line, other = (la, b) if la is not None else (lb, a)
            if quarter(other) == line:  # leaving the line into its quarter
                edge = (a, b) if la is not None else (b, a)
            elif NEXT.get(quarter(other)) == line:  # arriving on the line
                edge = (b, a) if la is not None else (a, b)
            else:
                problems.append(f"link {a}-{b}: line {line} next to quarter {quarter(other)}")
                continue
        else:
            q = quarter(a)
            if quarter(b) != q:
                problems.append(f"link {a}-{b} across quarters {q}/{quarter(b)}")
                continue
            target = set(lines[NEXT[q]]) if NEXT[q] else arrow_cells
            dist = bfs(adjacency, target, lambda i: i not in line_of or i in target or False, q, quarter)
            if frozenset((a, b)) in BY_HAND:
                edge = BY_HAND[frozenset((a, b))]
            elif dist.get(a) == dist.get(b):
                problems.append(f"link {a}-{b}: same distance {dist.get(a)}")
                continue
            else:
                edge = (a, b) if dist[a] > dist[b] else (b, a)
        f = forced.get(frozenset((a, b)))
        if f is not None and f != edge:
            problems.append(f"link {a}-{b}: arrow says {f}, distance says {edge}")
            edge = f
        directed.append(edge)
    return lines, arrow_cells, directed, forced, problems


def bfs(adjacency, target, _allowed, q, quarter):
    """Steps to `target` inside quarter `q` (target cells included)."""
    dist = {i: 0 for i in target}
    todo = list(target)
    while todo:
        nxt = []
        for i in todo:
            for j in adjacency[i]:
                if j not in dist and quarter(j) == q:
                    dist[j] = dist[i] + 1
                    nxt.append(j)
        todo = nxt
    return dist


# Half the side of the board's background square, in PDF units.
HALF = 559.0


def to_board(p):
    """PDF units around the disc -> the 0..1000 square of the whole board."""
    return (round(500 + p[0] * 500 / HALF), round(500 + p[1] * 500 / HALF))


def rust(centres, groups, lines, arrow_cells, directed):
    rows = [m for kind, m in groups if kind == "row"]
    out = [
        "// Généré par scripts/bantu-board.py depuis les pages 4 à 7 de la règle de",
        "// Summit Games (mise en ligne par François Haffner). Ne pas modifier à la main.",
        "",
        "use super::{Cell, CellDef};",
        "",
        f"pub static CELLS: [CellDef; {len(centres)}] = [",
    ]
    for i, c in enumerate(centres):
        x, y = to_board(c)
        out.append(f"    CellDef {{ x: {x}, y: {y}, arrow: {'true' if i in arrow_cells else 'false'} }},")
    out.append("];")
    out.append("")
    out.append("/// `START_LINES[n - 1]`: the start line of piece n, from the centre outwards.")
    out.append("pub static START_LINES: [[Cell; 4]; 4] = [")
    for n in (1, 2, 3, 4):
        cells = sorted(lines[n], key=lambda i: math.hypot(*centres[i]))
        out.append(f"    [{', '.join(map(str, cells))}],")
    out.append("];")
    out.append("")
    out.append(f"pub static ROWS: [&[Cell]; {len(rows)}] = [")
    for m in rows:
        out.append(f"    &[{', '.join(map(str, m))}],")
    out.append("];")
    out.append("")
    out.append("/// Light-blue links, oriented the way pieces move.")
    out.append(f"pub static LINKS: [(Cell, Cell); {len(directed)}] = [")
    for a, b in sorted(directed):
        out.append(f"    ({a}, {b}),")
    out.append("];")
    return "\n".join(out) + "\n"


def board_svg(centres, radius, groups, directed, arrow_cells):
    """The extracted graph in the board's 0..1000 square, to hold against the scan."""
    r = radius * 500 / HALF
    pos = [to_board(c) for c in centres]
    out = ['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 1000">',
           '<defs><marker id="h" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="4" markerHeight="4" orient="auto">'
           '<path d="M0,0 L10,5 L0,10 z" fill="#036"/></marker></defs>']
    for kind, members in groups:
        pts = " ".join(f"{pos[i][0]},{pos[i][1]}" for i in members)
        colour = "#e00" if kind == "start" else "#800"
        out.append(f'<polyline points="{pts}" stroke="{colour}" stroke-width="{r * 1.6:.1f}" fill="none" stroke-linecap="round" stroke-opacity="0.75"/>')
    for a, b in directed:
        (x1, y1), (x2, y2) = pos[a], pos[b]
        k = 1 - r / math.dist(pos[a], pos[b])
        out.append(f'<line x1="{x1}" y1="{y1}" x2="{x1 + (x2 - x1) * k:.1f}" y2="{y1 + (y2 - y1) * k:.1f}" stroke="#036" stroke-width="3" marker-end="url(#h)"/>')
    for i, (x, y) in enumerate(pos):
        fill = "#36f" if i in arrow_cells else "#9cf"
        out.append(f'<circle cx="{x}" cy="{y}" r="{r:.1f}" fill="{fill}" fill-opacity="0.7" stroke="#036"/>')
        out.append(f'<text x="{x}" y="{y + 5}" font-size="13" font-family="sans-serif" text-anchor="middle">{i}</text>')
    out.append("</svg>")
    return "\n".join(out)


def main():
    pdf, out = sys.argv[1], sys.argv[2]
    circles, bars, rows, starts, arrows, _ = extract(pdf)
    centres, radius, groups, links, loose = build((circles, bars, rows, starts, arrows))
    lines, arrow_cells, directed, forced, problems = orient(centres, radius, groups, links, arrows)
    print(json.dumps({"cells": len(centres), "groups": len(groups), "links": len(directed),
                      "one_way": len(forced), "arrow_cells": sorted(arrow_cells),
                      "lines": {n: lines[n] for n in sorted(lines)}}))
    for p in problems:
        print("problem:", p)
    if problems or loose:
        sys.exit(1)
    os.makedirs(out, exist_ok=True)
    debug_svg(centres, radius, groups, directed, loose, arrows, os.path.join(out, "debug.svg"))
    with open(os.path.join(out, "board.svg"), "w") as f:
        f.write(board_svg(centres, radius, groups, directed, arrow_cells))
    with open(os.path.join(out, "board_data.rs"), "w") as f:
        f.write(rust(centres, groups, lines, arrow_cells, directed))


if __name__ == "__main__":
    main()
