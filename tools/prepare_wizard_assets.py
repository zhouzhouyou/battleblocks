"""Export only the wizard's fine-pixel awakened idle/cast poses.

Normal artwork, animation metadata and thumbnail use the untouched legacy path.
The two existing fine-pixel source sheets are re-exported without redesign.
"""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / '.build-tools/art-site'))
from PIL import Image

SOURCE = ROOT / 'asset-sources/art-direction/characters/wizard/production/v3'
CLIPS = ('awake-idle', 'awake-cast')
FRAME, ANCHOR = 384, (192, 336)
CAST_MS = [60, 80, 90, 90, 120, 180, 280, 100, 100, 100, 100, 100]


def save(image, path):
    image.save(path, 'WEBP', lossless=True, exact=True, method=6)


def import_wizard():
    registration = json.loads((SOURCE/'registration.json').read_text())
    colors = json.loads((SOURCE/'palette.json').read_text())
    assert len(colors) == 64*3
    palette = Image.new('P', (1,1))
    palette.putpalette(colors + colors[-3:]*(256-64))
    clips = {}
    for clip in CLIPS:
        image = Image.open(SOURCE/f'{clip}-source-v3.webp').convert('RGBA')
        assert image.size == (1448,1086), 'Expected 4x3 authored square cells'
        cells = []
        for i,anchor in enumerate(registration[clip]):
            x,y = i%4*362,i//4*362
            cell = image.crop((x,y,x+362,y+362))
            cell.putalpha(cell.getchannel('A').point(lambda v:255 if v>=160 else 0))
            bounds = cell.getbbox()
            if not bounds or min(bounds[:2]) <= 0 or max(bounds[2:]) >= 362:
                raise ValueError(f'{clip}/{i}: source crosses cell edge')
            cells.append((cell,anchor,bounds))
        assert len(cells) == 12
        clips[clip] = cells
    scale = 320/max(a[1]-b[1] for _,a,b in clips['awake-idle'])
    size = round(362*scale)
    atlas = Image.new('RGBA',(FRAME*6,FRAME*4))
    report = {'frame_size':[FRAME,FRAME], 'atlas_size':list(atlas.size),
              'frames':24, 'foot_anchor':list(ANCHOR), 'scale':scale,
              'palette':colors, 'form':'awakened-only', 'clips':{}}
    all_frames = []
    for clip,cells in clips.items():
        sequence,checks = [],[]
        for cell,anchor,_ in cells:
            resized = cell.resize((size,size),Image.Resampling.NEAREST)
            colored = resized.convert('RGB').quantize(palette=palette,dither=Image.Dither.NONE).convert('RGBA')
            colored.putalpha(resized.getchannel('A'))
            offset = (ANCHOR[0]-round(anchor[0]*size/362),ANCHOR[1]-round(anchor[1]*size/362))
            b = colored.getbbox()
            placed = [b[0]+offset[0],b[1]+offset[1],b[2]+offset[0],b[3]+offset[1]]
            if placed[0]<4 or placed[1]<0 or placed[2]>FRAME-4 or placed[3]!=ANCHOR[1]:
                raise ValueError(f'{clip}: unsafe registered bounds {placed}')
            frame = Image.new('RGBA',(FRAME,FRAME))
            frame.alpha_composite(colored,offset)
            frame.putdata([p if p[3] else (0,0,0,0) for p in frame.get_flattened_data()])
            checks.append({'source_foot':anchor,'bounds':frame.getbbox(),
                           'sha256':hashlib.sha256(frame.tobytes()).hexdigest()})
            sequence.append(frame)
        assert len({c['sha256'] for c in checks}) == 12, 'Need independent authored poses'
        durations = [60]*12 if clip.endswith('idle') else CAST_MS
        start = len(all_frames)
        report['clips'][clip] = {'indices':list(range(start,start+12)), 'checks':checks,
            'duration_ms':sum(durations),'durations_ms':durations,
            'playback':'loop' if clip.endswith('idle') else 'once','source':f'{clip}-source-v3.webp'}
        sequence[0].save(SOURCE/f'{clip}.webp','WEBP',save_all=True,append_images=sequence[1:],
            duration=durations,loop=0,lossless=True,exact=True,method=6)
        all_frames.extend(sequence)
    for i,frame in enumerate(all_frames):
        atlas.alpha_composite(frame,(i%6*FRAME,i//6*FRAME))
    path = ROOT/'assets/roles/wizard.webp'
    save(atlas,path)
    save(all_frames[0],SOURCE/'awake-idle-0.webp')
    report['webp_bytes'] = path.stat().st_size
    (SOURCE/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f'wizard: 24 fine-pixel awakened poses; normal unchanged; {path.stat().st_size} bytes')


if __name__ == '__main__':
    import_wizard()
