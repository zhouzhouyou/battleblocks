from pathlib import Path
import json,os,re,subprocess,time
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'results-stability';OUT.mkdir(exist_ok=True);rows=[]
cases=[('macro_noaudio_gl',ROOT/'macro-noaudio/target/release/bb-macro-noaudio',''),('game_play_optimized',ROOT.parent.parent/'target/release/battleblocks','play_music')]
for name,binary,flags in cases:
 with (OUT/(name+'.log')).open('w') as log:
  p=subprocess.Popen([str(binary)],env=dict(os.environ,BB_MODE='empty',BB_BACKEND='gl',BB_PHYSICAL_NORMALIZED='0',BATTLEBLOCKS_BENCH_HERO='0',BATTLEBLOCKS_MEMORY_FLAGS=flags),stdout=log,stderr=subprocess.STDOUT)
  try:
   start=time.monotonic()
   for seconds in [5,15,30,60]:
    while time.monotonic()-start<seconds:time.sleep(.25)
    if p.poll() is not None:raise RuntimeError('Process exited')
    rss=int(subprocess.check_output(['ps','-p',str(p.pid),'-o','rss=']).decode())/1048576*1024
    m=subprocess.run(['vmmap','-summary',str(p.pid)],capture_output=True,text=True,timeout=25);(OUT/f'{name}-{seconds}.txt').write_text(m.stdout+m.stderr)
    f=re.search(r'Physical footprint:\s+([0-9.]+)([MK])',m.stdout);row={'case':name,'seconds':seconds,'rss_mib':rss,'footprint_mib':float(f[1])*(1 if f[2]=='M' else 1/1024)};rows.append(row);(OUT/'results.json').write_text(json.dumps(rows,indent=2));print(json.dumps(row),flush=True)
  finally:p.terminate();p.wait(timeout=10)
