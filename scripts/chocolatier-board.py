#!/usr/bin/env python3
"""Reads the seven board sections of Enemy Chocolatier from the PDF.

Usage:
  uv run --with numpy --with scipy --with pillow --with scikit-image \
      scripts/chocolatier-board.py "Chocolatier Boards.pdf" OUT_DIR

The PDF (Cheapass Games print-and-play, 2019) is not part of the repository.
Page 1 is the Factory, pages 2 to 7 the six outer sections, page 8 the
recipe cards. The boards are pictures: the shapes of the neighborhoods are
read from the picture (everything the street does not reach), the houses
and their prices come from the survey below, made by hand on a 100 dpi
rendering (coordinates in that rendering, trim margin included).

Writes OUT_DIR/board_data.rs (the engine's table, with each section's
neighborhoods on a 5 px grid used to find which ones touch), OUT_DIR/board.json
(shapes, houses and symbols for the web board) and OUT_DIR/control.png
(every section with what was read drawn over it, and one assembled layout
with the neighborhoods that touch linked).
"""

import json
import os
import subprocess
import sys
import tempfile

import numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage as ndi
from skimage import measure

DPI = 100
TRIM = 25  # quarter-inch margin around each 8.25 x 5.5 in section
W, H = 825, 550  # finished section, in pixels at 100 dpi
CELL = 5  # grid cell of the engine table

# Color of a neighborhood: its ingredient (gray houses are not ingredients).
COLORS = {"gray": "Gray", "pink": "Sugar", "brown": "Chocolate", "tan": "Nuts", "red": "Love"}
ABILITIES = {
    "Buy3": "BuyThree",
    "BuyClosed": "BuyClosed",
    "BuyAnywhere": "BuyAnywhere",
    "AllDollar": "AllDollars",
    "AllStar": "AllPoints",
    "FivePerPoint": "FiveDollarsPoint",
    "TenHouses": "TenDollarHouses",
}

