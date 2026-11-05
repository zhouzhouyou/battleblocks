"""Export the original coarse wizard style on a locked 24x24 pixel grid.

The user rejected the detailed full-body repaint. Normal retains the original
bust and seven colors; awakened retains the old staff/robe equipment silhouette.
Only authored source poses are sampled; no tweened or translated filler poses.
"""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / '.build-tools/art-site'))
from PIL import Image

SOURCE = ROOT / 'asset-sources/art-direction/characters/wizard/production/v2'
CLIPS = ('normal-idle', 'awake-idle', 'normal-cast', 'awake-cast')
FRAME, ANCHOR = 384, (192, 336)
CAST_MS = [60, 80, 90, 90, 120, 180, 280, 100, 100, 100, 100, 100]


def save(image, path):
    image.save(path, 'WEBP', lossless=True, exact=True, method=6)


def import_wizard():
    original = Image.open(ROOT / 'assets/wizzard.webp').convert('RGBA')
    colors = sorted({p[:3] for p in original.get_flattened_data() if p[3]})
    assert len(colors) == 7, 'Review changes to the original wizard palette'
    colors += [(38, 188, 224), (225, 248, 255)]  # awakened crystal only
    palette = Image.new('P', (1, 1))
    flat = [v for color in colors for v in color]
    palette.putpalette(flat + list(colors[-1]) * (256-len(colors)))
    sheets = {}
    for form in ('normal', 'awake'):
        sheet = Image.open(SOURCE / f'{form}-source-v2.webp').convert('RGBA')
        assert sheet.size == (1024, 1536), 'Expected 4x6 authored square cells'
        sheets[form] = sheet
    atlas = Image.new('RGBA', (FRAME*6, FRAME*8))
    report = {'frame_size': [FRAME, FRAME], 'atlas_size': list(atlas.size),
              'frames': 48, 'foot_anchor': list(ANCHOR), 'logical_size': [24,24],
              'pixel_scale': 16, 'palette': flat, 'clips': {}}
    all_frames = []
    for clip in CLIPS:
        form, action = clip.split('-')
        sequence, checks = [], []
        for i in range(12):
            source_index = i + (12 if action == 'cast' else 0)
            x, y = source_index % 4 * 256, source_index // 4 * 256
            cell = sheets[form].crop((x,y,x+256,y+256))
            cell.putalpha(cell.getchannel('A').point(lambda v: 255 if v >= 160 else 0))
            # A single scale for all poses, then fixed body-axis registration.
            small = cell.resize((22,22), Image.Resampling.NEAREST)
            colored = small.convert('RGB').quantize(palette=palette, dither=Image.Dither.NONE).convert('RGBA')
            colored.putalpha(small.getchannel('A'))
            logical = Image.new('RGBA', (24,24))
            logical.alpha_composite(colored, (0 if form == 'normal' else 2, 0))
            # Retain the actual original neutral portrait, not an approximation.
            if clip == 'normal-idle' and i == 0:
                logical = Image.new('RGBA', (24,24))
                logical.alpha_composite(original.crop((0,0,300,400)).resize((15,20),Image.Resampling.NEAREST),(4,1))
            logical.putdata([p if p[3] else (0,0,0,0) for p in logical.get_flattened_data()])
            bounds = logical.getbbox()
            if not bounds or bounds[0] <= 0 or bounds[2] >= 24 or bounds[3] != 21:
                raise ValueError(f'{clip}/{i}: unsafe logical bounds {bounds}')
            frame = logical.resize((FRAME,FRAME), Image.Resampling.NEAREST)
            checks.append({'logical_bounds': bounds, 'bounds': frame.getbbox(),
                           'sha256': hashlib.sha256(frame.tobytes()).hexdigest()})
            sequence.append(frame)
        if len({c['sha256'] for c in checks}) != 12:
            raise ValueError(f'{clip}: poses collapse on the coarse grid; repair source')
        durations = [60]*12 if action == 'idle' else CAST_MS
        start = len(all_frames)
        report['clips'][clip] = {'indices': list(range(start,start+12)),
            'checks': checks, 'duration_ms': sum(durations), 'durations_ms': durations,
            'playback': 'loop' if action == 'idle' else 'once', 'source': f'{form}-source-v2.webp'}
        sequence[0].save(SOURCE/f'{clip}.webp', 'WEBP', save_all=True,
            append_images=sequence[1:], duration=durations, loop=0, lossless=True, exact=True, method=6)
        all_frames.extend(sequence)
    for i,frame in enumerate(all_frames):
        atlas.alpha_composite(frame,(i%6*FRAME,i//6*FRAME))
    path = ROOT/'assets/roles/wizard.webp'
    save(atlas,path)
    save(all_frames[0],SOURCE/'normal-idle-0.webp')
    report['webp_bytes'] = path.stat().st_size
    (SOURCE/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f'wizard: 48 distinct coarse poses, 24x24 logical pixels, {len(colors)} colors, {path.stat().st_size} bytes')


if __name__ == '__main__':
    import_wizard()
