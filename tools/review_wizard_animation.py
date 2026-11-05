"""Export atlas inspection and unscaled excerpts from the real game recordings."""
from pathlib import Path
import argparse
import io
import json
import subprocess
import sys

from prepare_wizard_assets import ROOT, SOURCE, CLIPS, Image, save
from PIL import ImageDraw
from PIL import ImageChops, ImageStat
from contextlib import closing
from wizard_recording_frames import frames, WIDTH, HEIGHT

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--directory', type=Path, default=ROOT / '.local/analysis/issue27-wizard-animation')
OUT = parser.parse_args().directory.resolve()
OUT.mkdir(parents=True, exist_ok=True)
BASELINE = ROOT / '.local/analysis/issue27-wizard-animation/baseline'
sys.path.insert(0, str(ROOT / '.build-tools/media-site'))
import imageio_ffmpeg
FFMPEG = imageio_ffmpeg.get_ffmpeg_exe()
from asset_paths import report_path
from prepare_wizard_assets import import_wizard
if not report_path('wizard').exists(): import_wizard()
meta = json.loads(report_path('wizard').read_text())
atlas = Image.open(ROOT / 'assets/roles/wizard/runtime/atlas.webp').convert('RGBA')


def extract(path, seconds):
    raw = subprocess.check_output([FFMPEG, '-hide_banner', '-loglevel', 'error', '-ss', str(seconds),
        '-i', str(path), '-frames:v', '1', '-f', 'image2pipe', '-vcodec', 'png', '-'])
    return Image.open(io.BytesIO(raw)).convert('RGB')


def extract_frame(path, index):
    # VFR encoded-frame indices differ from the 60 Hz measurement samples.
    # Use exactly the same sampling stream as check_wizard_recordings.
    with closing(frames(FFMPEG, path)) as samples:
        for sample_index, raw in enumerate(samples):
            if sample_index == index:
                return Image.frombytes('RGB', (WIDTH, HEIGHT), raw)
    raise IndexError(f'{path}: sample {index} not present')


# These are actual unscaled 1280x900 window captures, not a reconstruction of the menu.
for form in ('normal', 'awake'):
    panel = Image.new('RGB', (610, 680), '#20232d')
    draw = ImageDraw.Draw(panel)
    baseline = extract(BASELINE / f'{form}-menu.mp4', 1.)
    current = extract(OUT / 'current' / f'{form}-menu.mp4', 1.)
    if form == 'normal':
        # The old four-pose loop has arbitrary wall-clock phase. Compare the
        # SAME actual pose, not different phases at equal video timestamps.
        region = (355,190,660,485)
        reference = baseline.crop(region)
        choices = []
        for t in (.05,.15,.25,.35,.45,.55):
            shot = extract(OUT/'current'/'normal-menu.mp4',t)
            difference = ImageStat.Stat(ImageChops.difference(reference,shot.crop(region)))
            error = sum(v*v for v in difference.rms)/3
            choices.append((error,t,shot))
        error,t,current = min(choices,key=lambda item:item[0])
        (OUT/'normal-menu-pixel-comparison.json').write_text(json.dumps({
            'baseline_video_seconds':1.,'current_video_seconds':t,'portrait_region':region,
            'mean_squared_rgb_error':error,'note':'Actual captures, same pose; MP4 compression can differ.'},indent=2)+'\n')
    for col, label in enumerate(('baseline', 'current')):
        capture = baseline if label == 'baseline' else current
        if label != 'baseline':
            save(capture, OUT / label / f'{form}-menu.webp')
        panel.paste(capture.crop((355, 140, 660, 788)), (col * 305, 32))
        draw.text((col * 305 + 12, 10), f'{label} / {form} / 1:1 capture', fill='white')
    save(panel, OUT / f'{form}-menu-comparison.webp')