# Survey, one line per neighborhood:
#   color: price@x0,y0,x1,y1 ... | $n@x,y (dollar income) *n@x,y (points) Name@x,y (ability)
SURVEY = {
    2: """
gray: 2@45,45,112,130 1@122,45,198,100 1@208,45,280,140 | $1@160,125
red: 1@322,45,410,120 1@322,125,410,195 1@322,205,410,285 3@45,180,120,260 2@125,180,200,240 2@205,180,295,245 4@45,325,135,415 | $1@90,302 *2@290,270 FivePerPoint@195,283
brown: 4@455,45,555,125 3@560,45,645,120 3@650,45,735,120 1@740,45,830,120 1@745,125,830,195 2@655,200,740,295 1@745,200,830,280 | $1@500,162 *1@708,165 BuyClosed@610,162
tan: 1@455,220,535,300 1@540,220,620,295 2@180,340,270,420 4@275,340,375,425 3@380,340,470,425 2@475,340,565,425 | *3@580,318 AllDollar@495,318
gray: 1@730,320,795,400 3@730,410,830,475 4@730,485,830,555 | $2@812,362
brown: 6@40,455,140,550 3@145,465,215,550 2@220,465,285,550 1@290,465,350,550 | $1@382,537 *2@387,490
pink: 2@615,345,695,440 5@455,465,540,560 4@540,465,615,560 3@620,480,695,560 | $3@655,462
""",
    3: """
tan: 7@40,45,140,120 4@40,125,150,215 3@40,220,125,280 | $1@155,197 *2@165,248
gray: 4@180,45,245,135 5@255,85,345,140 5@350,45,420,135 | $3@300,62
pink: 3@455,45,535,130 3@540,45,645,125 2@650,45,735,125 2@740,45,830,125 1@745,130,830,210 5@625,190,720,275 1@730,215,830,280 | $2@700,155 *3@633,153
red: 6@230,180,310,280 2@180,410,245,475 3@250,410,310,490 4@315,430,420,550 2@180,480,245,550 1@255,495,350,550 | $1@215,337 *1@275,317 Buy3@245,377
tan: 1@345,185,435,270 1@440,185,505,265 3@345,275,415,355 3@420,280,500,350 4@510,200,595,345 | $2@520,375 AllStar@420,375
gray: 1@40,320,100,400 2@40,410,140,475 1@45,485,120,550 | $1@127,350
brown: 5@675,345,760,430 4@765,325,830,415 3@730,420,830,480 1@735,490,830,550 | $2@678,338 *1@640,390
gray: 2@455,485,535,550 2@540,485,615,550 3@620,460,695,550 | $1@558,460 *1@485,457
""",
    4: """
red: 1@45,45,115,125 1@120,45,195,130 2@200,45,310,110 2@315,45,420,110 4@45,150,145,245 3@45,250,140,330 3@45,340,145,420 | $1@272,135 *1@205,125 Buy3@365,135
tan: 5@525,45,615,140 3@620,45,690,140 2@695,45,760,140 2@765,45,830,140 | $1@490,123 *1@487,70
brown: 1@180,190,245,280 1@250,190,320,280 2@325,190,395,275 2@400,190,500,290 5@455,290,580,345 3@455,350,550,410 | $2@565,240 BuyAnywhere@537,197
pink: 1@735,180,830,240 2@735,245,830,310 3@735,315,830,385 4@735,385,820,470 6@735,470,830,555 | $2@695,257 *2@685,208
pink: 3@190,320,280,390 3@190,400,280,475 6@115,460,190,550 5@195,480,280,550 | $2@80,535 *2@70,485
tan: 4@320,320,410,385 3@320,390,410,450 3@320,460,390,550 1@395,460,460,550 1@465,455,555,520 | $3@512,538
gray: 1@630,320,695,390 2@595,395,655,480 5@595,490,695,555 | $2@675,445
""",
    5: """
tan: 4@45,45,110,135 5@115,45,205,100 7@210,45,280,135 | $3@160,123
gray: 2@320,45,385,135 1@395,90,470,135 1@485,45,550,135 | $1@435,62
red: 4@595,45,665,140 3@670,45,745,120 3@750,45,830,120 1@740,140,830,215 1@730,220,830,280 | $1@630,182 *1@700,177 FivePerPoint@662,225
gray: 1@45,180,100,270 3@45,280,140,340 4@50,345,140,415 | $2@125,215
pink: 1@180,180,260,265 1@265,180,345,255 1@350,180,425,255 1@430,180,555,250 1@180,270,300,340 3@315,265,470,335 2@475,255,555,335 | *3@252,350 AllDollar@230,397
gray: 5@590,280,690,350 6@745,320,830,415 | $2@713,348
brown: 1@320,370,400,440 4@45,455,115,555 3@120,455,195,555 2@200,455,265,555 1@345,465,420,555 | $2@305,535 *1@305,487
red: 2@515,415,595,490 2@600,415,695,490 3@455,495,605,555 5@610,470,750,555 6@755,455,830,555 | *2@485,435 BuyClosed@492,385
""",
    6: """
gray: 1@45,45,100,125 2@45,135,140,195 1@50,205,135,275 | $1@125,85
tan: 1@180,45,250,125 2@255,45,340,120 3@345,45,420,155 6@180,185,275,280 5@280,205,370,280 | $1@278,185 *1@350,185 BuyAnywhere@260,145
pink: 4@455,45,525,155 2@530,45,620,115 2@625,45,695,115 6@560,215,695,280 | $1@655,153 *1@580,147 TenHouses@622,195
brown: 4@730,45,830,145 3@735,150,830,215 2@735,220,830,295 8@765,305,830,395 | $1@730,395 *3@725,350
red: 4@405,195,520,270 3@320,320,400,415 1@405,350,495,415 2@500,320,575,415 2@580,320,660,415 | $2@448,330 AllStar@465,290
pink: 2@45,320,120,395 4@125,320,200,400 5@205,320,280,400 6@45,470,140,555 | $2@125,457 *2@72,440
tan: 1@180,460,260,540 2@260,460,330,540 4@335,460,410,540 5@415,460,490,540 | $1@520,537 *2@525,485
gray: 2@595,455,665,545 2@670,490,750,555 4@755,455,830,555 | $2@710,475
""",
    7: """
brown: 4@45,45,120,150 3@125,45,200,130 1@205,45,280,140 5@45,195,165,280 6@175,185,280,280 | $1@243,160 *1@172,155 Buy3@95,180
gray: 3@320,45,385,135 2@395,45,470,100 3@480,45,550,135 | $2@435,125
red: 4@665,45,750,130 3@755,45,830,120 5@625,140,720,240 2@725,135,830,200 1@740,205,830,275 | $1@630,62 *3@622,115
pink: 6@320,180,410,270 4@420,185,525,260 3@530,185,605,275 2@450,270,525,355 1@530,280,605,355 | $2@408,268 BuyClosed@384,308
tan: 1@45,320,120,395 3@45,400,135,485 3@150,410,275,465 1@45,495,140,555 4@150,470,280,555 | $1@237,380 *1@160,385 FivePerPoint@185,337
gray: 6@360,370,415,470 5@320,480,415,555 | $2@338,407
gray: 7@645,290,720,375 5@760,305,830,390 | $1@738,340 TenHouses@730,402
brown: 5@455,455,525,555 4@530,450,605,550 3@610,460,680,555 2@690,460,755,555 2@760,455,830,555 | $2@570,417 *2@492,427
""",
}

