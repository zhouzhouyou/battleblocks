"""Bake the Rust landing's back/front layers and preview them on our dwarf atlas."""
from pathlib import Path
import json
import shutil
import subprocess
import tempfile
from PIL import Image, ImageDraw

ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'asset-sources/effects/heavy-landing/v1'
W,H,N,COLUMNS=176,80,22,7
with tempfile.TemporaryDirectory(prefix='battleblocks-landing-') as directory:
    binary=Path(directory)/'landing'
    subprocess.run([shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc'),'-O',str(SOURCE/'heavy-landing.rs'),'-o',str(binary)],check=True)
    raw=subprocess.check_output([str(binary)])
    lift=json.loads(subprocess.check_output([str(binary),'--motion']))
assert len(raw)==W*H*4*N*2
layers=[]
for layer in range(2):
    layers.append([Image.frombytes('RGBA',(W,H),raw[(layer*N+i)*W*H*4:(layer*N+i+1)*W*H*4]) for i in range(N)])
atlas=Image.new('RGBA',(W*COLUMNS,H*4*2))
for layer,frames in enumerate(layers):
    assert frames[-1].getextrema()[3]==(0,0)
    for i,frame in enumerate(frames):
        assert frame.crop((0,0,W,1)).getextrema()[3]==(0,0)
        assert frame.crop((0,H-1,W,H)).getextrema()[3]==(0,0)
        assert frame.crop((0,0,1,H)).getextrema()[3]==(0,0)
        assert frame.crop((W-1,0,W,H)).getextrema()[3]==(0,0)
        atlas.paste(frame,((i%COLUMNS)*W,(layer*4+i//COLUMNS)*H))
out=ROOT/'assets/effects/heavy-landing.webp'
atlas.save(out,lossless=True,exact=True,method=6)
with Image.open(out) as decoded: assert decoded.convert('RGBA').tobytes()==atlas.tobytes()
(SOURCE/'atlas.json').write_text(json.dumps(dict(width=W,height=H,frames=N,columns=COLUMNS,layer_rows=4,layers=['back','front'],fps=30,pivot=[88,60],landing_seconds=0.65,dust_duration=0.7,upstream='patch-gg/pixelfx@a84159688af3ca37a4be05a0d422b94451b2e24e',effect='impacts/heavy-landing'),indent=2)+'\n')

# Match the game's 208x184 actor slot, 384x384 frame, and (192,336) foot anchor.
dwarf=Image.open(ROOT/'assets/roles/dwarf.webp').convert('RGBA')
scale=min(208/84,184/83,3)*96/384
size=round(384*scale)
reviews=[]
for i,height in enumerate(lift):
    frame=Image.new('RGBA',(256,272),'#252633')
    ImageDraw.Draw(frame).line((16,228,240,228),fill='#535064')
    age=i/30-0.65
    index=int(age*30+0.00001) if age>=0 else -1
    if 0<=index<N:frame.alpha_composite(layers[0][index],(128-88,228-60))
    actor_index=8 if i/30<0.65 else int((i/30)/0.18)%4
    actor=dwarf.crop(((actor_index%4)*384,(actor_index//4)*384,(actor_index%4+1)*384,(actor_index//4+1)*384)).resize((size,size),Image.Resampling.NEAREST)
    frame.alpha_composite(actor,(round(128-size/2),round(228-336*scale-height)))
    if 0<=index<N:
        front=layers[1][index].copy()
        # The GUI clips dust above the passive row, 4 pixels below the standing foot.
        ImageDraw.Draw(front).rectangle((0,64,W,H),fill=(0,0,0,0))
        frame.alpha_composite(front,(128-88,228-60))
    reviews.append(frame.convert('RGB'))
reviews[0].save(SOURCE/'dwarf-preview.webp',save_all=True,append_images=reviews[1:],duration=33,loop=0,lossless=True,method=6)
contact=Image.new('RGB',(1024,3*292),'#161820')
draw=ImageDraw.Draw(contact)
for j,i in enumerate([0,6,10,15,19,20,22,25,28,32,37,42]):
    contact.paste(reviews[i],((j%4)*256,(j//4)*292+20))
    draw.text(((j%4)*256+6,(j//4)*292+4),f'{i/30:.2f}s',fill='#E2D4BC')
contact.save(SOURCE/'contact.webp',lossless=True,method=6)
print(f'{out.relative_to(ROOT)}: {out.stat().st_size} bytes; two layers, {N} frames each; lossless pixels, empty borders and ending verified')