contact = Image.new('RGB', (6*192, 4*216), '#20232d')
draw = ImageDraw.Draw(contact)
for index in range(24):
    x, y = index % 6 * 384, index // 6 * 384
    frame = atlas.crop((x,y,x+384,y+384)).resize((192,192), Image.Resampling.NEAREST)
    px, py = index % 6 * 192, index // 6 * 216
    draw.text((px+5,py+4), f'{CLIPS[index//12]} {index%12+1:02}', fill='white')
    contact.paste(frame, (px,py+20), frame)
    draw.line((px,py+188,px+192,py+188), fill='#53746a')
save(contact, OUT / 'atlas-contact.webp')

# Native-size loop seam: same 230x190/140 logical slot scale as the menu, no magnification.
seams = Image.new('RGB', (4*290, 280), '#20232d')
draw = ImageDraw.Draw(seams)
for row, clip in enumerate(('awake-idle',)):
    scale = min(230/84, 140/83, 3) / 4 * 1.25
    size = round(384 * scale)
    for col, pose in enumerate((10,11,0,1)):
        index = row*12+pose
        frame = atlas.crop((index%6*384,index//6*384,index%6*384+384,index//6*384+384))
        frame = frame.resize((size,size), Image.Resampling.NEAREST)
        foot = row*280+250
        px = col*290+145-size//2
        py = foot-round(336*scale)
        seams.paste(frame,(px,py),frame)
        draw.line((col*290,foot,(col+1)*290,foot),fill='#70a89b')
        draw.text((col*290+12,row*280+8),f'{clip} pose {pose+1}',fill='white')
save(seams, OUT/'idle-seams.webp')

for scene in ('normal-lightning','normal-meteor','awake-lightning','awake-meteor','awake-delayed-meteor','awake-terminal-meteor','awake-rollback'):
    # Timestamps refer to video time; event-aligned timings remain in capture.json/JSONL.
    times = [0.7,1.1,1.35,1.65,1.9,2.1,2.4,2.6,2.9,3.3,3.8,4.2]
    strip = Image.new('RGB',(4*320,3*300),'#20232d')
    draw = ImageDraw.Draw(strip)
    for i,t in enumerate(times):
        shot=extract(OUT/'current'/f'{scene}.mp4',t)
        # Character and feet remain at recorded device-pixel size.
        x,y=i%4*320,i//4*300
        strip.paste(shot.crop((0,205,300,480)),(x,y+24))
        draw.text((x+8,y+5),f'{scene} / video {t:.2f}s',fill='white')
    save(strip,OUT/f'{scene}-keys.webp')
    # Only label a hit after locating it in actual pixels; victory/rollback do
    # not have an unobscured matching hit and retain the neutral sample label.
    measurements = json.loads((OUT/'recording-measurements.json').read_text())
    if scene in measurements:
        hit_frame = measurements[scene]['effect_impact_frame']
        keys = [('guide', hit_frame-1), ('impact', hit_frame)]
    else:
        keys = [('sample', 126)]
    for tag,frame_index in keys:
        save(extract_frame(OUT/'current'/f'{scene}.mp4',frame_index),OUT/'current'/f'{scene}-{tag}.webp')

page='''<!doctype html><meta charset="utf-8"><title>老巫师动作验收 · #27</title>
<style>body{background:#191d28;color:#edf0f8;font:16px system-ui;margin:28px;max-width:1100px}button,select,input{font:inherit;margin:6px}canvas{image-rendering:pixelated;border:1px solid #465567}video{width:100%;max-width:900px}small{color:#b6c4d5}a{color:#83cfff}</style>
<h1>老巫师 · 觉醒两组十二姿势 · 普通版未改</h1><p>图集采样检查（非实战录像）。角色、帧时长来自实际运行时资产与导入报告。单次施法结束后进入待机；命中延迟时停留引导姿势。</p>
<select id="clip"></select><select id="slot"><option value="menu">实际选角尺寸</option><option value="battle">实际战斗尺寸</option><option value="source">原帧尺寸</option></select>
<button id="replay">重播</button><button id="pause">暂停/继续</button><label>实际命中 <input id="hit" type="number" value="0.9" min="0.9" max="3" step="0.1">s</label><br>
<input id="seek" type="range" min="0" max="4000" value="0"><output id="readout"></output><br><canvas id="view" width="420" height="430"></canvas>
<p><a href="atlas-contact.webp">觉醒24帧</a> · <a href="idle-seams.webp">待机11→12→1→2及脚线</a> · <a href="normal-menu-comparison.webp">普通同尺寸前后对比</a> · <a href="awake-menu-comparison.webp">觉醒同尺寸前后对比</a></p>
<h2>实战录像 · 真实游戏窗口</h2><p>固定棋盘通过原生控制台载入；施法、觉醒、清除和回滚均运行原游戏 Session。原始事件与录像同目录。录制目标60fps、1280×900设备像素，源视频为可变帧率；同步按统一60Hz采样。</p>
VIDEOS
<script>const meta=METADATA;const img=new Image();img.src='../../assets/roles/wizard/runtime/atlas.webp';const select=document.querySelector('#clip'),canvas=document.querySelector('#view'),ctx=canvas.getContext('2d');for(const name of Object.keys(meta.clips)){select.add(new Option(name,name))}let elapsed=0,last=performance.now(),playing=true;
document.querySelector('#replay').onclick=()=>{elapsed=0;playing=true};document.querySelector('#pause').onclick=()=>playing=!playing;select.onchange=()=>elapsed=0;document.querySelector('#seek').oninput=e=>{elapsed=+e.target.value;playing=false};
function sample(clip,t){if(clip.playback==='loop')t%=clip.duration_ms;for(let i=0;i<12;i++){if(t<clip.durations_ms[i])return clip.indices[i];t-=clip.durations_ms[i]}return clip.indices[11]}
function draw(now){if(playing)elapsed+=now-last;last=now;const name=select.value,awake=name.startsWith('awake');let clip=meta.clips[name],t=elapsed;const hit=+document.querySelector('#hit').value*1000;if(name.endsWith('cast')){t=elapsed<hit?Math.min(elapsed,833.333):900+elapsed-hit;if(t>=1400){clip=meta.clips[awake?'awake-idle':'normal-idle'];t-=1400}}const index=sample(clip,t);ctx.fillStyle='#191d28';ctx.fillRect(0,0,420,430);ctx.imageSmoothingEnabled=false;const mode=document.querySelector('#slot').value;let scale=mode==='source'?1:mode==='battle'?Math.min(208/84,184/83,3)/4:Math.min(230/84,(awake?140:190)/83,3)/4;let foot=mode==='source'?386:340;ctx.drawImage(img,index%6*384,Math.floor(index/6)*384,384,384,210-192*scale,foot-336*scale,384*scale,384*scale);ctx.strokeStyle='#70a89b';ctx.beginPath();ctx.moveTo(0,foot);ctx.lineTo(420,foot);ctx.stroke();document.querySelector('#readout').textContent=(elapsed/1000).toFixed(3)+'s / atlas '+index+' / pose '+(index%12+1);document.querySelector('#seek').value=Math.min(4000,elapsed);requestAnimationFrame(draw)}img.onload=()=>requestAnimationFrame(draw);</script>'''
videos=''
for scene in ('normal-menu','awake-menu','normal-lightning','normal-meteor','awake-lightning','awake-meteor','awake-delayed-meteor','awake-terminal-meteor','awake-rollback'):
    videos+=f'<h3>{scene}</h3><video controls preload="metadata" src="current/{scene}.mp4"></video><p><a href="current/{scene}.jsonl">事件日志</a></p>'
import os
atlas_url = os.path.relpath(ROOT/'assets/roles/wizard/runtime/atlas.webp', OUT).replace('\\','/')
(OUT/'index.html').write_text(page.replace('../../assets/roles/wizard/runtime/atlas.webp',atlas_url).replace('METADATA',json.dumps(meta)).replace('VIDEOS',videos),encoding='utf-8')
print(f'Review evidence: {OUT}')
