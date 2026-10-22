#!/usr/bin/env python3
"""Reads the board of Das glorreiche Manöver from the rules PDF.

Usage: manoever-board.py RULES.pdf OUT_DIR

The PDF (jeuxsoc translation put online by l'Escale à jeux) is not part of
the repository. Pages 5 and 6 hold the board in two halves; the numbers of
the squares (1 to 84) and of the barracks places (I to XII) are positioned
text over a background image. Page 6 goes above page 5: square 42 leaves the
top of page 5 and 43 enters the bottom of page 6.

Writes OUT_DIR/board_data.rs (the engine's table) and OUT_DIR/control.png
(both pages assembled, with what was read drawn over them; needs Pillow).
"""

import os
import re
import subprocess
import sys
import tempfile

PAGE_W = 841.89
PAGE_H = 595.275
# Page 6 on top, page 5 below.
OFFSET = {6: 0.0, 5: PAGE_H}
ROMAN = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII"]
WORD = re.compile(
    r'<word xMin="([\d.]+)" yMin="([\d.]+)" xMax="([\d.]+)" yMax="([\d.]+)">([^<]+)</word>'
)


def read_words(pdf, page):
    out = subprocess.run(
        ["pdftotext", "-f", str(page), "-l", str(page), "-bbox", pdf, "-"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    for x0, y0, x1, y1, text in WORD.findall(out):
        cx = (float(x0) + float(x1)) / 2
        cy = (float(y0) + float(y1)) / 2 + OFFSET[page]
        yield text.strip(), (round(cx), round(cy))


def survey(pdf):
    squares, places = {}, {}
    for page in (5, 6):
        for text, point in read_words(pdf, page):
            if text.isdigit():
                table, key = squares, int(text)
            elif text in ROMAN:
                table, key = places, ROMAN.index(text) + 1
            else:
                sys.exit(f"unexpected word {text!r} on page {page}")
            if key in table:
                sys.exit(f"{text} found twice")
            table[key] = point
    missing = [n for n in range(1, 85) if n not in squares]
    missing += [ROMAN[n - 1] for n in range(1, 13) if n not in places]
    if missing or len(squares) != 84:
        sys.exit(f"missing or extra words: {missing}, {len(squares)} squares")
    return [squares[n] for n in range(1, 85)], [places[n] for n in range(1, 13)]


def rust(squares, places):
    lines = [
        "// Généré par scripts/manoever-board.py depuis les pages 5 et 6 de la",
        "// traduction jeuxsoc (mise en ligne par l'Escale à jeux). Ne pas modifier",
        "// à la main.",
        "",
        "use super::Point;",
        "",
        "/// Square `n` at index `n - 1`.",
        "pub static SQUARES: [Point; 84] = [",
    ]
    lines += [f"    Point {{ x: {x}, y: {y} }}," for x, y in squares]
    lines += ["];", "", "/// Place `n` (I to XII) at index `n - 1`.", "pub static BARRACKS: [Point; 12] = ["]
    lines += [f"    Point {{ x: {x}, y: {y} }}," for x, y in places]
    lines += ["];", ""]
    return "\n".join(lines)


def control(pdf, squares, places, path):
    from PIL import Image, ImageDraw

    with tempfile.TemporaryDirectory() as tmp:
        sheet = Image.new("RGB", (round(PAGE_W), round(2 * PAGE_H)), "white")
        for page in (5, 6):
            base = os.path.join(tmp, f"p{page}")
            subprocess.run(
                ["pdftocairo", "-png", "-r", "72", "-singlefile", "-f", str(page), "-l", str(page), pdf, base],
                check=True,
            )
            sheet.paste(Image.open(base + ".png").convert("RGB"), (0, round(OFFSET[page])))
    draw = ImageDraw.Draw(sheet)
    draw.line(squares, fill=(0, 0, 255), width=2)
    discs = {12: "black", 20: "black", 47: "black", 65: "black",
             8: "white", 24: "white", 31: "white", 36: "white", 60: "white", 74: "white",
             25: "grey", 40: "grey", 55: "grey"}
    for n, (x, y) in enumerate(squares, start=1):
        draw.ellipse((x - 30, y - 30, x + 30, y + 30), outline=(0, 0, 255), width=2)
        if n in discs:
            draw.ellipse((x - 12, y - 12, x + 12, y + 12), fill=discs[n], outline="blue")
        draw.text((x + 14, y + 14), str(n), fill=(0, 0, 255))
    for n, (x, y) in enumerate(places, start=1):
        draw.rectangle((x - 30, y - 30, x + 30, y + 30), outline=(255, 0, 255), width=2)
        draw.text((x + 14, y + 14), ROMAN[n - 1], fill=(255, 0, 255))
    sheet.save(path)


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    pdf, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    squares, places = survey(pdf)
    with open(os.path.join(out, "board_data.rs"), "w") as f:
        f.write(rust(squares, places))
    control(pdf, squares, places, os.path.join(out, "control.png"))
    print(f"wrote {out}/board_data.rs and {out}/control.png")


if __name__ == "__main__":
    main()
