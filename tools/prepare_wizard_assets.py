"""Import Issue #27's authored poses using shoe anchors, never weapon bounds.

All four clips use one source scale, one palette and explicit source foot origins.
This is export/registration only: no tweening, pose synthesis or per-frame scaling.
"""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / '.build-tools/art-site'))
from PIL import Image

SOURCE = ROOT / 'asset-sources/art-direction/characters/wizard/production/v1'
CLIPS = ('normal-idle', 'awake-idle', 'normal-cast', 'awake-cast')
FRAME, ANCHOR = 384, (192, 336)
CAST_MS = [60, 80, 90, 90, 120, 180, 280, 100, 100, 100, 100, 100]


def save(image, path):
    image.save(path, 'WEBP', lossless=True, exact=True, method=6)


def import_wizard():
    registration = json.loads((SOURCE / 'registration.json').read_text())
    clips = {}
    for clip in CLIPS:
        im = Image.open(SOURCE / f'{clip}-source-v1.webp').convert('RGBA')
        if im.size != (1448, 1086):
            raise ValueError(f'{clip}: source layout changed; review shoe anchors')
        cells = []
        for i, anchor in enumerate(registration[clip]):
            x, y = i % 4 * 362, i // 4 * 362
            cell = im.crop((x, y, x + 362, y + 362))
            cell.putalpha(cell.getchannel('A').point(lambda v: 255 if v >= 160 else 0))
            bounds = cell.getbbox()
            if not bounds or min(bounds[:2]) <= 0 or max(bounds[2:]) >= 362:
                raise ValueError(f'{clip}/{i}: source crosses a cell edge')
            cells.append((cell, anchor, bounds))
        if len(cells) != 12:
            raise ValueError(f'{clip}: need twelve authored poses')
        clips[clip] = cells

    # Lock body scale across forms AND actions. Weapon extension cannot shrink a pose.
    reference_height = max(a[1] - b[1] for clip in CLIPS[:2] for _, a, b in clips[clip])
    scale = 320 / reference_height
    sample = Image.new('RGB', (362 * 12, 362 * 4), (22, 24, 32))
    accents = []
    for row, cells in enumerate(clips.values()):
        for col, (cell, _, _) in enumerate(cells):
            sample.paste(cell, (col * 362, row * 362), cell.getchannel('A'))
            accents.extend(p[:3] for p in cell.get_flattened_data()
                           if p[3] and p[2] > 150 and p[1] > 130 and p[2] > p[0] * 1.25)
    palette = sample.quantize(colors=56, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    strip = Image.new('RGB', (len(accents), 1))
    strip.putdata(accents)
    blue = strip.quantize(colors=8, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    colors = palette.getpalette()[:168] + blue.getpalette()[:24]
    palette.putpalette(colors + colors[-3:] * (256 - 64))
    atlas = Image.new('RGBA', (FRAME * 6, FRAME * 8))
    report = {'frame_size': [FRAME, FRAME], 'atlas_size': list(atlas.size), 'frames': 48,
              'foot_anchor': list(ANCHOR), 'scale': scale, 'palette': colors, 'clips': {}}
    all_frames = []
    # Resample whole registered cells at one scale. Offsets depend on soles, NOT bounds.
    size = round(362 * scale)
    for clip, cells in clips.items():
        sequence, checks = [], []
        for cell, anchor, _ in cells:
            resized = cell.resize((size, size), Image.Resampling.NEAREST)
            colored = resized.convert('RGB').quantize(palette=palette, dither=Image.Dither.NONE).convert('RGBA')
            colored.putalpha(resized.getchannel('A'))
            offset = (ANCHOR[0] - round(anchor[0] * size / 362),
                      ANCHOR[1] - round(anchor[1] * size / 362))
            bounds = colored.getbbox()
            placed = [bounds[0]+offset[0], bounds[1]+offset[1], bounds[2]+offset[0], bounds[3]+offset[1]]
            if placed[0] < 4 or placed[1] < 0 or placed[2] > FRAME-4 or placed[3] > ANCHOR[1]:
                raise ValueError(f'{clip}: unsafe registered bounds {placed}; repair source/registration')
            frame = Image.new('RGBA', (FRAME, FRAME))
            frame.alpha_composite(colored, offset)
            frame.putdata([p if p[3] else (0, 0, 0, 0) for p in frame.get_flattened_data()])
            checks.append({'source_foot': anchor, 'bounds': frame.getbbox(),
                           'sha256': hashlib.sha256(frame.tobytes()).hexdigest()})
            sequence.append(frame)
        if len({c['sha256'] for c in checks}) != 12:
            raise ValueError(f'{clip}: duplicate authored poses')
        durations = [60] * 12 if 'idle' in clip else CAST_MS
        start = len(all_frames)
        report['clips'][clip] = {'indices': list(range(start, start+12)), 'checks': checks,
            'duration_ms': sum(durations), 'durations_ms': durations,
            'playback': 'loop' if 'idle' in clip else 'once',
            'source': f'{clip}-source-v1.webp'}
        # Preview files repeat for review. Runtime one-shot behavior is in AnimationClip.
        sequence[0].save(SOURCE / f'{clip}.webp', 'WEBP', save_all=True,
                         append_images=sequence[1:], duration=durations, loop=0,
                         lossless=True, exact=True, method=6)
        all_frames.extend(sequence)
    for i, frame in enumerate(all_frames):
        atlas.alpha_composite(frame, (i % 6 * FRAME, i // 6 * FRAME))
    path = ROOT / 'assets/roles/wizard.webp'
    save(atlas, path)
    save(all_frames[0], SOURCE / 'normal-idle-0.webp')
    report['webp_bytes'] = path.stat().st_size
    (SOURCE / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'wizard: 48 registered poses, shared scale {scale:.6f}, {path.stat().st_size} bytes')


if __name__ == '__main__':
    import_wizard()
