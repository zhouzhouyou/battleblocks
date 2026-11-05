"""Bake the deterministic Rust effect into a lossless WebP atlas and review previews.

No window or third-party JS runtime is needed. Source and previews stay out of packages.
Run with .build-tools/python-path.txt's Python (Pillow required).
"""
from pathlib import Path
import json
import math
import shutil
import subprocess
import tempfile
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'assets-source/effects/hammer-smash'
PREVIEWS=ROOT/'.local/previews/hammer_smash'
PREVIEWS.mkdir(parents=True,exist_ok=True)
WIDTH, HEIGHT, COUNT, COLUMNS, FPS = 192, 128, 28, 7, 30

with tempfile.TemporaryDirectory(prefix='battleblocks-hammer-') as directory:
    binary = Path(directory) / 'hammer-smash'
    subprocess.run([shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc'),
                    '-O', str(SOURCE/'hammer-smash.rs'), '-o', str(binary)], check=True)
    raw = subprocess.check_output([str(binary)])
assert len(raw) == WIDTH * HEIGHT * 4 * COUNT
frames = [Image.frombytes('RGBA', (WIDTH, HEIGHT), raw[i*WIDTH*HEIGHT*4:(i+1)*WIDTH*HEIGHT*4]) for i in range(COUNT)]
atlas = Image.new('RGBA', (WIDTH*COLUMNS, HEIGHT*((COUNT+COLUMNS-1)//COLUMNS)))
for i, frame in enumerate(frames):
    atlas.paste(frame, ((i%COLUMNS)*WIDTH, (i//COLUMNS)*HEIGHT))
out = ROOT/'assets/effects/hammer-smash.webp'
out.parent.mkdir(parents=True, exist_ok=True)
atlas.save(out, lossless=True, exact=True, method=6)
with Image.open(out) as decoded:
    assert decoded.convert('RGBA').tobytes() == atlas.tobytes(), 'WebP must preserve every pixel'
assert frames[-1].getextrema()[3] == (0, 0), 'effect must end empty'
for frame in frames:
    assert frame.crop((0,0,WIDTH,1)).getextrema()[3] == (0,0)
    assert frame.crop((0,HEIGHT-1,WIDTH,HEIGHT)).getextrema()[3] == (0,0)
    assert frame.crop((0,0,1,HEIGHT)).getextrema()[3] == (0,0)
    assert frame.crop((WIDTH-1,0,WIDTH,HEIGHT)).getextrema()[3] == (0,0)
(SOURCE/'atlas.json').write_text(json.dumps(dict(width=WIDTH, height=HEIGHT, frames=COUNT,
    columns=COLUMNS, fps=FPS, pivot=[96,88], impact_frame=12, impact_seconds=0.4,
    upstream='patch-gg/pixelfx@a84159688af3ca37a4be05a0d422b94451b2e24e',
    effect='melee/hammer-smash'), indent=2)+'\n')

# Review on our actual board textures at 1 logical pixel per image pixel.
board = Image.new('RGBA', (512,512), '#252633')
tiles = [Image.open(ROOT/'assets/ui'/name).convert('RGBA').resize((64,64),Image.Resampling.NEAREST)
         for name in ['balanced.webp','hp.webp','damage.webp','mp.webp','hd.webp','hm.webp','dm.webp']]
for y in range(8):
    for x in range(8):
        board.alpha_composite(tiles[(x+2*y)%7],(64*x,64*y))
review = []
for i, frame in enumerate(frames):
    canvas=board.copy()
    # Four cleared cells centered on (256,256); fall stays in the game, not this asset preview.
    for y in (3,4):
        for x in (3,4):
            ImageDraw.Draw(canvas).rectangle((x*64,y*64,x*64+63,y*64+63), fill='#252633')
            if i<12:
                canvas.alpha_composite(tiles[(x+2*y)%7],(x*64,y*64))
            elif i/FPS-0.4 < 0.36:
                # Same presentation parameters as src/effects/hammer_smash.rs::fragment.
                age=i/FPS-0.4
                alpha=min(1.,max(0.,(0.36-age)/0.16))
                for row in range(4):
                    for column in range(4):
                        px,py=x*64+column*16,y*64+row*16
                        dx,dy=px+8-256,py+8-256
                        length=math.hypot(dx,dy) or 1
                        seed=x*31+y*17+column*7+row*13
                        vx=dx/length*(90+abs(math.sin(seed))*90)
                        vy=-100-abs(math.cos(seed*1.7))*105+dy/length*75
                        piece=tiles[(x+2*y)%7].crop((column*16,row*16,column*16+16,row*16+16))
                        piece.putalpha(piece.getchannel('A').point(lambda value:round(value*alpha)))
                        canvas.alpha_composite(piece,(round(px+vx*age),round(py+vy*age+500*age*age)))
    canvas.alpha_composite(frame,(256-96,256-88))
    review.append(canvas.convert('RGB'))
review[0].save(PREVIEWS/'board-preview.webp',save_all=True,append_images=review[1:],
               duration=[33,33,34]*(COUNT//3)+[33]*(COUNT%3),loop=0,lossless=True,method=6)
contact=Image.new('RGB',(4*256,3*220),'#161820')
draw=ImageDraw.Draw(contact)
for j,i in enumerate([5,8,10,11,12,13,14,15,17,20,24,27]):
    crop=review[i].crop((128,144,384,336))
    contact.paste(crop,((j%4)*256,(j//4)*220+20))
    draw.text(((j%4)*256+6,(j//4)*220+4),f'{i/FPS:.2f}s',fill='#E2D4BC')
contact.save(PREVIEWS/'contact.webp',lossless=True,method=6)
print(f'{out.relative_to(ROOT)}: {out.stat().st_size} bytes, {COUNT} frames, lossless round-trip and empty borders verified')
