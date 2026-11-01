"""Export the forest fire elf's reviewed animation source as inspectable WebP art.

This stages art under asset-sources only. It does not register a playable role.
"""

from __future__ import annotations

import json
from collections import deque
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "asset-sources/art-direction/characters/forest-fire-elf/production/v1"
FRAME = 384
ANCHOR = (192, 336)
ALPHA_THRESHOLD = 160
# The generated source has unequal row heights. Boundaries follow the valleys
# between the four hand-drawn animation rows, not an assumed uniform grid.
XS = (0, 313, 626, 944, 1254)
YS = (0, 350, 693, 993, 1254)
ROWS = ("normal-idle", "focused-idle", "fire-explosion", "fall-asleep")


def opaque_mask(image: Image.Image) -> Image.Image:
    return image.getchannel("A").point(
        lambda value: 255 if value >= ALPHA_THRESHOLD else 0
    )


def remove_row_fragments(alpha: Image.Image) -> Image.Image:
    """Discard tiny pieces of adjacent poses at the hand-drawn row cuts."""
    width, height = alpha.size
    data = bytearray(alpha.tobytes())
    seen = bytearray(len(data))
    components: list[list[int]] = []
    for start, value in enumerate(data):
        if not value or seen[start]:
            continue
        pending = deque([start])
        seen[start] = 1
        component: list[int] = []
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
    cutoff = max(20, round(largest * 0.01))
    clean = bytearray(len(data))
    for component in components:
        if len(component) >= cutoff:
            for pos in component:
                clean[pos] = 255
    return Image.frombytes("L", alpha.size, bytes(clean))


def main() -> None:
    source = Image.open(SOURCE / "animation-source-v1.webp").convert("RGBA")
    assert source.size == (1254, 1254), source.size
    cells: list[Image.Image] = []
    boxes: list[tuple[int, int, int, int]] = []
    for row in range(4):
        for col in range(4):
            cell = source.crop((XS[col], YS[row], XS[col + 1], YS[row + 1]))
            alpha = remove_row_fragments(opaque_mask(cell))
            bounds = alpha.getbbox()
            assert bounds is not None, (row, col, "empty frame")
            cell.putalpha(alpha)
            cells.append(cell.crop(bounds))
            boxes.append(bounds)

    # All poses share one source-to-frame scale, so the sleeping elf remains
    # physically smaller than the standing elf instead of being enlarged.
    standing_height = max(cell.height for cell in cells[:12])
    scale = 320 / standing_height
    samples: list[tuple[int, int, int]] = []
    for cell in cells:
        pixels = [pixel[:3] for pixel in cell.get_flattened_data() if pixel[3]]
        samples.extend(pixels[:: max(1, len(pixels) // 4000)])
    sample_strip = Image.new("RGB", (len(samples), 1))
    sample_strip.putdata(samples)
    palette = sample_strip.quantize(
        colors=64, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE
    )

    frames: list[Image.Image] = []
    checks = []
    for number, cell in enumerate(cells):
        row, col = divmod(number, 4)
        size = (round(cell.width * scale), round(cell.height * scale))
        assert 0 < size[0] <= FRAME and 0 < size[1] <= 320, (row, col, size)
        cell = cell.resize(size, Image.Resampling.NEAREST)
        reduced = cell.convert("RGB").quantize(
            palette=palette, dither=Image.Dither.NONE
        ).convert("RGBA")
        reduced.putalpha(cell.getchannel("A"))
        frame = Image.new("RGBA", (FRAME, FRAME))
        frame.alpha_composite(reduced, ((FRAME - size[0]) // 2, ANCHOR[1] - size[1]))
        bounds = frame.getchannel("A").getbbox()
        assert bounds is not None and bounds[3] == ANCHOR[1], (row, col, bounds)
        assert set(frame.getchannel("A").get_flattened_data()) <= {0, 255}
        color_count = len(
            {pixel[:3] for pixel in frame.get_flattened_data() if pixel[3]}
        )
        assert color_count <= 64, (row, col, color_count)
        frames.append(frame)
        checks.append(
            {
                "row": row,
                "column": col,
                "source_bounds": boxes[number],
                "frame_bounds": bounds,
                "colors": color_count,
            }
        )

    atlas = Image.new("RGBA", (FRAME * 4, FRAME * 4))
    for number, frame in enumerate(frames):
        atlas.alpha_composite(frame, (number % 4 * FRAME, number // 4 * FRAME))
    atlas.save(SOURCE / "grid-384.webp", "WEBP", lossless=True, exact=True, method=6)
    frames[0].save(
        SOURCE / "normal-idle-0.webp", "WEBP", lossless=True, exact=True, method=6
    )
    for row, name in enumerate(ROWS):
        sequence = frames[row * 4 : row * 4 + 4]
        sequence[0].save(
            SOURCE / f"{name}.webp",
            "WEBP",
            save_all=True,
            append_images=sequence[1:],
            duration=180 if row < 2 else 140,
            loop=0,
            lossless=True,
            exact=True,
            method=6,
        )

    report = {
        "source": "animation-source-v1.webp",
        "atlas": "grid-384.webp",
        "rows": ROWS,
        "frame_size": [FRAME, FRAME],
        "frames": len(frames),
        "foot_anchor": ANCHOR,
        "shared_palette_colors": 64,
        "source_scale": scale,
        "checks": checks,
        "gameplay_integrated": False,
    }
    (SOURCE / "report.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps({key: value for key, value in report.items() if key != "checks"}))


if __name__ == "__main__":
    main()
