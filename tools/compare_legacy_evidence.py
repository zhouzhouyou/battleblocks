"""Assemble labelled crops from real recordings at original device-pixel scale."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/'.build-tools/art-site'),str(ROOT/'.build-tools/media-site')]
from PIL import Image,ImageDraw,ImageChops,ImageStat
import imageio_ffmpeg
from wizard_recording_frames import frames
OUT=ROOT/'analysis/issue28-awake-animation'
FF=imageio_ffmpeg.get_ffmpeg_exe()
def sample(video,n):
    result=None
    for i,raw in enumerate(frames(FF,video)):
        if i==n:result=Image.frombytes('RGB',(1280,900),raw)
    assert result is not None
    return result
def panel(pairs,path,width,height):
    sheet=Image.new('RGB',(width*len(pairs),height+28),'#20232d');draw=ImageDraw.Draw(sheet)
    for i,(name,im) in enumerate(pairs):
        sheet.paste(im,(i*width,28));draw.text((i*width+8,8),name,fill='white')
    sheet.save(path,lossless=True,method=0)
final=json.loads((OUT/'final/capture.json').read_text(encoding='utf-8'))
baseline=json.loads((OUT/'baseline/capture.json').read_text(encoding='utf-8'))
menu_box=(355,190,660,485)
menu=[]
for role in ['knight','elf','goblin']:
    before=Image.open(OUT/'research'/f'{role}-menu.webp').convert('RGB').crop(menu_box)
    after=Image.open(OUT/'final'/f'{role}-awake-menu-030.webp').convert('RGB').crop(menu_box)
    panel([('baseline awake',before),('current awake',after)],OUT/f'{role}-awake-menu-before-after.webp',305,295)
    menu.append((role+' awake',after))
    oldshots=[Image.open(OUT/'baseline'/s['path']).convert('RGB').crop(menu_box) for s in baseline['scenes'][f'{role}-normal-menu']['samples_60hz'].values()]
    newshots=[Image.open(OUT/'final'/s['path']).convert('RGB').crop(menu_box) for s in final['scenes'][f'{role}-normal-menu']['samples_60hz'].values()]
    _,old,new=min((sum(ImageStat.Stat(ImageChops.difference(a,b)).mean),i,j) for i,a in enumerate(oldshots) for j,b in enumerate(newshots))
    panel([('baseline normal',oldshots[old]),('current normal',newshots[new])],OUT/f'{role}-normal-menu-before-after.webp',305,295)
    skill={'knight':'charge','elf':'nature','goblin':'greedy'}[role]
    old=sample(OUT/'research'/f'{role}-{skill}.mp4',180).crop((0,205,300,480))
    new=Image.open(OUT/'final'/f'{role}-awake-{skill}-210.webp').convert('RGB').crop((0,205,300,480))
    panel([('baseline awake / battle',old),('current awake / battle',new)],OUT/f'{role}-awake-battle-before-after.webp',300,275)
newknight=Image.open(OUT/'final/newknight-awake-menu-030.webp').convert('RGB').crop(menu_box)
panel([menu[0],('new knight awake',newknight),menu[1],menu[2]],OUT/'awake-menu-lineup.webp',305,295)
print('Actual menu/battle crops written; comparisons are not rescaled.')
