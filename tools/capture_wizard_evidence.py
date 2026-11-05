"""Record the real Windows game via FFmpeg, driving its existing stdin console.

No combat simulation or synthetic video frames. Install imageio-ffmpeg==0.6.0
into .build-tools/media-site, or pass --ffmpeg. Run only one capture at a time.
The capture size is the observed client area in device pixels (125% DPI here).
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', required=True, type=Path)
parser.add_argument('--label', required=True)
parser.add_argument('--ffmpeg')
parser.add_argument('--size', default='1280:900')
parser.add_argument('--scenes', default='normal-menu,awake-menu,normal-lightning,normal-meteor,awake-lightning,awake-meteor,delayed-meteor,terminal-meteor,rollback')
args = parser.parse_args()
if sys.platform != 'win32':
    parser.error('This recorder uses Windows gdigrab; gameplay probes remain platform independent.')
if not args.ffmpeg:
    sys.path.insert(0, str(ROOT / '.build-tools/media-site'))
    import imageio_ffmpeg
    args.ffmpeg = imageio_ffmpeg.get_ffmpeg_exe()
OUT = ROOT / 'analysis/issue27-wizard-animation' / args.label
OUT.mkdir(parents=True, exist_ok=True)
clean = [[(x + 2*y) % 7 for x in range(8)] for y in range(8)]
board_command = 'board load ' + ' '.join(str(v) for row in clean for v in row)
binary = args.binary.resolve()
summary = {'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
           'git_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
           'fps': 60, 'device_pixel_crop': args.size, 'scenes': {}}
summary_path = OUT / 'capture.json'
if summary_path.exists():
    previous = json.loads(summary_path.read_text())
    if previous['binary_sha256'] != summary['binary_sha256']:
        raise SystemExit('Use a new label for a different binary; do not mix evidence.')
    summary['scenes'] = previous['scenes']


for scene in args.scenes.split(','):
    log = OUT / f'{scene}.jsonl'
    env = dict(os.environ, BATTLEBLOCKS_MEMORY_FLAGS='no_music,no_sound', BATTLEBLOCKS_LANG='zh')
    menu = scene.endswith('-menu')
    commands = [] if menu else ['pause on', 'hero player 2', 'hero enemy 0', 'seed 270011', board_command, 'mp player 250']
    if menu:
        env.update(BATTLEBLOCKS_MENU_HERO='2', BATTLEBLOCKS_MENU_PAGE='2' if scene.startswith('awake') else '0')
    command = [str(binary), '--log', str(log)]
    for line in commands:
        command += ['--command', line]
    error = (OUT / f'{scene}-game.log').open('w', encoding='utf-8')
    game = subprocess.Popen(command, cwd=ROOT, env=env, stdin=subprocess.PIPE,
                            stdout=subprocess.DEVNULL, stderr=error, text=True, encoding='utf-8')
    recorder = None

    def records():
        try:
            return [json.loads(line) for line in log.read_text(encoding='utf-8').splitlines() if line]
        except (FileNotFoundError, json.JSONDecodeError):
            return []

    def wait(predicate, seconds=20):
        until = time.monotonic() + seconds
        while time.monotonic() < until:
            if game.poll() is not None:
                raise RuntimeError(f'{scene}: game exited {game.returncode}')
            data = records()
            if predicate(data):
                return data
            time.sleep(.015)
        raise RuntimeError(f'{scene}: timeout; last events {records()[-2:]}')

    def send(line):
        game.stdin.write(line + '\n')
        game.stdin.flush()

    def edit(line):
        marker = records()[-1]['seq']
        send(line)
        wait(lambda rows: any(e['seq'] > marker and e['event'] == 'debug_edit' and e['detail'] == line for e in rows))

    def player_turn_after(marker):
        wait(lambda rows: any(e['seq'] > marker and e['event'] == 'turn_start' and not e['after']['enemy_turn'] for e in rows))

    try:
        wait(lambda rows: bool(rows) if menu else any(e['event'] == 'debug_edit' and e['detail'] == commands[-1] for e in rows))
        if scene.startswith('awake-') and not menu:
            # A real opponent hit awakens the wizard; debug HP editing alone does not.
            edit('hp player 1')
            edit('mp enemy 200')
            edit('turn enemy')
            marker = records()[-1]['seq']
            send('pause off')
            player_turn_after(marker)
            assert records()[-1]['after']['awakened'][0], 'setup must awaken through real combat'
            edit('pause on')
            edit(board_command)
            edit('mp player 400')
            edit('hp enemy 700')
        if scene == 'delayed-meteor':
            edit('hp enemy 221')
        if scene == 'terminal-meteor':
            edit('hp enemy 1')
            marker = records()[-1]['seq']
            send('pause off')
            send('cast 2')
            player_turn_after(marker)
            assert records()[-1]['after']['awakened'][1]
            edit('pause on')
            edit(board_command)
            edit('mp player 250')
            edit('hp enemy 1')
        # A status event marks the scene's exact initial game state.
        marker = records()[-1]['seq']
        send('status')
        wait(lambda rows: rows[-1]['seq'] > marker)
        time.sleep(.3)
        video = OUT / f'{scene}.mp4'
        rec_error = (OUT / f'{scene}-ffmpeg.log').open('w', encoding='utf-8')
        started = time.monotonic()
        recorder = subprocess.Popen([args.ffmpeg, '-hide_banner', '-loglevel', 'warning', '-y',
            '-f', 'gdigrab', '-framerate', '60', '-draw_mouse', '0', '-i', 'title=BattleBlocks',
            '-vf', f'crop={args.size}:0:0', '-t', '6', '-an', '-c:v', 'libx264', '-crf', '16',
            '-preset', 'fast', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', str(video)],
            stdout=subprocess.DEVNULL, stderr=rec_error, creationflags=subprocess.CREATE_NO_WINDOW)
        skill = None
        if not menu:
            time.sleep(1.)
            marker = records()[-1]['seq']
            send('pause off')
            send('cast 1' if 'lightning' in scene or scene == 'rollback' else 'cast 2')
            data = wait(lambda rows: any(e['seq'] > marker and e['event'] == 'skill' for e in rows))
            skill = next(e for e in data if e['seq'] > marker and e['event'] == 'skill')
            seen = time.monotonic() - started
            if scene == 'rollback':
                time.sleep(.55)
                send('undo player')
                data = wait(lambda rows: any(e['seq'] > skill['seq'] and e['event'] == 'rollback' for e in rows))
                restored = next(e for e in data if e['seq'] > skill['seq'] and e['event'] == 'rollback')
                assert restored['after'] == skill['before']
                send('pause on')
        recorder.wait(timeout=20)
        if recorder.returncode:
            raise RuntimeError(f'ffmpeg failed: {scene}')
        data = records()
        entry = {'video': video.name, 'events': log.name, 'recorded_frames_expected': 360}
        if skill:
            hit = next((e for e in data if e['seq'] > skill['seq'] and e['event'] == 'resolution' and e['action'] == skill['action']), None)
            entry.update(skill_seq=skill['seq'], skill_time=skill['time'], skill=skill['detail'],
                         skill_observed_seconds_after_recorder_start=seen,
                         clear_seconds_after_skill=None if hit is None else hit['time'] - skill['time'],
                         player_awake=skill['before']['awakened'][0])
            if scene not in ('terminal-meteor', 'rollback'):
                assert hit is not None, 'video must include actual board clear'
            if scene == 'delayed-meteor':
                assert entry['clear_seconds_after_skill'] > 1.4, 'must exercise a cast beyond the old lifetime'
        summary['scenes'][scene] = entry
        summary_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        print(f'CAPTURED {args.label}/{scene}: {entry.get("clear_seconds_after_skill")}', flush=True)
    finally:
        if recorder and recorder.poll() is None:
            recorder.terminate()
            recorder.wait(timeout=5)
        game.terminate()
        try:
            game.wait(timeout=5)
        except subprocess.TimeoutExpired:
            game.kill()
            game.wait()
        error.close()
