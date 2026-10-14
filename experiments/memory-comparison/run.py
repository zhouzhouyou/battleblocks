from pathlib import Path
import hashlib,json,os,re,statistics,subprocess,time,sys
ROOT=Path(__file__).resolve().parent;OUT=ROOT/'results';OUT.mkdir(exist_ok=True)
case=[('std_idle','std-control','empty','gl'),('cocoa_window','cocoa-control','empty','gl'),('miniquad_gl','miniquad','empty','gl'),('miniquad_metal','miniquad','empty','metal'),('macro_noaudio_gl','macro-noaudio','empty','gl'),('macro_noaudio_metal','macro-noaudio','empty','metal'),('macro_audio_empty','macro-audio','empty','gl'),('macro_font_loaded','macro-noaudio','font','gl'),('macro_font_drawn','macro-noaudio','font,draw','gl'),('macro_music_loaded','macro-audio','music','gl'),('macro_font_music','macro-audio','font,draw,music,play','gl'),('game_original','game','empty','gl'),('game_optimized','game','empty','gl')]
if '--normalized' in sys.argv:
 OUT=ROOT/'results-normalized';OUT.mkdir(exist_ok=True)
 case=[('miniquad_metal_1x','miniquad','empty','metal'),('macro_noaudio_metal_1x','macro-noaudio','empty','metal'),('macro_audio_metal_1x','macro-audio','empty','metal')]
if '--isolate' in sys.argv:
 OUT=ROOT/'results-isolate';OUT.mkdir(exist_ok=True)
 case=[('audio_font_loaded','macro-audio','font','gl'),('audio_font_drawn','macro-audio','font,draw','gl'),('music_play_no_text','macro-audio','music,play','gl'),('font_music_no_play','macro-audio','font,draw,music','gl'),('font_music_prewarm','macro-audio','font,draw,music,play,prewarm','gl')]
if '--buffers' in sys.argv:
 OUT=ROOT/'results-buffers';OUT.mkdir(exist_ok=True)
 case=[('audio_font_small','macro-audio','font,draw','gl'),('font_music_small','macro-audio','font,draw,music,play','gl'),('font_music_prewarm_small','macro-audio','font,draw,music,play,prewarm','gl'),('game_play_original','game','empty','gl'),('game_play_optimized','game','empty','gl')]
if '--font-cache' in sys.argv:
 OUT=ROOT/'results-font-cache';OUT.mkdir(exist_ok=True)
 case=[('font_fixed','macro-noaudio','font,draw','gl'),('font_resize_unbounded','macro-noaudio','font,draw,resize','gl'),('font_resize_bucket','macro-noaudio','font,draw,resize,bucket','gl')]
if '--font-frame' in sys.argv:
 OUT=ROOT/'results-font-frame';OUT.mkdir(exist_ok=True)
 case=[('font_resize_bucket_framewarm','macro-noaudio','font,draw,resize,bucket,framewarm','gl')]
rows=[]
for repeat in range(2):
 for name,project,mode,backend in (case if repeat==0 else case[::-1]):
  if project=='cocoa-control':binary=OUT/'cocoa-control'
  elif project=='game':binary=ROOT.parent.parent/'target/release/battleblocks'
  else:binary=ROOT/project/'target/release'/('bb-'+project)
  env=dict(os.environ,BB_MODE=mode,BB_BACKEND=backend,BB_SMALL_BUFFERS='1' if '--buffers' in sys.argv or '--font-cache' in sys.argv or '--font-frame' in sys.argv else '0',BB_PHYSICAL_NORMALIZED='1' if '--normalized' in sys.argv else '0',BATTLEBLOCKS_BENCH_HERO='0',BATTLEBLOCKS_MEMORY_FLAGS=('legacy_batches,large_buffers' if name in ('game_original','game_play_original') else '')+(',play_music' if name.startswith('game_play') else ''))
  log=OUT/f'{name}-{repeat}.log'
  with log.open('w') as output:
   process=subprocess.Popen([str(binary)],env=env,stdout=output,stderr=subprocess.STDOUT)
   try:
    start=time.monotonic();samples=[];snapshots=[]
    for target in [5,10,15]:
     while time.monotonic()-start<target:time.sleep(min(.25,target-(time.monotonic()-start)))
     if process.poll() is not None:raise RuntimeError(log.read_text())
     rss=int(subprocess.check_output(['ps','-p',str(process.pid),'-o','rss=']).decode().strip())*1024
     samples.append({'seconds':target,'rss_bytes':rss})
     if target in [5,15]:
      vm=subprocess.run(['vmmap','-summary',str(process.pid)],capture_output=True,text=True,timeout=25)
      text=vm.stdout+vm.stderr;(OUT/f'{name}-{repeat}-{target}.vmmap.txt').write_text(text)
      match=re.search(r'Physical footprint:\s+([0-9.]+)([MKG])',text)
      if not match:raise RuntimeError('vmmap failed: '+text[:500])
      snapshots.append({'seconds':target,'footprint_mib':float(match[1])*({'M':1,'K':1/1024,'G':1024}[match[2]])})
    row={'case':name,'repeat':repeat,'backend':backend,'mode':mode,'binary_bytes':binary.stat().st_size,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'rss_samples':samples,'footprint_samples':snapshots,'display_log':log.read_text().strip()}
    rows.append(row);(OUT/'results.json').write_text(json.dumps(rows,indent=2))
    print(json.dumps({'case':name,'repeat':repeat,'rss15_mib':round(samples[-1]['rss_bytes']/1048576,2),'footprint15_mib':snapshots[-1]['footprint_mib']}),flush=True)
   finally:
    process.terminate();process.wait(timeout=10)
summary=[]
for name,_,_,_ in case:
 r=[x for x in rows if x['case']==name]
 summary.append({'case':name,'rss15_mib':statistics.mean(x['rss_samples'][-1]['rss_bytes'] for x in r)/1048576,'footprint15_mib':statistics.mean(x['footprint_samples'][-1]['footprint_mib'] for x in r),'footprint5_mib':statistics.mean(x['footprint_samples'][0]['footprint_mib'] for x in r)})
(OUT/'summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary,indent=2),flush=True)
