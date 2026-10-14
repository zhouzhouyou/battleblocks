"""macOS idle battle RSS sampling; not a full battle peak benchmark."""
from pathlib import Path
import json, os, statistics, subprocess, time
ROOT=Path(__file__).resolve().parents[1]
results=[]
for i,name in enumerate(['Knight','Elf','Wizard','Goblin','Imp','Chort','Big Zombie']):
    env=dict(os.environ,BATTLEBLOCKS_BENCH_HERO=str(i))
    process=subprocess.Popen([str(ROOT/'target/release/battleblocks')],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
    try:
        time.sleep(2)
        rows=[]
        for _ in range(8):
            if process.poll() is not None:raise RuntimeError(process.stderr.read().decode())
            fields=subprocess.check_output(['ps','-p',str(process.pid),'-o','rss=,%cpu=']).decode().split()
            rows.append((int(fields[0])*1024,float(fields[1])))
            time.sleep(.25)
        results.append({'hero':name,'samples':len(rows),'rss_min_bytes':min(r[0] for r in rows),'rss_median_bytes':statistics.median(r[0] for r in rows),'rss_max_bytes':max(r[0] for r in rows)})
        print(json.dumps(results[-1]),flush=True)
    finally:
        process.terminate();process.wait(timeout=5)
(ROOT/'dist/memory.json').write_text(json.dumps({'platform':'macOS arm64','scenario':'idle battle, selected hero + boss; 2s warmup, 8 RSS samples at 0.25s intervals','not_measured':'full battle peaks, browser total memory, Windows/Linux','results':results},indent=2))
