"""Import the approved nun concept sources into inspectable game-ready art files.

This stages resources without adding the nun to the playable roster. The creative
drawing is kept in versioned source PNGs; this importer only slices, anchors,
quantizes, and packs pixels on the same grid as the existing role atlases.
"""

from __future__ import annotations

import json
from collections import deque
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "asset-sources/art-direction/characters/nun/production/v1"
OUTPUT = ROOT / "assets/roles"
FRAME = 384
ANCHOR = (192, 336)
ROW_RANGES = ((0, 317), (315, 634), (627, 940), (936, 1254))
COLS = tuple(round(i * 1254 / 4) for i in range(5))
THRESHOLD = 160


def binary_alpha(image: Image.Image) -> Image.Image:
    return image.getchannel("A").point(lambda value: 255 if value >= THRESHOLD else 0)


def connected_alpha(alpha: Image.Image) -> Image.Image:
    """Keep the figure and lantern, discarding adjacent-row fragments."""
    width, height = alpha.size
    data = bytearray(alpha.tobytes())
    seen = bytearray(width * height)
    components = []
    for start, value in enumerate(data):
        if not value or seen[start]:
            continue
        component = []
        pending = deque([start])
        seen[start] = 1
        while pending:
            at = pending.popleft()
            component.append(at)
            x, y = at % width, at // width
            for ny in range(max(0, y - 1), min(height, y + 2)):
                for nx in range(max(0, x - 1), min(width, x + 2)):
                    pos = ny * width + nx
                    if data[pos] and not seen[pos]:
                        seen[pos] = 1
                        pending.append(pos)
        components.append(component)
    largest = max(map(len, components), default=0)
    output = bytearray(width * height)
    for component in components:
        if len(component) >= largest * 0.03:
            for pos in component:
                output[pos] = 255
    return Image.frombytes("L", alpha.size, bytes(output))


def make_palette(images: list[Image.Image]) -> Image.Image:
    samples: list[tuple[int, int, int]] = []
    for image in images:
        rgba = image.convert("RGBA")
        opaque = [pixel[:3] for pixel in rgba.get_flattened_data() if pixel[3] >= THRESHOLD]
        step = max(1, len(opaque) // 50000)
        samples.extend(opaque[::step])
    strip = Image.new("RGB", (len(samples), 1))
    strip.putdata(samples)
    return strip.quantize(colors=64, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)


def reduce_colors(image: Image.Image, palette: Image.Image) -> Image.Image:
    alpha = binary_alpha(image)
    reduced = image.convert("RGB").quantize(palette=palette, dither=Image.Dither.NONE).convert("RGBA")
    reduced.putalpha(alpha)
    return reduced


def main() -> None:
    OUTPUT.mkdir(exist_ok=True)
    source = Image.open(SOURCE / "animation-source-v2.png").convert("RGBA")
    assert source.size == (1254, 1254)
    standees = {
        "normal": Image.open(SOURCE / "standee-normal-source-v1.png").convert("RGBA"),
        "awake": Image.open(SOURCE / "standee-awake-source-v1.png").convert("RGBA"),
    }
    palette = make_palette([source, *standees.values()])
    palette_colors = [palette.getpalette()[i : i + 3] for i in range(0, 64 * 3, 3)]

    row_heights = []
    for row in range(4):
        heights = []
        for col in range(4):
            top, bottom = ROW_RANGES[row]
            cell = source.crop((COLS[col], top, COLS[col + 1], bottom))
            box = connected_alpha(binary_alpha(cell)).getbbox()
            assert box is not None, (row, col, "empty source cell")
            heights.append(box[3] - box[1])
        row_heights.append(max(heights))

    frames = []
    frame_checks = []
    for row in range(4):
        scale = 320 / row_heights[row]
        for col in range(4):
            top, bottom = ROW_RANGES[row]
            cell = source.crop((COLS[col], top, COLS[col + 1], bottom))
            alpha = connected_alpha(binary_alpha(cell))
            box = alpha.getbbox()
            assert box is not None
            cell.putalpha(alpha)
            cell = cell.crop(box)
            size = (round(cell.width * scale), round(cell.height * scale))
            assert 0 < size[0] <= FRAME - 4 and 0 < size[1] <= 320, (row, col, size)
            cell = cell.resize(size, Image.Resampling.NEAREST)
            cell = reduce_colors(cell, palette)
            frame = Image.new("RGBA", (FRAME, FRAME))
            frame.alpha_composite(cell, ((FRAME - size[0]) // 2, ANCHOR[1] - size[1]))
            box = frame.getchannel("A").getbbox()
            assert box is not None and box[3] == ANCHOR[1], (row, col, box)
            colors = {pixel[:3] for pixel in frame.get_flattened_data() if pixel[3]}
            assert len(colors) <= 64
            assert set(frame.getchannel("A").get_flattened_data()) <= {0, 255}
            frames.append(frame)
            frame_checks.append({"row": row, "column": col, "bounds": box, "colors": len(colors)})

    atlas = Image.new("RGBA", (FRAME * 4, FRAME * 4))
    for number, frame in enumerate(frames):
        atlas.alpha_composite(frame, (number % 4 * FRAME, number // 4 * FRAME))
    atlas.save(SOURCE / "grid-384.png")
    atlas.save(OUTPUT / "nun.webp", format="WEBP", lossless=True, exact=True, method=6)
    frames[0].save(SOURCE / "normal-idle-0.png")
    frames[4].save(SOURCE / "awake-idle-0.png")
    for name, row in (("normal-idle", 0), ("awake-idle", 1), ("normal-heal", 2), ("awake-attack", 3)):
        sequence = frames[row * 4 : row * 4 + 4]
        sequence[0].save(
            SOURCE / f"{name}.gif",
            save_all=True,
            append_images=sequence[1:],
            duration=180 if row < 2 else 130,
            loop=0,
            disposal=2,
            transparency=0,
        )

    for name, image in standees.items():
        reduced = reduce_colors(image, palette)
        assert set(reduced.getchannel("A").get_flattened_data()) <= {0, 255}
        reduced.save(SOURCE / f"standee-{name}.png")
        reduced.save(OUTPUT / f"nun-standee-{name}.webp", format="WEBP", lossless=True, exact=True, method=6)

    report = {
        "source": "animation-source-v2.png",
        "frame_size": [FRAME, FRAME],
        "frames": len(frames),
        "rows": ["normal-idle", "awake-idle", "normal-heal", "awake-attack"],
        "visible_height": 320,
        "foot_anchor": ANCHOR,
        "palette": ["".join(f"{value:02X}" for value in rgb) for rgb in palette_colors],
        "checks": frame_checks,
        "standee_size": {name: image.size for name, image in standees.items()},
        "runtime_staged": ["nun.webp", "nun-standee-normal.webp", "nun-standee-awake.webp"],
    }
    (SOURCE / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: value for key, value in report.items() if key not in ("checks", "palette")}))


if __name__ == "__main__":
    main()
