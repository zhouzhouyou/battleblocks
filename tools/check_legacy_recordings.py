"""Match rendered actor poses in the shared 60 Hz recording sample stream.

This observes pixels, not animation timers. It does not infer a model hit from a
cast pose: real skill_hit / resolution stages are listed separately from JSONL.
"""
from pathlib import Path
import argparse, hashlib, json, sys
ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/'.build-tools/art-site'),str(ROOT/'.build-tools/media-site')]
import numpy as np
from PIL import Image, ImageDraw
import imageio_ffmpeg
from wizard_recording_frames import frames, FPS

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--label',default='current')
args=parser.parse_args()
OUT=ROOT/'analysis/issue28-awake-animation'/args.label
capture=json.loads((OUT/'capture.json').read_text(encoding='utf-8'))
FF=imageio_ffmpeg.get_ffmpeg_exe()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
report={'source_commit':capture['source_commit'],'binary_sha256':capture['binary_sha256'],
        'sampling_fps':FPS,'sampling':'wizard_recording_frames.frames, sample n is n/60 seconds','scenes':{}}
for name,scene in capture['scenes'].items():
    if '-awake-' not in name or '-menu' in name:continue
    role=name.split('-')[0]
    enemy=name.endswith('-enemy')
    origin=980 if enemy else 0
    atlas=Image.open(ROOT/'assets/roles'/f'{role}-legacy.webp').convert('RGBA')
    scale=184/83/4*1.25
    left,top=((900 if enemy else 124)-192*184/83/4)*1.25,(372-336*184/83/4)*1.25
    xs=np.floor((np.arange(300)+origin+.5-left)/scale).astype(int)
    ys=np.floor((np.arange(275)+205+.5-top)/scale).astype(int)
    valid=(ys[:,None]>=0)&(ys[:,None]<384)&(xs[None,:]>=0)&(xs[None,:]<384)
    templates=[]
    for i in range(24):
        cell=np.array(atlas.crop((i%6*384,i//6*384,i%6*384+384,i//6*384+384)))
        rendered=cell[ys.clip(0,383)[:,None],xs.clip(0,383)[None,:]].copy()
        rendered[:,:,3]*=valid
        templates.append(rendered)
    templates=np.array(templates);mask=templates[:,:,:,3].max(axis=0)>0
    grid=np.zeros_like(mask);grid[::3,::3]=True;mask&=grid
    # Health popups and ability-name banners occupy the top of the actor slot.
    # Match the unobscured face/arms/torso/legs, not their changing overlay text.
    mask[:125,:]=False
    refs=templates[:,:,:,:3][:,mask].astype(np.float32)
    alpha=templates[:,:,:,3][:,mask].astype(np.float32)/255
    assert sha(OUT/scene['video'])==scene['video_sha256']
    assert sha(OUT/scene['log'])==scene['log_sha256']
    samples=[];pictures={};bg=None
    for n,raw in enumerate(frames(FF,OUT/scene['video'])):
        screen=np.frombuffer(raw,np.uint8).reshape(900,1280,3)
        if str(n) in scene['samples_60hz']:
            image=Image.fromarray(screen)
            assert hashlib.sha256(image.convert('RGBA').tobytes()).hexdigest()==scene['samples_60hz'][str(n)]['rgba_sha256']
            assert Image.open(OUT/scene['samples_60hz'][str(n)]['path']).convert('RGB').tobytes()==raw
        observed=screen[205:480,origin:origin+300][mask].astype(np.float32)
        background=screen[260,1260 if enemy else 20].astype(np.float32)
        expected=refs*alpha[:,:,None]+background*(1-alpha[:,:,None])
        # Discard the worst 20% of pixels (transient healing/skill overlays).
        # The remaining dense interior samples still distinguish every pose.
        per_pixel=np.mean((expected-observed)**2,axis=2)
        errors=np.mean(np.sort(per_pixel,axis=1)[:,:int(per_pixel.shape[1]*.8)],axis=1)
        pose=int(np.argmin(errors))
        border=screen[252:255,origin+30:origin+280]
        # H.264 chroma subsampling softens the one-pixel gold outline.
        gold=((border[:,:,0]>170)&(border[:,:,1]>130)&(border[:,:,2]<170)).mean(axis=1).max()
        samples.append({'sample_60hz':n,'seconds':n/60,'atlas_pose':pose,'mse':round(float(errors[pose]),2),
                        'hit_reaction_border':round(float(gold),3)})
        if 12<=pose<24 and n>=45 and errors[pose]<1500 and pose not in pictures:
            pictures[pose]=screen.copy()
    casts=[s for s in samples if s['sample_60hz']>=45 and s['atlas_pose']>=12 and s['mse']<1500]
    seen=sorted({s['atlas_pose'] for s in casts})
    entry={'observed_cast_poses':seen,'samples':samples,
        'real_model_stages':scene.get('events_after_skill',[]),'exported_samples_pixel_identical':True}
    if casts:
        start=casts[0]['sample_60hz'];end=casts[-1]['sample_60hz']
        entry.update(first_cast_sample=start,last_cast_sample=end,observed_cast_span_seconds=(end-start+1)/60)
    timing_cases=['knight-awake-charge','knight-awake-guard','goblin-awake-greedy',
                  'goblin-awake-greedy-delayed','knight-awake-charge-delayed','knight-awake-charge-pause']
    if name in timing_cases:
        starts=[s['sample_60hz'] for s in casts if s['atlas_pose']==12]
        hits=[s['sample_60hz'] for s in casts if s['atlas_pose']==18 and starts and s['sample_60hz']>starts[0]]
        stage='skill_hit' if 'charge' in name else 'resolution'
        actual=next(e for e in scene['events_after_skill'] if e['event']==stage and not e.get('enemy_turn',False))
        if starts and hits:
            delta=(hits[0]-starts[0])/60-actual['delta']
            entry['relative_stage_alignment']={'stage':stage,'start_pose_sample':starts[0],
                'key_pose_sample':hits[0],'model_delta_seconds':actual['delta'],
                'pixel_delta_seconds':(hits[0]-starts[0])/60,'difference_seconds':delta,
                'within_3_samples':abs(delta)<=3/60,
                'note':'Relative to first visible cast pose; not an absolute event-to-video clock calibration.'}
            if stage=='skill_hit':
                visible=[s['sample_60hz'] for s in samples if s['sample_60hz']>=starts[0] and s['hit_reaction_border']>.8]
                if visible:
                    entry['visible_hit_alignment']={'reaction_sample':visible[0],'actor_key_sample':hits[0],
                        'delta_samples':hits[0]-visible[0],
                        'landmark':'Charge Mastery reaction gold border, logical rect (22,202,204,30); actual damage handler creates it.'}
    # Primary uninterrupted captures must show the full twelve-pose gesture.
    primary=name in [f'{r}-awake-{s}' for r,ss in [('knight',['charge','guard']),('elf',['strike','nature']),('goblin',['greedy','blood'])] for s in ss]
    entry['primary_full_pose_check']=not primary or seen==list(range(12,24))
    report['scenes'][name]=entry
    contact=Image.new('RGB',(6*300,2*300),'#20232d');draw=ImageDraw.Draw(contact)
    for pose,screen in pictures.items():
        x,y=(pose-12)%6*300,(pose-12)//6*300
        contact.paste(Image.fromarray(screen[205:480,origin:origin+300]),(x,y+25));draw.text((x+8,y+7),f'cast {pose-11}',fill='white')
    contact.save(OUT/f'{name}-observed-poses.webp',lossless=True,method=0)
    print(name,seen,entry['primary_full_pose_check'],flush=True)
(OUT/'pixel-measurements.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
assert all(v['primary_full_pose_check'] for v in report['scenes'].values()), 'Inspect failed pose coverage before acceptance'