# Which neighborhoods touch: across a street at most MAX_GAP wide, facing
# each other along at least MIN_FACE (a corner across an intersection does
# not count). Mirrored by `layout.rs`.
MAX_GAP = 50 // CELL
MIN_FACE = 15 // CELL


def parse_point(text):
    return tuple(int(v) for v in text.split(","))


def parse_survey():
    boards = {}
    for page, text in SURVEY.items():
        hoods = []
        for line in text.strip().splitlines():
            color, rest = line.split(":", 1)
            houses_text, features_text = rest.split("|")
            houses = []
            for item in houses_text.split():
                price, box = item.split("@")
                houses.append((int(price), parse_point(box)))
            hood = {"color": COLORS[color.strip()], "houses": houses, "income": 0,
                    "points": 0, "ability": None, "symbols": []}
            for item in features_text.split():
                name, at = item.split("@")
                at = parse_point(at)
                if name[0] == "$":
                    hood["income"] = int(name[1:])
                    hood["symbols"].append(("dollars", int(name[1:]), at))
                elif name[0] == "*":
                    hood["points"] = int(name[1:])
                    hood["symbols"].append(("points", int(name[1:]), at))
                else:
                    hood["ability"] = ABILITIES[name]
                    hood["symbols"].append(("ability", ABILITIES[name], at))
            hoods.append(hood)
        boards[page] = hoods
    return boards


def render(pdf, tmp):
    subprocess.run(["pdftoppm", "-r", str(DPI), "-f", "1", "-l", "7", "-png", pdf,
                    os.path.join(tmp, "p")], check=True)
    pages = {}
    for page in range(1, 8):
        path = os.path.join(tmp, f"p-{page}.png")
        pages[page] = Image.open(path).convert("RGB")
    return pages


def blocks(image):
    """Labels of the blocks: everything the street, flooded from a corner
    through pixels that are not outline-dark, does not reach."""
    rgb = np.asarray(image).astype(int)
    dark = rgb.mean(axis=2) < 70
    lab, _ = ndi.label(~dark)
    street = lab == lab[2, 2]
    labels, count = ndi.label(~street)
    sizes = ndi.sum(~street, labels, range(1, count + 1))
    keep = [i + 1 for i, s in enumerate(sizes) if s > 2000]
    return labels, keep


