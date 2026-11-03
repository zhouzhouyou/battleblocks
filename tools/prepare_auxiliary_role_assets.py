"""Rebuild the unchanged dwarf book layers and nun standees from historical sources.

Keep their original palettes separate from the new character animation palettes.
"""
import json
from prepare_character_animations import Image, ROOT, save_webp


def palette_from(path):
    colors = json.loads(path.read_text(encoding="utf-8"))["palette"]
    rgb = [tuple(bytes.fromhex(color)) for color in colors]
    palette = Image.new("P", (1, 1))
    palette.putpalette([v for color in rgb + [rgb[-1]] * (256 - len(rgb)) for v in color])
    return palette


def colored(image, palette, binary=False):
    alpha = image.getchannel("A")
    if binary:
        alpha = alpha.point(lambda v: 255 if v >= 160 else 0)
    result = image.convert("RGB").quantize(palette=palette, dither=Image.Dither.NONE).convert("RGBA")
    result.putalpha(alpha)
    return result


def import_book():
    root = ROOT / "asset-sources/art-direction/characters/dwarf"
    palette = palette_from(root / "animations/v6/runtime-palette.json")
    layers = {name: Image.open(root / f"animations/v6/book-{name}.webp").convert("RGBA")
              for name in ("cover", "pages", "bottom")}
    cover = layers["cover"].resize((160, 24), Image.Resampling.NEAREST)
    bottom = layers["bottom"].resize((160, 8), Image.Resampling.NEAREST)
    atlas = Image.new("RGBA", (384 * 4, 384))
    for level in range(1, 4):
        pages = layers["pages"].resize((160, level * 12), Image.Resampling.NEAREST)
        y = 336 - (8 + level * 3) * 4
        atlas.alpha_composite(cover, (384 * level + 112, y))
        atlas.alpha_composite(pages, (384 * level + 112, y + 24))
        atlas.alpha_composite(bottom, (384 * level + 112, 328))
    save_webp(colored(atlas, palette), ROOT / "assets/roles/grudge-book.webp")
    for name, layer in (("cover", cover), ("bottom", bottom),
                        ("pages", layers["pages"].resize((160, 12), Image.Resampling.NEAREST))):
        save_webp(colored(layer, palette), ROOT / f"assets/roles/book-{name}.webp")


def import_standees():
    root = ROOT / "asset-sources/art-direction/characters/nun/production/v1"
    palette = palette_from(root / "report.json")
    for name in ("normal", "awake"):
        source = Image.open(root / f"standee-{name}-source-v1.webp").convert("RGBA")
        result = colored(source, palette, binary=True)
        save_webp(result, ROOT / f"assets/roles/nun-standee-{name}.webp")


if __name__ == "__main__":
    import_book()
    import_standees()
