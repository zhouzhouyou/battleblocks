from pathlib import Path
import json,os,re,subprocess,time
ROOT=Path(__file__).resolve().parents[1];LAB=ROOT/'analysis/audio-lab';OUT=LAB/'results';OUT.mkdir(exist_ok=True)
rows=[]
with (OUT/'stream-stability.log').open('w') as log:
 p=subprocess.Popen([str(LAB/'target/release/battleblocks-audio-lab'),'stream','60'],env=dict(os.environ,AUDIO_LAB_GAIN='0'),stdout=log,stderr=log)
 try:
  start=time.monotonic()
  for seconds in [5,30,58]:
   time.sleep(max(0,seconds-(time.monotonic()-start)))
   if p.poll() is not None:raise RuntimeError('Stream process exited')
   fields=subprocess.check_output(['ps','-p',str(p.pid),'-o','rss=,%cpu='],text=True).split();vm=subprocess.check_output(['vmmap','-summary',str(p.pid)],text=True)
   (OUT/f'stream-stability-{seconds}.txt').write_text(vm);m=re.search(r'Physical footprint:\s+([\d.]+)([MKG])',vm)
   row=dict(seconds=seconds,rss_mib=int(fields[0])/1024,cpu_percent=float(fields[1]),footprint_mib=float(m[1])*dict(M=1,K=1/1024,G=1024)[m[2]]);rows.append(row);print(json.dumps(row),flush=True)
  p.wait(timeout=10)
  if p.returncode:raise RuntimeError('Stream process failed')
 finally:
  if p.poll() is None:p.terminate();p.wait()
(LAB/'stability.json').write_text(json.dumps(dict(samples=rows,output=[json.loads(line) for line in (OUT/'stream-stability.log').read_text().splitlines() if line.startswith('{')]),indent=2))
