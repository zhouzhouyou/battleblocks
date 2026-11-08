"""Produce review contact sheets and quantitative checks; does not certify art."""
from pathlib import Path
import hashlib
import json
import sys
import argparse

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'.build-tools/art-site'))
from PIL import Image, ImageDraw, ImageChops
from prepare_legacy_awake_assets import ROLES, source_dir
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--role', action='append', choices=ROLES, help='Review only these roles; default: all legacy imports')
parser.add_argument('--out', type=Path, default=ROOT/'.local/analysis/legacy-awake/art')
args = parser.parse_args()
OUT = args.out
OUT.mkdir(parents=True, exist_ok=True)
report = {}
for role in args.role or ROLES:
    im = Image.open(ROOT/'assets/roles'/role/'runtime/atlas.webp').convert('RGBA')
    registration = json.loads((source_dir(role)/'registration.json').read_text())
    frames = [im.crop((i%6*384, i//6*384, i%6*384+384, i//6*384+384)) for i in range(24)]
    assert im.size == (2304,1536)
    assert len({f.tobytes() for f in frames}) == 24
    # Cropping each pose to its own bounds removes translations from this check.
    assert len({(f.crop(f.getbbox()).size, f.crop(f.getbbox()).tobytes()) for f in frames}) == 24
    assert set(im.getchannel('A').get_flattened_data()) == {0,255}
    assert len({p[:3] for p in im.get_flattened_data() if p[3]}) <= 64
    checks=[]
    sheet=Image.new('RGB',(6*192,4*214),'#303744');draw=ImageDraw.Draw(sheet)
    for i,f in enumerate(frames):
        b=f.getbbox()
        assert b[0]>=4 and b[1]>=4 and b[2]<=380 and b[3]<=380
        clip = 'awake-idle' if i < 12 else 'awake-cast'
        if not registration[clip].get('explicit_sole'):
            assert b[3]==336
        foot=f.getchannel('A').crop((0,328,384,336)).getbbox()
        checks.append({'bounds':b,'foot_span_bottom8':foot,'sha256':hashlib.sha256(f.tobytes()).hexdigest()})
        # Menu production scale is 140/83*96/384 = .421687, not enlarged art.
        small=f.resize((162,162),Image.Resampling.NEAREST)
        x,y=i%6*192,i//6*214
        sheet.paste(small,(x+15,y+22),small)
        draw.line((x,y+22+round(336*162/384),x+191,y+22+round(336*162/384)),fill='#647080')
        draw.text((x+8,y+4),f'{"idle" if i<12 else "cast"} {i%12+1:02}',fill='white')
    sheet.save(OUT/f'{role}-poses.webp',lossless=True,method=6)
    # Seam is viewed in its actual cyclic order, independent of hash uniqueness.
    seam=Image.new('RGB',(4*192,214),'#303744')
    for j,i in enumerate([10,11,0,1]):
        f=frames[i].resize((162,162),Image.Resampling.NEAREST)
        seam.paste(f,(j*192+15,22),f)
    seam.save(OUT/f'{role}-seam.webp',lossless=True,method=6)
    diffs=[]
    for a,b in zip(frames[:12],frames[1:12]+frames[:1]):
        diffs.append(sum(p!=(0,0,0,0) for p in ImageChops.difference(a,b).get_flattened_data()))
    report[role]={'unique_poses':24,'not_whole_image_translations':True,'binary_alpha':True,
        'palette_colors':len({p[:3] for p in im.get_flattened_data() if p[3]}),
        'idle_adjacent_changed_pixels':diffs,'frames':checks,
        'visual_status':'Requires actual pose, equipment, seam and foot-span review; hashes alone are insufficient.'}
(OUT/'checks.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(f'{len(report)*24} unique poses, no whole-image translation duplicates, palettes/bounds/alpha passed; contact sheets written')