def assign(page, hoods, labels, keep):
    """The block of each surveyed neighborhood; every house and symbol must
    lie in it, and every block must get exactly one neighborhood."""
    owners = []
    for k, hood in enumerate(hoods):
        found = set()
        for _, (x0, y0, x1, y1) in hood["houses"]:
            found.add(int(labels[(y0 + y1) // 2, (x0 + x1) // 2]))
        for _, _, (x, y) in hood["symbols"]:
            found.add(int(labels[y, x]))
        if len(found) != 1 or 0 in found:
            sys.exit(f"page {page}, neighborhood {k}: spans blocks {sorted(found)}")
        owners.append(found.pop())
    if sorted(owners) != sorted(keep):
        sys.exit(f"page {page}: blocks {sorted(keep)} but neighborhoods in {sorted(owners)}")
    return owners


def grid(labels, owners):
    """The section on the engine's grid: '.' street, 'a'.. neighborhoods in
    survey order (a cell belongs to the block covering most of it)."""
    sub = labels[TRIM:TRIM + H, TRIM:TRIM + W]
    rows = []
    for gy in range(H // CELL):
        row = []
        for gx in range(W // CELL):
            cell = sub[gy * CELL:(gy + 1) * CELL, gx * CELL:(gx + 1) * CELL].ravel()
            best, best_n = ".", CELL * CELL // 2
            for k, owner in enumerate(owners):
                n = int((cell == owner).sum())
                if n > best_n:
                    best, best_n = chr(ord("a") + k), n
            row.append(best)
        rows.append("".join(row))
    return rows


def outline(labels, owner):
    mask = labels[TRIM:TRIM + H, TRIM:TRIM + W] == owner
    padded = np.pad(mask, 1).astype(float)
    contour = max(measure.find_contours(padded, 0.5), key=len)
    poly = measure.approximate_polygon(contour, tolerance=1.2)
    return [[round(float(x) - 1, 1), round(float(y) - 1, 1)] for y, x in poly[:-1]]


def local(point):
    return [point[0] - TRIM, point[1] - TRIM]


# Assembly used by the control picture: the rules' square, Factory in the
# middle of the center column, side sections standing upright.
def placed(page_grids):
    """Layout with the outer sections in page order, unturned center pieces
    and side pieces turned a quarter clockwise; same convention as the
    engine (`layout.rs`)."""
    slots = layout_slots()
    return [(p, slot) for p, slot in zip([2, 3, 4, 5, 6, 7], slots)]


def layout_slots():
    # (x, y, quarter turns clockwise) of each outer slot, in grid cells.
    w, h = W // CELL, H // CELL
    return [(0, 0, 1), (0, w, 1), (h, 0, 0), (h, 2 * h, 0), (h + w, 0, 1), (h + w, w, 1)]


def rotate(rows, quarters):
    a = np.array([list(r) for r in rows])
    return np.rot90(a, -quarters)


def assemble(page_grids, factory_rows):
    w, h = W // CELL, H // CELL
    big = np.full((2 * w, 2 * h + w), ".", dtype=object)
    names = {}
    for page, (x, y, q) in placed(page_grids):
        g = rotate(page_grids[page], q)
        for (gy, gx), ch in np.ndenumerate(g):
            if ch != ".":
                big[y + gy, x + gx] = f"{page}{ch}"
    f = np.array([list(r) for r in factory_rows])
    for (gy, gx), ch in np.ndenumerate(f):
        if ch != ".":
            big[h + gy, h + gx] = "F"
    return big


def touching(big):
    counts = {}
    for grid2 in (big, big.T):
        for line in grid2:
            last, gap = None, 0
            for ch in line:
                if ch == ".":
                    gap += 1
                    continue
                if last is not None and last != ch and 0 < gap <= MAX_GAP:
                    key = tuple(sorted((last, ch)))
                    counts[key] = counts.get(key, 0) + 1
                last, gap = ch, 0
    return sorted(k for k, n in counts.items() if n >= MIN_FACE)


def rust(boards, page_grids, factory_rows):
    out = ["// Generated by scripts/chocolatier-board.py from the print-and-play PDF.",
           "// Do not edit by hand.", "",
           "use crate::{Ability, Ingredient};", "",
           "pub struct HoodData {",
           "    pub color: Option<Ingredient>,",
           "    pub prices: &'static [u8],",
           "    pub dollars: u8,",
           "    pub points: u8,",
           "    pub ability: Option<Ability>,",
           "}", "",
           "pub struct SectionData {",
           "    pub hoods: &'static [HoodData],",
           "    /// '.' street, 'a'.. the hoods, one row per line, cells of "
           f"{CELL} px at {DPI} dpi.",
           "    pub grid: &'static str,",
           "}", "",
           f"pub const GRID_W: usize = {W // CELL};",
           f"pub const GRID_H: usize = {H // CELL};",
           f"pub const MAX_GAP: usize = {MAX_GAP};",
           f"pub const MIN_FACE: usize = {MIN_FACE};", "",
           "pub const FACTORY_GRID: &str = \"\\",
           *[r + "\\n\\" for r in factory_rows], "\";", "",
           "pub const SECTIONS: [SectionData; 6] = ["]
    for page in range(2, 8):
        out.append("    SectionData {")
        out.append("        hoods: &[")
        for hood in boards[page]:
            color = "None" if hood["color"] == "Gray" else f"Some(Ingredient::{hood['color']})"
            ability = f"Some(Ability::{hood['ability']})" if hood["ability"] else "None"
            prices = ", ".join(str(p) for p, _ in hood["houses"])
            out.append(f"            HoodData {{ color: {color}, prices: &[{prices}], "
                       f"dollars: {hood['income']}, points: {hood['points']}, ability: {ability} }},")
        out.append("        ],")
        out.append("        grid: \"\\")
        out.extend(r + "\\n\\" for r in page_grids[page])
        out.append("\",")
        out.append("    },")
    out.append("];")
    return "\n".join(out) + "\n"


def web(boards, outlines, factory_outline):
    sections = []
    for page in range(2, 8):
        hoods = []
        for hood, shape in zip(boards[page], outlines[page]):
            hoods.append({
                "color": hood["color"],
                "shape": shape,
                "houses": [[x0 - TRIM, y0 - TRIM, x1 - x0, y1 - y0] for _, (x0, y0, x1, y1)
                           in hood["houses"]],
                "symbols": [{"kind": kind, "value": value, "at": local(at)}
                            for kind, value, at in hood["symbols"]],
            })
        sections.append({"hoods": hoods})
    return {"width": W, "height": H, "factory": factory_outline, "sections": sections}


def control(pages, boards, owners_by_page, page_grids, factory_rows, path):
    tiles = []
    for page in range(2, 8):
        im = pages[page].copy()
        d = ImageDraw.Draw(im)
        for hood in boards[page]:
            for price, (x0, y0, x1, y1) in hood["houses"]:
                d.rectangle((x0, y0, x1, y1), outline=(0, 0, 255), width=2)
                d.text((x0 + 4, y0 + 2), str(price), fill=(0, 0, 255))
            for kind, value, (x, y) in hood["symbols"]:
                d.ellipse((x - 5, y - 5, x + 5, y + 5), fill=(255, 0, 255))
                d.text((x + 7, y - 6), f"{kind[0]}{value}", fill=(255, 0, 255))
        tiles.append(im)
    big = assemble(page_grids, factory_rows)
    links = touching(big)
    centers = {}
    for name in set(big.ravel()) - {"."}:
        ys, xs = np.nonzero(big == name)
        centers[name] = (xs.mean() * CELL, ys.mean() * CELL)
    palette = {"Gray": (170, 170, 170), "Sugar": (240, 160, 190), "Chocolate": (140, 90, 50),
               "Nuts": (230, 190, 120), "Love": (210, 70, 70)}
    lay = Image.new("RGB", (big.shape[1] * CELL, big.shape[0] * CELL), (235, 215, 190))
    ld = ImageDraw.Draw(lay)
    for (gy, gx), name in np.ndenumerate(big):
        if name == ".":
            continue
        color = (90, 60, 40) if name == "F" else palette[boards[int(name[0])][ord(name[1]) - 97]["color"]]
        ld.rectangle((gx * CELL, gy * CELL, gx * CELL + CELL - 1, gy * CELL + CELL - 1), fill=color)
    for a, b in links:
        ld.line((centers[a], centers[b]), fill=(0, 0, 255), width=3)
    for name, (x, y) in centers.items():
        ld.text((x - 6, y - 6), name, fill=(0, 0, 0))
    tw, th = tiles[0].size
    sheet = Image.new("RGB", (max(2 * tw, lay.width), 3 * th + lay.height), (255, 255, 255))
    for k, im in enumerate(tiles):
        sheet.paste(im, ((k % 2) * tw, (k // 2) * th))
    sheet.paste(lay, (0, 3 * th))
    sheet.save(path)
    return links


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    pdf, out_dir = sys.argv[1], sys.argv[2]
    boards = parse_survey()
    with tempfile.TemporaryDirectory() as tmp:
        pages = render(pdf, tmp)
    page_grids, outlines, owners_by_page = {}, {}, {}
    for page in range(2, 8):
        labels, keep = blocks(pages[page])
        owners = assign(page, boards[page], labels, keep)
        owners_by_page[page] = owners
        page_grids[page] = grid(labels, owners)
        outlines[page] = [outline(labels, o) for o in owners]
    labels, keep = blocks(pages[1])
    if len(keep) != 1:
        sys.exit(f"factory: {len(keep)} blocks")
    factory_rows = [r.replace("a", "F") for r in grid(labels, keep)]
    factory_outline = outline(labels, keep[0])
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "board_data.rs"), "w") as f:
        f.write(rust(boards, page_grids, factory_rows))
    with open(os.path.join(out_dir, "board.json"), "w") as f:
        json.dump(web(boards, outlines, factory_outline), f, separators=(",", ":"))
    links = control(pages, boards, owners_by_page, page_grids, factory_rows,
                    os.path.join(out_dir, "control.png"))
    counts = {}
    for page in range(2, 8):
        for hood in boards[page]:
            counts[hood["color"]] = counts.get(hood["color"], 0) + len(hood["houses"])
    print("houses per color:", counts)
    print("links in the control layout:", len(links))


if __name__ == "__main__":
    main()
