"""Capture the actual Windows game; logs and pixels share a fixed 60 Hz stream.

Run one recorder at a time. --baseline uses the preserved verified PR31 binary.
Scene names: knight|elf|goblin[-awake|-normal]-menu or role-form-skill[-case].
Cases: delayed, terminal, rollback, pending-rollback, no-target, full, followup,
pause, switch. This drives the existing console, never simulates game rules.
"""
from pathlib import Path
import argparse, hashlib, json, os, subprocess, sys, time

ROOT = Path(__file__).resolve().parents[1]
sys.path[:0] = [str(ROOT/'.build-tools/art-site'), str(ROOT/'.build-tools/media-site')]
from PIL import Image
import imageio_ffmpeg
from wizard_recording_frames import frames, FPS

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--label', required=True)
parser.add_argument('--binary', type=Path, default=ROOT/'target/debug/battleblocks.exe')
parser.add_argument('--scenes', required=True)
parser.add_argument('--baseline', action='store_true')
args = parser.parse_args()
OUT = ROOT/'analysis/issue28-awake-animation'/args.label
OUT.mkdir(parents=True, exist_ok=True)
BIN = args.binary.resolve()
FF = imageio_ffmpeg.get_ffmpeg_exe()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
BASE = 'c0aa438fa4077785706a26276795efbb93e886d6'
head = subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
if args.baseline:
    assert sha(BIN) == '7bdd6acce62b00a766698ceb99b018d02f2d8d74684bb6669e254908852fa92a'
else:
    assert not subprocess.check_output(['git','diff','HEAD','--','src','assets','Cargo.toml','Cargo.lock','build.rs'],cwd=ROOT), 'Commit runtime before recording'
binding = {'source_commit': BASE if args.baseline else head, 'binary':str(BIN), 'binary_sha256':sha(BIN),
    'sampling':'All video export and pixel measurement use wizard_recording_frames.frames: fps=60, sample n at n/60 seconds.',
    'device_pixels':[1280,900], 'recorder_sha256':sha(Path(__file__)), 'scenes':{}}
if (OUT/'capture.json').exists():
    old=json.loads((OUT/'capture.json').read_text(encoding='utf-8'))
    assert old['binary_sha256']==binding['binary_sha256'] and old['source_commit']==binding['source_commit']
    binding['scenes']=old['scenes']
