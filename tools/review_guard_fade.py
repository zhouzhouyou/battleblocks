"""Export review RED/GREEN samples through the same explicit 60 Hz stream."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/'.build-tools/art-site'),str(ROOT/'.build-tools/media-site')]
from PIL import Image,ImageDraw
import imageio_ffmpeg
from wizard_recording_frames import frames
OUT=ROOT/'analysis/issue28-awake-animation'
wanted=[119,125,129,130,131,132]
sheet=Image.new('RGB',(1800,600),'#20232d');draw=ImageDraw.Draw(sheet)
for row,label in enumerate(['final','review-fix']):
    for n,raw in enumerate(frames(imageio_ffmpeg.get_ffmpeg_exe(),OUT/label/'knight-awake-guard.mp4')):
        if n in wanted:
            x,y=wanted.index(n)*300,row*300
            sheet.paste(Image.frombytes('RGB',(1280,900),raw).crop((0,205,300,480)),(x,y+25))
            draw.text((x+6,y+5),f'{label} / sample {n}',fill='white')
sheet.save(OUT/'review-fix/guard-fade-before-after.webp',lossless=True,method=0)
print('RED/GREEN samples exported; visual inspection required.')
