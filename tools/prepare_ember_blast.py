"""Render the standalone Rust board blast and original ZZZ bitmaps, without editing roles."""
from pathlib import Path
import json
import shutil
import subprocess
import tempfile
from PIL import Image,ImageDraw

ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'assets-source/effects/ember-blast'
PREVIEWS=ROOT/'.local/previews/ember_blast'
PREVIEWS.mkdir(parents=True,exist_ok=True)
SLEEP=ROOT/'assets-source/effects/sleep'
with tempfile.TemporaryDirectory(prefix='battleblocks-ember-') as directory:
    binary=Path(directory)/'ember'
    subprocess.run([shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc'),'-O',str(SOURCE/'ember-blast.rs'),'-o',str(binary)],check=True)
    fire_raw=subprocess.check_output([str(binary)])
    sleep_raw=subprocess.check_output([str(binary),'--sleep'])

def bake(raw,w,h,count,columns,path):
    assert len(raw)==w*h*4*count
    frames=[Image.frombytes('RGBA',(w,h),raw[i*w*h*4:(i+1)*w*h*4]) for i in range(count)]
    atlas=Image.new('RGBA',(w*columns,h*((count+columns-1)//columns)))
    for i,frame in enumerate(frames):atlas.paste(frame,((i%columns)*w,(i//columns)*h))
    atlas.save(path,lossless=True,exact=True,method=6)
    with Image.open(path) as check:assert check.convert('RGBA').tobytes()==atlas.tobytes()
    print(f'{path.relative_to(ROOT)}: {path.stat().st_size} bytes, {count} frames, lossless pixels verified',flush=True)
    return frames

fire=bake(fire_raw,512,512,22,6,ROOT/'assets/effects/ember-blast.webp')
sleep=bake(sleep_raw,96,96,36,6,ROOT/'assets/effects/sleep-zzz.webp')
assert fire[-1].getextrema()[3]==(0,0)
assert fire[6].getextrema()[3][0]>=200, 'peak must cover all 64 cells and corners'
(SOURCE/'atlas.json').write_text(json.dumps(dict(width=512,height=512,frames=22,columns=6,fps=15,impact_frame=6,impact_seconds=0.4,duration=1.4,pixel_scale=1),indent=2)+'\n')
(SLEEP/'atlas.json').write_text(json.dumps(dict(width=96,height=96,frames=36,columns=6,fps=15,loop_seconds=2.4,head_anchor=[24,70],font='original bitmap glyphs, no external font'),indent=2)+'\n')

tiles=[Image.open(ROOT/'assets/ui'/name).convert('RGBA') for name in ['balanced.webp','hp.webp','damage.webp','mp.webp','hd.webp','hm.webp','dm.webp']]
board=Image.new('RGBA',(512,512),'#252633')
for y in range(8):
    for x in range(8):
        ImageDraw.Draw(board).rectangle((x*64,y*64,x*64+63,y*64+63),fill='#535064',outline='#ACA0A1',width=2)
        board.alpha_composite(tiles[(x+2*y)%7],(x*64,y*64))
preview=[]
for effect in fire:
    image=board.copy();image.alpha_composite(effect);preview.append(image.convert('RGB'))
preview[0].save(PREVIEWS/'board-preview.webp',save_all=True,append_images=preview[1:],duration=[67,66,67]*7+[66],loop=0,lossless=True,method=6)
contact=Image.new('RGB',(1024,3*276),'#161820')
draw=ImageDraw.Draw(contact)
for j,i in enumerate([0,3,5,6,7,8,10,12,14,16,19,21]):
    contact.paste(preview[i].resize((256,256),Image.Resampling.NEAREST),((j%4)*256,(j//4)*276+20))
    draw.text(((j%4)*256+6,(j//4)*276+4),f'{i/15:.2f}s',fill='#E2D4BC')
contact.save(PREVIEWS/'contact.webp',lossless=True,method=6)

role=Image.open(ROOT/'assets/roles/fire-elf/runtime/atlas.webp').convert('RGBA')
images=[]
for i,effect in enumerate(sleep):
    canvas=Image.new('RGBA',(256,272),'#252633')
    index=12+int(i/15/0.18)%4
    sprite=role.crop(((index%4)*384,(index//4)*384,(index%4+1)*384,(index//4+1)*384)).resize((213,213),Image.Resampling.NEAREST)
    canvas.alpha_composite(sprite,(22,round(248-336*(184/83*96/384))))
    canvas.alpha_composite(effect,(128-12-24,248-84-70))
    images.append(canvas.convert('RGB'))
images[0].save(PREVIEWS/'elf-preview.webp',save_all=True,append_images=images[1:],duration=[67,66,67]*12,loop=0,lossless=True,method=6)
