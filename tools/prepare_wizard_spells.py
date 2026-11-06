"""Bake pixel lightning/meteor atlas assets and fixed-board review previews."""
from pathlib import Path
import json
import shutil
import subprocess
import tempfile
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'assets-source/effects/wizard-spells'
PREVIEWS=ROOT/'.local/previews/wizard_spells'
PREVIEWS.mkdir(parents=True,exist_ok=True)
with tempfile.TemporaryDirectory(prefix='battleblocks-wizard-') as directory:
    binary=Path(directory)/'wizard-spells'
    subprocess.run([shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc'),'-O',str(SOURCE/'wizard-spells.rs'),'-o',str(binary)],check=True)
    raws=[subprocess.check_output([str(binary),'--lightning']),subprocess.check_output([str(binary)])]
    progress=[json.loads(subprocess.check_output([str(binary),'--lightning','--progress'])),json.loads(subprocess.check_output([str(binary),'--progress']))]
fixtures=json.loads((SOURCE/'review-targets.json').read_text())
tiles=[Image.open(ROOT/'assets/ui'/name).convert('RGBA') for name in ['balanced.webp','hp.webp','damage.webp','mp.webp','hd.webp','hm.webp','dm.webp']]
board=Image.new('RGBA',(512,512),'#252633')
for y in range(8):
    for x in range(8):
        ImageDraw.Draw(board).rectangle((x*64,y*64,x*64+63,y*64+63),fill='#535064',outline='#ACA0A1',width=2)
        board.alpha_composite(tiles[(x+2*y)%7],(x*64,y*64))
for name,count,columns,raw,states in zip(['lightning','meteor'],[29,29],[6,6],raws,progress):
    assert len(raw)==512*512*4*count
    frames=[Image.frombytes('RGBA',(512,512),raw[i*512*512*4:(i+1)*512*512*4]) for i in range(count)]
    atlas=Image.new('RGBA',(columns*512,((count+columns-1)//columns)*512))
    for i,frame in enumerate(frames):atlas.paste(frame,((i%columns)*512,(i//columns)*512))
    path=ROOT/f'assets/effects/wizard-{name}.webp'
    atlas.save(path,lossless=True,exact=True,method=6,quality=100)
    with Image.open(path) as check:assert check.convert('RGBA').tobytes()==atlas.tobytes()
    assert frames[-1].getextrema()[3]==(0,0)
    previews=[]
    targets={tuple(p) for p in fixtures[name]['cells']}
    for i,frame in enumerate(frames):
        image=board.copy()
        for x,y in targets:
            amount=states[i][y*8+x]
            ImageDraw.Draw(image).rectangle((x*64,y*64,x*64+63,y*64+63),fill='#535064',outline='#ACA0A1',width=2)
            if amount<1:
                size=round(64*(1-amount*0.12))
                tile=tiles[fixtures[name]['board'][y][x]].resize((size,size),Image.Resampling.NEAREST)
                tile.putalpha(tile.getchannel('A').point(lambda v:round(v*(1-amount))))
                image.alpha_composite(tile,(x*64+(64-size)//2,y*64+(64-size)//2))
        image.alpha_composite(frame);previews.append(image.convert('RGB'))
    previews[0].save(PREVIEWS/f'{name}-preview.webp',save_all=True,append_images=previews[1:],duration=100,loop=0,lossless=True,method=6)
    picks=[0,3,6,8,9,11,13,15,17,20,24,28]
    contact=Image.new('RGB',(1024,828),'#161820');draw=ImageDraw.Draw(contact)
    for j,i in enumerate(picks):
        contact.paste(previews[i].resize((256,256),Image.Resampling.NEAREST),((j%4)*256,(j//4)*276+20))
        draw.text(((j%4)*256+6,(j//4)*276+4),f'{i/10:.2f}s',fill='#E2D4BC')
    contact.save(PREVIEWS/f'{name}-contact.webp',lossless=True,method=6)
    (SOURCE/f'{name}.json').write_text(json.dumps(dict(width=512,height=512,frames=count,columns=columns,fps=10,impact_frame=9,impact_seconds=0.9,duration=2.8,loop=False),indent=2)+'\n')
    print(f'{path.relative_to(ROOT)}: {path.stat().st_size} bytes; {count} frames, lossless pixels and empty ending verified',flush=True)