clean=[[(x+2*y)%7 for x in range(8)] for y in range(8)]
board=lambda b:'board load '+' '.join(str(v) for row in b for v in row)
skills={'knight':['charge','guard'],'elf':['strike','nature'],'goblin':['greedy','blood']}
for scene in args.scenes.split(','):
    role,form,kind,*case=scene.split('-');case='-'.join(case)
    hero={'knight':0,'elf':1,'goblin':3,'newknight':7}[role]
    menu=kind=='menu';slot=None if menu else skills[role].index(kind)+1
    log=OUT/f'{scene}.jsonl';log.write_text('',encoding='utf-8')
    commands=[] if menu else ['pause on',f'hero player {hero}','hero enemy 0','seed 280011',board(clean),'mp player 200']
    env=dict(os.environ,BATTLEBLOCKS_MEMORY_FLAGS='no_music,no_sound',BATTLEBLOCKS_LANG='zh')
    if menu: env.update(BATTLEBLOCKS_MENU_HERO=str(hero),BATTLEBLOCKS_MENU_PAGE='2' if form=='awake' else '0')
    command=[str(BIN),'--log',str(log)]
    for c in commands:command+=['--command',c]
    err=(OUT/f'{scene}-game.log').open('w',encoding='utf-8')
    game=subprocess.Popen(command,cwd=ROOT,env=env,stdin=subprocess.PIPE,stdout=subprocess.DEVNULL,stderr=err,text=True,encoding='utf-8')
    rec=None;sent=[]
    def rows():
        try:return [json.loads(s) for s in log.read_text(encoding='utf-8').splitlines() if s]
        except (FileNotFoundError,json.JSONDecodeError):return []
    def wait(pred,seconds=25):
        end=time.monotonic()+seconds
        while time.monotonic()<end:
            if game.poll() is not None:raise RuntimeError(f'{scene}: game exited')
            data=rows()
            if pred(data):return data
            time.sleep(.015)
        raise RuntimeError(f'{scene}: timeout {rows()[-1:]}')
    def send(c):
        sent.append(c);game.stdin.write(c+'\n');game.stdin.flush()
    def edit(c):
        seq=rows()[-1]['seq'];send(c)
        wait(lambda r:any(e['seq']>seq and e['event']=='debug_edit' and e['detail']==c for e in r))
    def playerturn(seq):
        return wait(lambda r:any(e['seq']>seq and e['event']=='turn_start' and not e['after']['enemy_turn'] for e in r))
    try:
        wait(lambda r:bool(r) if menu else any(e['event']=='debug_edit' and e['detail']==commands[-1] for e in r))
        if not menu and form=='awake':
            edit('hp player 1');edit('mp enemy 200');edit('turn enemy')
            seq=rows()[-1]['seq'];send('pause off');playerturn(seq)
            assert rows()[-1]['after']['awakened'][0]
            edit('pause on');edit(board(clean));edit('mp player 200');edit('hp enemy 700')
        if not menu:
            if kind in ('guard','blood') and case!='full':edit('hp player 100' if form=='awake' else 'hp player 300')
            if case=='delayed':edit('hp enemy 221')
            if case=='no-target':edit(board([[v if v!=1 else 0 for v in row] for row in clean]))
            if case=='terminal':
                edit('hp enemy 1');seq=rows()[-1]['seq'];send('pause off');send(f'cast {slot} 2 2');playerturn(seq)
                assert rows()[-1]['after']['awakened'][1]
                edit('pause on');edit(board(clean));edit('mp player 200');edit('hp enemy 1')
            # Retire setup popups and awakening presentation before evidence starts.
            time.sleep(3.)
        else:time.sleep(3.)
        send('status');time.sleep(.15)
        video=OUT/f'{scene}.mp4';recerr=(OUT/f'{scene}-ffmpeg.log').open('w',encoding='utf-8')
        rec=subprocess.Popen([FF,'-hide_banner','-loglevel','warning','-y','-f','gdigrab','-framerate','60','-draw_mouse','0','-i','title=BattleBlocks',
            '-vf','crop=1280:900:0:0,fps=60','-fps_mode','passthrough','-t','4' if menu else '6','-an','-c:v','libx264','-crf','16','-preset','fast','-pix_fmt','yuv420p','-movflags','+faststart',str(video)],
            stdout=subprocess.DEVNULL,stderr=recerr,creationflags=subprocess.CREATE_NO_WINDOW)
        skill=None;rollback=None
        if not menu:
            time.sleep(.8);seq=rows()[-1]['seq'];send('pause off');send(f'cast {slot} 2 2')
            data=wait(lambda r:any(e['seq']>seq and e['event']=='skill' for e in r))
            skill=next(e for e in data if e['seq']>seq and e['event']=='skill')
            if case=='rollback':
                time.sleep(.2);send('undo player')
                data=wait(lambda r:any(e['seq']>skill['seq'] and e['event']=='rollback' for e in r))
                rollback=next(e for e in data if e['seq']>skill['seq'] and e['event']=='rollback')
                assert rollback['after']==skill['before']
                send('pause on')
            if case=='pending-rollback':
                time.sleep(.2);edit('pause on');edit('mp player 100');send('undo')
                wait(lambda r:any(e['seq']>skill['seq'] and e['event']=='rollback' for e in r))
                time.sleep(.2);send('pause off')
            if case=='pause':
                time.sleep(.15);edit('pause on');time.sleep(1.6);edit('pause off')
            if case=='followup':
                time.sleep(.18);send('cast 2 2 2')
            if case=='switch':
                time.sleep(.25);edit(f'hero player {hero}')
        rec.wait(timeout=18);assert rec.returncode==0;recerr.close()
        wanted=[30,60,90,120,150,180,210,240,270,300]
        sampled={}
        for n,raw in enumerate(frames(FF,video)):
            if n in wanted:
                shot=Image.frombytes('RGB',(1280,900),raw)
                assert max(b-a for a,b in shot.getextrema())>100, 'Black capture'
                name=f'{scene}-{n:03}.webp';shot.save(OUT/name,lossless=True,method=6)
                sampled[str(n)]={'path':name,'rgba_sha256':hashlib.sha256(shot.convert('RGBA').tobytes()).hexdigest()}
        entry={'video':video.name,'video_sha256':sha(video),'log':log.name,'log_sha256':sha(log),
            'commands':commands+sent,'samples_60hz':sampled}
        if skill:
            entry.update(skill_seq=skill['seq'],skill_time=skill['time'],skill_detail=skill['detail'],awake=skill['before']['awakened'][0],
                events_after_skill=[{'event':e['event'],'delta':round(e['time']-skill['time'],6),'detail':e['detail'],'seq':e['seq']} for e in rows() if e['seq']>=skill['seq'] and e['event'] in ['skill','skill_hit','resolution','awakening','battle_end','rollback']])
            if case=='delayed' and kind=='greedy':
                assert any(e['event']=='resolution' and e['delta']>1.4 for e in entry['events_after_skill'])
            if case=='pending-rollback':
                hits=[e for e in entry['events_after_skill'] if e['event']=='skill_hit']
                assert len(hits)==1, hits
            if rollback:entry['rollback_full_snapshot_equal']=True
        binding['scenes'][scene]=entry
        (OUT/'capture.json').write_text(json.dumps(binding,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
        print('CAPTURED',scene,entry.get('events_after_skill','menu'),flush=True)
    finally:
        if rec and rec.poll() is None:rec.terminate();rec.wait(timeout=5)
        game.terminate();game.wait(timeout=5);err.close()
