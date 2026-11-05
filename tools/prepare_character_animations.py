"""Import the versioned, drawn 12-pose character sheets (no synthetic tweening).

Requires Pillow. Run with .build-tools/python-path.txt's Python. Creative changes
belong in the source sheets; this tool only slices, anchors, quantizes and packs.
"""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / ".build-tools/art-site"))
from PIL import Image
FRAME = 384
ANCHOR = (192, 336)
COLUMNS = 6
CLIPS = ("normal-idle", "awake-idle", "normal-cast", "awake-cast")
ROLES = {
    "knight": ("knight", "v4"),
    "dwarf": ("dwarf", "v4"),
    "priest": ("priest", "v5"),
    "nun": ("nun", "v2"),
    "fire-elf": ("forest-fire-elf", "v2"),
}


def source_dir(role):
    name, version = ROLES[role]
    return ROOT / "asset-sources/art-direction/characters" / name / "production" / version


def clips_for(role):
    return (*CLIPS[:3], "sleep") if role == "fire-elf" else CLIPS


def save_webp(image, path):
    image.save(path, "WEBP", lossless=True, exact=True, method=6)


def load_cells(path):
    source = Image.open(path).convert("RGBA")
    frames = []
    for row in range(3):
        for col in range(4):
            cell = source.crop((round(col * source.width / 4), round(row * source.height / 3),
                                round((col + 1) * source.width / 4), round((row + 1) * source.height / 3)))
            alpha = cell.getchannel("A").point(lambda value: 255 if value >= 160 else 0)
            cell.putalpha(alpha)
            bounds = alpha.getbbox()
            if not bounds:
                raise ValueError(f"Empty pose: {path}, {row}, {col}")
            if bounds[0] == 0 or bounds[1] == 0 or bounds[2] == cell.width or bounds[3] == cell.height:
                raise ValueError(f"Pose touches source cell edge: {path}, {row}, {col}; repair the source")
            frames.append((cell, bounds))
    return frames


