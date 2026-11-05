"""Measure actor pose and spell impact in actual recorded pixels, not timers.

Review-only dependencies: Pillow 13, numpy 2.4, imageio-ffmpeg 0.6. This probe
matches the native 125% DPI captures; it never creates game state or rewards.
"""
import json
import argparse
import subprocess
import sys
from pathlib import Path
from prepare_wizard_assets import ROOT, Image, save
from PIL import ImageDraw
sys.path.insert(0, str(ROOT / '.build-tools/media-site'))
import numpy as np
import imageio_ffmpeg

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--directory', type=Path, default=ROOT / 'analysis/issue27-wizard-animation')
OUT = parser.parse_args().directory.resolve()
FFMPEG = imageio_ffmpeg.get_ffmpeg_exe()
atlas = Image.open(ROOT / 'assets/roles/wizard.webp').convert('RGBA')
scale = 184 / 83 / 4 * 1.25
left, top = (124 - 192 * (184 / 83 / 4)) * 1.25, (372 - 336 * (184 / 83 / 4)) * 1.25
# Pixel-centre nearest-neighbour projection of the same runtime atlas rectangle.
xs = np.floor((np.arange(300) + .5 - left) / scale).astype(int)
ys = np.floor((np.arange(275) + 205 + .5 - top) / scale).astype(int)
valid = (ys[:,None]>=0)&(ys[:,None]<384)&(xs[None,:]>=0)&(xs[None,:]<384)
templates=[]
for index in range(48):
    cell=np.array(atlas.crop((index%6*384,index//6*384,index%6*384+384,index//6*384+384)))
    rendered=cell[ys.clip(0,383)[:,None], xs.clip(0,383)[None,:]].copy()
    rendered[:,:,3]*=valid
    templates.append(rendered)
templates=np.array(templates)
mask=(templates[:,:,:,3].max(axis=0)>0)
# Sparse interior samples are enough to distinguish all poses without UI pixels.
grid=np.zeros_like(mask);grid[::3,::3]=True;mask &= grid
references=templates[:,:,:,:3][:,mask].astype(np.float32)
alphas=templates[:,:,:,3][:,mask].astype(np.float32)/255
report={}
# Victory overlays obscure the spell landmarks. Its no-clear recovery is checked
# by the clock test and terminal recording, not counted as pixel-sync evidence.
for scene in ('normal-lightning','normal-meteor','awake-lightning','awake-meteor','delayed-meteor'):
    kind='lightning' if 'lightning' in scene else 'meteor'
    fx=Image.open(ROOT/f'assets/effects/wizard-{kind}.webp').convert('RGBA')
    def effect_frame(i):
        return np.array(fx.crop((i%6*512,i//6*512,i%6*512+512,i//6*512+512)))
    hit,pre=effect_frame(9),effect_frame(8)
    # Opaque hot pixels that are absent before impact; these survive video encoding.
    points=np.argwhere((hit[:,:,3]>245)&(hit[:,:,0]>220)&(hit[:,:,1]>200)&(pre[:,:,3]<60))
    points=points[::max(1,len(points)//150)]
    if len(points)<10:raise AssertionError(f'insufficient impact landmarks: {kind}')
    fy=np.floor((110+points[:,0]+.5)*1.25).astype(int)
    fxpos=np.floor((256+points[:,1]+.5)*1.25).astype(int)
    colors=hit[points[:,0],points[:,1],:3].astype(float)
    process=subprocess.Popen([FFMPEG,'-hide_banner','-loglevel','error','-i',str(OUT/'current'/f'{scene}.mp4'),
        '-f','rawvideo','-pix_fmt','rgb24','-'],stdout=subprocess.PIPE)
    samples=[];selected=[]
    awake=scene.startswith('awake')
    candidates=list(range(12,24) if awake else range(12))+list(range(36,48) if awake else range(24,36))
    index=0
    while True:
        raw=process.stdout.read(1280*900*3)
        if not raw:break
        if len(raw)!=1280*900*3:raise AssertionError('partial decoded frame')
        screen=np.frombuffer(raw,dtype=np.uint8).reshape(900,1280,3)
        actor=screen[205:480,:300]
        observed=actor[mask].astype(np.float32)
        background=screen[260,20].astype(np.float32)
        expected=references[candidates]*alphas[candidates,:,None]+background*(1-alphas[candidates,:,None])
        errors=np.mean((expected-observed)**2,axis=(1,2))
        pose=candidates[int(np.argmin(errors))]
        confidence=float(np.mean(np.max(np.abs(screen[fy,fxpos].astype(float)-colors),axis=1)<45))
        samples.append({'frame':index,'seconds':index/60,'atlas_pose':pose,'pose_mse':round(float(errors.min()),2),'impact_match':round(confidence,3)})
        if .8<index/60<3.5:
            selected.append((index,Image.fromarray(screen.copy())))
        index+=1
    if process.wait():raise RuntimeError('decoder failed')
    hit_frames=[s['frame'] for s in samples if s['seconds']>.8 and s['impact_match']>.7]
    actor_frames=[s['frame'] for s in samples if s['seconds']>.8 and s['atlas_pose']==(43 if awake else 31)]
    if not hit_frames or not actor_frames:
        raise AssertionError(f'{scene}: impact not observable; inspect landmarks/templates')
    first_hit,first_actor=hit_frames[0],actor_frames[0]
    delta=first_actor-first_hit
    report[scene]={'frames':index,'actor_impact_frame':first_actor,'effect_impact_frame':first_hit,
                   'delta_frames':delta,'delta_ms':round(delta/60*1000,3),
                   'impact_landmarks':len(points),'samples':samples}
    keys=[first_hit-1,first_hit,first_hit+1,first_hit+6]
    contact=Image.new('RGB',(1280,948),'#20232d');draw=ImageDraw.Draw(contact)
    for i,frame_index in enumerate(keys):
        shot=next(im for f,im in selected if f==frame_index)
        # Overview resized only for the contact; MP4 and side-panel keys stay native size.
        contact.paste(shot.resize((640,450),Image.Resampling.NEAREST),(i%2*640,i//2*474+24))
        sample=samples[frame_index]
        draw.text((i%2*640+8,i//2*474+6),f'{scene} | video frame {frame_index} | pose {sample["atlas_pose"]%12+1} | fx {sample["impact_match"]:.2f}',fill='white')
    save(contact,OUT/f'{scene}-synchronization.webp')
    print(f'{scene}: actor={first_actor}, effect={first_hit}, delta={delta} frames',flush=True)
    if abs(delta)>1:raise AssertionError(f'{scene}: actor/effect impact differ by {delta} frames')
(OUT/'recording-measurements.json').write_text(json.dumps(report,indent=2)+'\n')
