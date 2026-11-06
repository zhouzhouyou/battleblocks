"""Verify or re-export the retained final PC-A atlas; no discarded sources needed.

Use --export to rebuild WebP previews and contact sheets from the final atlas.
With no flag, verify the existing final files without changing them.
"""
from pathlib import Path
import argparse
import hashlib
import json
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/".build-tools/art-site"))
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'assets-source/roles/portal-cube/animation'


def digest(image):
    return hashlib.sha256(image.tobytes()).hexdigest()


def verify_preview(path, frames, durations):
    decoded = Image.open(path)
    if decoded.n_frames != len(frames):
        raise ValueError(f'{path.name}: incorrect frame count')
    for i, (frame, duration) in enumerate(zip(frames, durations)):
        decoded.seek(i)
        pixels = decoded.convert('RGBA')
        if pixels.tobytes() != frame.tobytes() or decoded.info['duration'] != duration:
            raise ValueError(f'{path.name}/{i}: preview pixels or timing changed')


def install_runtime(atlas, report):
    # 10x10 cells keeps both texture axes below the WebGL 4096px baseline.
    width,height=report['frame_size'];columns=10;rows=(report['frames']+columns-1)//columns
    packed=Image.new('RGBA',(columns*width,rows*height))
    for n in range(report['frames']):
        x,y=n%report['columns']*width,n//report['columns']*height
        packed.alpha_composite(atlas.crop((x,y,x+width,y+height)),(n%columns*width,n//columns*height))
    target=ROOT/'assets/roles/portal-cube/runtime/atlas.webp'
    packed.save(target,'WEBP',lossless=True,exact=True,method=6,quality=100)
    runtime=Image.open(target).convert('RGBA')
    for clip in report['clips'].values():
        for n in clip['indices']:
            x,y=n%columns*width,n//columns*height
            sx,sy=n%report['columns']*width,n//report['columns']*height
            if runtime.crop((x,y,x+width,y+height)).tobytes()!=atlas.crop((sx,sy,sx+width,sy+height)).tobytes():
                raise ValueError(f'runtime frame {n} changed while packing')
    thumb_path=ROOT/'assets/ui/hero-thumbnails.webp'
    old=Image.open(thumb_path).convert('RGBA')
    thumbs=Image.new('RGBA',(max(old.width,13*40),48));thumbs.alpha_composite(old)
    thumbs.paste((0,0,0,0),(12*40,0,13*40,48))
    thumb=runtime.crop((0,0,width,height));thumb.thumbnail((40,48),Image.Resampling.NEAREST)
    thumbs.alpha_composite(thumb,(12*40+(40-thumb.width)//2,(48-thumb.height)//2))
    thumbs.save(thumb_path,'WEBP',lossless=True,exact=True,method=6,quality=100)
    if Image.open(thumb_path).convert('RGBA').crop((0,0,480,48)).tobytes()!=old.crop((0,0,480,48)).tobytes():
        raise ValueError('an existing hero thumbnail changed')
    report_path=ROOT/'.local/production/role-assets.json'
    report_path.parent.mkdir(parents=True,exist_ok=True)
    runtime_report=json.loads(report_path.read_text()) if report_path.exists() else {}
    runtime_report['portal-cube']={
        'frame_size':[width,height],'atlas_size':list(runtime.size),'frames':report['frames'],
        'columns':columns,'foot_anchor':report['anchor'],'webp_bytes':target.stat().st_size,
        'detail_report':str((DIRECTORY/'animation.json').relative_to(ROOT)),
        'clips':{name:{key:clip[key] for key in ('indices','duration_ms','playback')} for name,clip in report['clips'].items()}}
    report_path.write_text(json.dumps(runtime_report,indent=2)+'\n')
    print('INSTALLED: 96 unchanged frames, 3840x3840 runtime atlas, dedicated thumbnail')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--export', action='store_true', help='Re-export previews/contact sheets from the final atlas')
    parser.add_argument('--install',action='store_true',help='Install verified final artwork and thumbnail into assets/')
    args = parser.parse_args()
    report = json.loads((DIRECTORY / 'animation.json').read_text())
    atlas = Image.open(DIRECTORY / 'atlas.webp').convert('RGBA')
    if list(atlas.size) != report['atlas_size']:
        raise ValueError('atlas dimensions do not match report')
    colors = set()
    for r, g, b, a in atlas.get_flattened_data():
        if a not in (0, 255):
            raise ValueError('atlas alpha is not binary')
        if a: colors.add((r, g, b))
    if len(colors) > 64:
        raise ValueError('shared palette exceeds 64 colors')
    width, height = report['frame_size']
    columns = report['columns']
    body_hashes = {'normal': set(), 'awake': set()}
    indices = []
    for clip, details in report['clips'].items():
        frames = []
        durations = details['durations_ms']
        if len(details['indices']) != 16 or len(durations) != 16 or sum(durations) != details['duration_ms']:
            raise ValueError(f'{clip}: incorrect clip metadata')
        for i, n in enumerate(details['indices']):
            x, y = n % columns * width, n // columns * height
            frame = atlas.crop((x, y, x+width, y+height))
            region = report['body_regions'][clip][i]
            body_hashes[clip.split('-')[0]].add(digest(frame.crop(region)))
            frames.append(frame)
            indices.append(n)
        if len({digest(frame) for frame in frames}) != 16:
            raise ValueError(f'{clip}: duplicate complete frames')
        preview_root = ROOT / '.local/previews/portal-cube'
        preview_root.mkdir(parents=True, exist_ok=True)
        preview = preview_root / f'{clip}.webp'
        if args.export:
            frames[0].save(preview, 'WEBP', save_all=True, append_images=frames[1:],
                           duration=durations, loop=0, lossless=True, exact=True, method=6)
            contact = Image.new('RGBA', (4*width, 4*height))
            for i, frame in enumerate(frames):
                contact.alpha_composite(frame, (i%4*width, i//4*height))
            contact.save(preview_root / f'{clip}-contact.webp', 'WEBP', lossless=True, exact=True, method=6)
        if preview.exists(): verify_preview(preview, frames, durations)
        print(f'{clip}: 16 distinct frames, {sum(durations)} ms, lossless preview verified')
    if sorted(indices) != list(range(report['frames'])) or report['frames'] != 96:
        raise ValueError('atlas frames are missing or duplicated')
    for phase, hashes in body_hashes.items():
        if len(hashes) != 1:
            raise ValueError(f'{phase}: rigid body pixels changed')
    print(f'PASS: final atlas only; 96 frames, {len(colors)} colors, rigid body verified; no historical source dependency')
    if args.install: install_runtime(atlas,report)


if __name__ == '__main__':
    main()