def import_role(role):
    directory = source_dir(role)
    sources = {clip: directory / f"{clip}-source-v{2 if role == 'knight' and 'cast' in clip else 1}.webp"
               for clip in clips_for(role)}
    clips = {clip: load_cells(path) for clip, path in sources.items()}
    # One deterministic palette per character, shared by every form and action.
    samples = Image.new("RGB", (384 * 12, 384 * 4), (22, 24, 32))
    for row, cells in enumerate(clips.values()):
        for col, (cell, _) in enumerate(cells):
            cell = cell.copy()
            cell.thumbnail((384, 384), Image.Resampling.NEAREST)
            samples.paste(cell, (col * 384, row * 384), cell.getchannel("A"))
    # A small lantern occupies too few pixels for global median-cut to retain its
    # identity color. Reserve eight source-derived purple shades for the nun.
    accents = [p[:3] for cells in clips.values() for cell, _ in cells
               for p in cell.get_flattened_data()
               if p[3] and p[2] > 70 and p[2] > p[1] * 1.3 and p[0] > p[1] * 1.3] if role == "nun" else []
    palette = samples.quantize(colors=56 if accents else 64, method=Image.Quantize.MEDIANCUT,
                               dither=Image.Dither.NONE)
    if accents:
        strip = Image.new("RGB", (len(accents), 1))
        strip.putdata(accents)
        accent_palette = strip.quantize(colors=8, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
        colors = palette.getpalette()[:56 * 3] + accent_palette.getpalette()[:8 * 3]
        palette.putpalette(colors + colors[-3:] * (256 - 64))
    atlas = Image.new("RGBA", (COLUMNS * FRAME, 8 * FRAME))
    report = {"frame_size": [FRAME, FRAME], "atlas_size": list(atlas.size), "frames": 48,
              "foot_anchor": list(ANCHOR), "palette": palette.getpalette()[:192], "clips": {},
              "sources": {clip: path.name for clip, path in sources.items()}}
    all_frames = []
    for clip, cells in clips.items():
        # Scale all poses of an action together, preserving crouches and sleep.
        # Standing first pose sets the height; margins also protect wide swings.
        first = cells[0][1]
        reference_height = (max(b[3] - b[1] for _, b in cells)
                            if "idle" in clip else first[3] - first[1])
        scale = min(320 / reference_height,
                    min((FRAME - 8) / (b[2] - b[0]) for _, b in cells),
                    min(ANCHOR[1] / (b[3] - b[1]) for _, b in cells))
        sequence = []
        checks = []
        for cell, bounds in cells:
            crop = cell.crop(bounds)
            size = (round(crop.width * scale), round(crop.height * scale))
            crop = crop.resize(size, Image.Resampling.NEAREST)
            colored = crop.convert("RGB").quantize(palette=palette, dither=Image.Dither.NONE).convert("RGBA")
            colored.putalpha(crop.getchannel("A"))
            frame = Image.new("RGBA", (FRAME, FRAME))
            frame.alpha_composite(colored, ((FRAME - size[0]) // 2, ANCHOR[1] - size[1]))
            # Hidden RGB is canonical too, so byte hashes describe visible poses.
            frame.putdata([p if p[3] else (0, 0, 0, 0) for p in frame.get_flattened_data()])
            checks.append({"bounds": frame.getbbox(), "sha256": hashlib.sha256(frame.tobytes()).hexdigest()})
            sequence.append(frame)
        if len({c["sha256"] for c in checks}) != 12:
            raise ValueError(f"{role}/{clip}: repeated poses; redraw the source")
        duration = 60 if "idle" in clip else [117] * 8 + [116] * 4
        sequence[0].save(directory / f"{clip}.webp", "WEBP", save_all=True, append_images=sequence[1:],
                         duration=duration, loop=0, lossless=True, exact=True, method=6)
        start = len(all_frames)
        report["clips"][clip] = {"indices": list(range(start, start + 12)),
            "duration_ms": 720 if "idle" in clip else 1400, "scale": scale, "checks": checks,
            "playback": "loop" if "idle" in clip else "once"}
        all_frames.extend(sequence)
    for i, frame in enumerate(all_frames):
        atlas.alpha_composite(frame, (i % COLUMNS * FRAME, i // COLUMNS * FRAME))
    save_webp(atlas, ROOT / "assets/roles" / f"{role}.webp")
    save_webp(all_frames[0], directory / "normal-idle-0.webp")
    report["webp_bytes"] = (ROOT / "assets/roles" / f"{role}.webp").stat().st_size
    (directory / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"{role}: 48 distinct poses, {report['webp_bytes']} bytes")


def registry():
    legacy = json.loads((ROOT / "assets/report.json").read_text())
    normal, awakened = [], []
    thumbs = Image.new("RGBA", ((7+len(ROLES)+1)*40, 48))
    for i, name in enumerate(("knight", "elf", "wizzard", "goblin", "imp", "chort", "big_zombie")):
        meta = legacy[name]
        w, h = meta["frame_size"]
        normal.append(f'    SpriteAsset {{ bytes: include_bytes!("../../assets/{name}.webp"), w: {w}., h: {h}., durations: &{meta["durations"]}, indices: &{meta["indices"]}, cast: None, sleep: None, anchor: None }},')
        frame = Image.open(ROOT / f"assets/{name}.webp").crop((0, 0, w, h))
        frame.thumbnail((40, 48), Image.Resampling.NEAREST)
        thumbs.alpha_composite(frame.convert("RGBA"), (i * 40 + (40 - frame.width) // 2, 48 - frame.height))
    for name in ("knight", "elf", "wizard", "goblin", "imp", "chort", "zombie"):
        if name in ('knight', 'elf', 'goblin'):
            awakened.append(f'    SpriteAsset {{ bytes: include_bytes!("../../assets/roles/{name}-legacy.webp"), w: 384., h: 384., durations: &[0.06; 12], indices: &{list(range(12))}, cast: Some(LEGACY_AWAKE_CAST), sleep: None, anchor: Some([192., 336.]) }},')
            continue
        if name == 'wizard':
            awakened.append(f'    SpriteAsset {{ bytes: include_bytes!("../../assets/roles/wizard.webp"), w: 384., h: 384., durations: &[0.06; 12], indices: &{list(range(12))}, cast: Some(WIZARD_AWAKE_CAST), sleep: None, anchor: Some([192., 336.]) }},')
            continue
        path = ROOT / f"assets/awakened/{name}-awakening.webp"
        w, h = Image.open(path).size
        awakened.append(f'    SpriteAsset {{ bytes: include_bytes!("../../assets/awakened/{name}-awakening.webp"), w: {w}., h: {h}., durations: &[1.], indices: &[0], cast: None, sleep: None, anchor: None }},')
    for i, role in enumerate(ROLES, 7):
        for awake, entries in ((False, normal), (True, awakened)):
            idle = list(range(12, 24)) if awake else list(range(12))
            cast = "AWAKE_CAST" if awake and role != "fire-elf" else "NORMAL_CAST"
            sleep = "Some(SLEEP)" if role == "fire-elf" else "None"
            entries.append(f'    SpriteAsset {{ bytes: include_bytes!("../../assets/roles/{role}.webp"), w: 384., h: 384., durations: &[0.06; 12], indices: &{idle}, cast: Some({cast}), sleep: {sleep}, anchor: Some([192., 336.]) }},')
        frame = Image.open(ROOT / f"assets/roles/{role}.webp").crop((0, 0, FRAME, FRAME)).resize((48, 48), Image.Resampling.NEAREST)
        thumbs.alpha_composite(frame.convert("RGBA"), (i * 40 - 4, 0))
    cube = Image.open(ROOT / "assets/roles/portal-cube.webp").crop((0,0,FRAME,FRAME))
    cube.thumbnail((40,48),Image.Resampling.NEAREST)
    thumbs.alpha_composite(cube.convert("RGBA"),((7+len(ROLES))*40+(40-cube.width)//2,(48-cube.height)//2))
    save_webp(thumbs, ROOT / "assets/hero-thumbnails.webp")
    report_path = ROOT / "assets/roles/report.json"
    report = json.loads(report_path.read_text(encoding="utf-8"))
    for role in ROLES:
        details = json.loads((source_dir(role) / "report.json").read_text(encoding="utf-8"))
        report[role] = {key: details[key] for key in ("frame_size", "atlas_size", "frames", "foot_anchor", "webp_bytes")}
        report[role]["detail_report"] = (source_dir(role) / "report.json").relative_to(ROOT).as_posix()
        report[role]["clips"] = {name: {key: clip[key] for key in ("indices", "duration_ms", "playback")}
                                for name, clip in details["clips"].items()}
    from prepare_wizard_assets import SOURCE
    details = json.loads((SOURCE / 'report.json').read_text())
    report['wizard'] = {key: details[key] for key in ('frame_size', 'atlas_size', 'frames', 'foot_anchor', 'webp_bytes')}
    report['wizard']['detail_report'] = (SOURCE / 'report.json').relative_to(ROOT).as_posix()
    report['wizard']['clips'] = {name: {key: clip[key] for key in ('indices', 'duration_ms', 'playback')}
                               for name, clip in details['clips'].items()}
    from prepare_legacy_awake_assets import ROLES as LEGACY_ROLES, source_dir as legacy_source
    for role in LEGACY_ROLES:
        directory = legacy_source(role)
        details = json.loads((directory/'report.json').read_text())
        report[role] = {key: details[key] for key in ('frame_size', 'atlas_size', 'frames', 'foot_anchor', 'webp_bytes')}
        report[role]['detail_report'] = (directory/'report.json').relative_to(ROOT).as_posix()
        report[role]['clips'] = {name: {key: clip[key] for key in ('indices', 'duration_ms', 'playback')}
                                for name, clip in details['clips'].items()}
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    normal.append("    PORTAL_NORMAL,")
    awakened.append("    PORTAL_AWAKE,")
    dest = ROOT / "src/sprites/registry.rs"
    dest.parent.mkdir(exist_ok=True)
    dest.write_text('// Generated by tools/prepare_character_animations.py; do not edit.\nuse super::*;\npub const SPRITES: &[SpriteAsset] = &[\n' + '\n'.join(normal) + '\n];\npub const AWAKENED: &[SpriteAsset] = &[\n' + '\n'.join(awakened) + '\n];\n', encoding="utf-8")
    subprocess.run(["rustfmt", str(dest)], check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    from prepare_legacy_awake_assets import ROLES as LEGACY_ROLES, import_role as import_legacy
    parser.add_argument("--role", choices=(*ROLES, 'wizard', *LEGACY_ROLES), help="Import one role without rebuilding the shared registry")
    parser.add_argument("--registry-only", action="store_true")
    args = parser.parse_args()
    if not args.registry_only:
        for role in LEGACY_ROLES:
            if not args.role or args.role == role:
                import_legacy(role)
        if not args.role or args.role == 'wizard':
            from prepare_wizard_assets import import_wizard
            import_wizard()
        for role in ([args.role] if args.role else ROLES):
            if role != 'wizard' and role not in LEGACY_ROLES:
                import_role(role)
    if not args.role:
        registry()


if __name__ == "__main__":
    main()
