"""Compare actual audio output modes on macOS with a shared CPAL backend."""
from pathlib import Path
import hashlib,json,os,re,statistics,subprocess,time
ROOT=Path(__file__).resolve().parents[1];LAB=ROOT/'analysis/audio-lab';OUT=LAB/'results';OUT.mkdir(exist_ok=True)
binary=LAB/'target/release/battleblocks-audio-lab';rows=[]
for repeat in range(2):
 for mode in (['idle','buffer','stream','synth'] if repeat==0 else ['synth','stream','buffer','idle']):
  with (OUT/f'{mode}-{repeat}.log').open('w') as log:
   p=subprocess.Popen([str(binary),mode,'16'],env=dict(os.environ,AUDIO_LAB_GAIN='0'),stdout=log,stderr=log)
   try:
    start=time.monotonic();samples=[]
    for seconds in [5,10,14]:
     time.sleep(max(0,seconds-(time.monotonic()-start)))
     if p.poll() is not None:raise RuntimeError((OUT/f'{mode}-{repeat}.log').read_text())
     fields=subprocess.check_output(['ps','-p',str(p.pid),'-o','rss=,%cpu='],text=True).split()
     vm=subprocess.check_output(['vmmap','-summary',str(p.pid)],text=True)
     (OUT/f'{mode}-{repeat}-{seconds}.txt').write_text(vm)
     m=re.search(r'Physical footprint:\s+([\d.]+)([MKG])',vm)
     samples.append(dict(seconds=seconds,rss_mib=int(fields[0])/1024,cpu_percent=float(fields[1]),footprint_mib=float(m[1])*dict(M=1,K=1/1024,G=1024)[m[2]]))
    p.wait(timeout=10)
    if p.returncode:raise RuntimeError((OUT/f'{mode}-{repeat}.log').read_text())
   finally:
    if p.poll() is None:p.terminate();p.wait(timeout=10)
  output=(OUT/f'{mode}-{repeat}.log').read_text();messages=[json.loads(line) for line in output.splitlines() if line.startswith('{')]
  if messages[-1]['decoder_failed'] or messages[-1]['underrun_frames']:raise RuntimeError('Audio errors or underruns observed')
  row=dict(mode=mode,repeat=repeat,samples=samples,output=messages);rows.append(row)
  (LAB/'measurements.json').write_text(json.dumps(dict(binary_bytes=binary.stat().st_size,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),scenario='Release macOS; shared CPAL output backend; gain zero still performs all sample generation/decoding; 16 seconds per process, reverse repeat order.',rows=rows),indent=2))
  print(json.dumps(row),flush=True)
print(json.dumps({mode:{'footprint14_mib':statistics.mean(r['samples'][-1]['footprint_mib'] for r in rows if r['mode']==mode),'rss14_mib':statistics.mean(r['samples'][-1]['rss_mib'] for r in rows if r['mode']==mode),'cpu14_percent':statistics.mean(r['samples'][-1]['cpu_percent'] for r in rows if r['mode']==mode)} for mode in ['idle','buffer','stream','synth']},indent=2),flush=True)
