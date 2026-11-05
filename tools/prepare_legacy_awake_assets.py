"""Import Issue 28's authored awake-only poses. Never writes normal assets.

Source plates have independent pixel density; registration fixes one scale per
plate, shared by all twelve poses, and uses soles rather than weapon bounds.
Palette and registration are versioned inputs, not re-estimated on rebuild.
"""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / '.build-tools/art-site'))
from PIL import Image

ROLES = ('knight-legacy', 'elf-legacy', 'goblin-legacy')
CLIPS = ('awake-idle', 'awake-cast')
FRAME, ANCHOR = 384, (192, 336)


def source_dir(role):
    return ROOT / 'assets-source/roles' / role / 'animation'


def source_cells(path):
    im = Image.open(path).convert('RGBA')
    cells = []
    for i in range(12):
        x, y = i % 4, i // 4
        cell = im.crop((round(x*im.width/4), round(y*im.height/3),
                        round((x+1)*im.width/4), round((y+1)*im.height/3)))
        cell.putalpha(cell.getchannel('A').point(lambda v: 255 if v >= 160 else 0))
        b = cell.getbbox()
        if not b or min(b[:2]) <= 0 or b[2] >= cell.width or b[3] >= cell.height:
            raise ValueError(f'{path.name}/{i}: invalid or cropped source pose')
        cells.append(cell)
    return cells


def import_role(role):
    directory = source_dir(role)
    previews = ROOT / '.local/previews' / role
    previews.mkdir(parents=True, exist_ok=True)
    registration = json.loads((directory/'registration.json').read_text())
    colors = json.loads((directory/'palette.json').read_text())
    assert len(colors) == 192
    palette = Image.new('P', (1, 1))
    palette.putpalette(colors + colors[-3:] * 192)
    atlas = Image.new('RGBA', (FRAME*6, FRAME*4))
    report = {'form': 'awakened-only', 'frame_size': [FRAME, FRAME],
              'atlas_size': list(atlas.size), 'frames': 24,
              'foot_anchor': list(ANCHOR), 'palette': colors, 'clips': {}}
    for row, clip in enumerate(CLIPS):
        reg = registration[clip]
        cells = source_cells(directory/reg['source'])
        frames, checks = [], []
        scale = reg['scale']
        for i, (cell, foot) in enumerate(zip(cells, reg['feet'], strict=True)):
            w, h = round(cell.width*scale), round(cell.height*scale)
            resized = cell.resize((w, h), Image.Resampling.NEAREST)
            rgb = resized.convert('RGB').quantize(palette=palette, dither=Image.Dither.NONE).convert('RGBA')
            rgb.putalpha(resized.getchannel('A'))
            b = rgb.getbbox()
            # Exact bottom alignment avoids nearest-neighbour rounding by 1 px.
            offset = (ANCHOR[0]-round(foot[0]*w/cell.width), ANCHOR[1]-b[3])
            placed = [b[0]+offset[0], b[1]+offset[1], b[2]+offset[0], b[3]+offset[1]]
            if placed[0]<4 or placed[1]<4 or placed[2]>FRAME-4:
                raise ValueError(f'{role}/{clip}/{i}: equipment outside safety margin {placed}')
            frame = Image.new('RGBA', (FRAME, FRAME))
            frame.alpha_composite(rgb, offset)
            frame.putdata([p if p[3] else (0, 0, 0, 0) for p in frame.get_flattened_data()])
            checks.append({'source_foot': foot, 'bounds': frame.getbbox(),
                           'sha256': hashlib.sha256(frame.tobytes()).hexdigest()})
            frames.append(frame)
            n = row*12+i
            atlas.alpha_composite(frame, (n%6*FRAME, n//6*FRAME))
        assert len({c['sha256'] for c in checks}) == 12, 'Repeated authored pose'
        durations = [60]*12 if row == 0 else [117]*8+[116]*4
        frames[0].save(previews/f'{clip}.webp', 'WEBP', save_all=True,
                       append_images=frames[1:], duration=durations, loop=0,
                       lossless=True, exact=True, method=6)
        if row == 0:
            frames[0].save(previews/'awake-idle-0.webp', lossless=True, exact=True, method=6)
        report['clips'][clip] = {'indices': list(range(row*12, row*12+12)),
            'source': reg['source'], 'scale': scale, 'checks': checks,
            'duration_ms': sum(durations), 'playback': 'loop' if row == 0 else 'once',
            'preview_durations_ms': durations,
            'runtime_timing': 'src/sprites/legacy.rs (per skill)' if row else '60ms per pose'}
    target = ROOT/'assets/roles'/role/'runtime/atlas.webp'
    atlas.save(target, 'WEBP', lossless=True, exact=True, method=6)
    report['webp_bytes'] = target.stat().st_size
    (directory/'report.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(f'{role}: 24 authored awake poses, {target.stat().st_size} bytes')


if __name__ == '__main__':
    for role in ROLES:
        import_role(role)
