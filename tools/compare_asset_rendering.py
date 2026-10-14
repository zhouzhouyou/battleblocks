"""Release image/procedural board comparison; no_actors is an upper bound, not a replacement."""
from pathlib import Path
import hashlib, json, os, re, subprocess, time
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'dist/asset-rendering'; OUT.mkdir(parents=True, exist_ok=True)
binary = ROOT / 'target/release/battleblocks'
cases = [('procedural_board', ''), ('procedural_board_no_actors', 'no_actors'), ('procedural_board_no_music', 'no_music')]
rows = []
for repeat in range(2):
    for name, flags in (cases if repeat == 0 else cases[::-1]):
        with (OUT / f'{name}-{repeat}.log').open('w') as log:
            p = subprocess.Popen([str(binary)], env=dict(os.environ, BATTLEBLOCKS_BENCH_HERO='0', BATTLEBLOCKS_MEMORY_FLAGS=flags+',play_music'), stdout=log, stderr=log)
            try:
                started = time.monotonic(); samples = []
                for seconds in [5, 10]:
                    time.sleep(max(0, seconds - (time.monotonic() - started)))
                    if p.poll() is not None: raise RuntimeError('Game exited')
                    fields = subprocess.check_output(['ps', '-p', str(p.pid), '-o', 'rss=,%cpu='], text=True).split()
                    vm = subprocess.check_output(['vmmap', '-summary', str(p.pid)], text=True)
                    (OUT / f'{name}-{repeat}-{seconds}.txt').write_text(vm)
                    m = re.search(r'Physical footprint:\s+([\d.]+)([MKG])', vm)
                    samples.append(dict(seconds=seconds, rss_mib=int(fields[0])/1024, cpu_percent=float(fields[1]), footprint_mib=float(m[1])*dict(M=1,K=1/1024,G=1024)[m[2]]))
                row = dict(case=name, repeat=repeat, flags=flags, samples=samples);rows.append(row)
                print(json.dumps(row), flush=True)
            finally:
                p.terminate();p.wait(timeout=10)
result = dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), scenario='Release, knight + demon, fixed window, BGM playing, small buffers and grouped draws; no_actors removes sprites without replacing them.', rows=rows)
(ROOT/'analysis/asset-rendering-current.json').write_text(json.dumps(result,indent=2))
