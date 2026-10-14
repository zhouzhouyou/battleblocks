from pathlib import Path
import json,os,statistics,subprocess,time,sys
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'dist/memory-analysis';OUT.mkdir(parents=True,exist_ok=True)
scenarios=[('baseline',''),('no_music','no_music'),('procedural','procedural'),('no_actors','no_actors'),('no_text','no_text'),('no_sound','no_sound'),('combined','no_music,no_sound,no_actors,no_text,procedural'),('small_window','small_window'),('hidpi','hidpi'),('empty_window','')]
if '--render' in sys.argv:
    OUT=ROOT/'dist/memory-render-analysis';OUT.mkdir(parents=True,exist_ok=True)
    scenarios=[('baseline',''),('batched','batched'),('small_buffers','small_buffers'),('batched_small_buffers','batched,small_buffers'),('procedural_batched_small','procedural,batched,small_buffers'),('hidpi_no_text','hidpi,no_text')]
results=[]
# Keep historic baseline semantics after promoting the optimizations to defaults.
legacy=[]
for name,flags in scenarios:
    if name in ('batched_small_buffers','procedural_batched_small'):
        pass
    elif name=='batched':flags+=',large_buffers'
    elif name=='small_buffers':flags+=',legacy_batches'
    elif name!='empty_window':flags+=',legacy_batches,large_buffers'
    legacy.append((name,flags.strip(',')))
scenarios=legacy
for repeat in range(2):
    for name,flags in (scenarios if repeat==0 else scenarios[::-1]):
        binary='memory_probe' if name=='empty_window' else 'battleblocks'
        process=subprocess.Popen([str(ROOT/'target/release'/binary)],env=dict(os.environ,BATTLEBLOCKS_BENCH_HERO='0',BATTLEBLOCKS_MEMORY_FLAGS=flags),stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
        try:
            time.sleep(2)
            samples=[]
            for _ in range(6):
                if process.poll() is not None:raise RuntimeError(process.stderr.read().decode())
                fields=subprocess.check_output(['ps','-p',str(process.pid),'-o','rss=,%cpu=']).decode().split();samples.append(int(fields[0])*1024);time.sleep(.25)
            summary=subprocess.run(['vmmap','-summary',str(process.pid)],capture_output=True,text=True,timeout=20)
            (OUT/f'{name}-{repeat}.txt').write_text(summary.stdout+summary.stderr)
            result={'scenario':name,'repeat':repeat,'flags':flags,'rss_median_bytes':statistics.median(samples),'rss_max_bytes':max(samples),'vmmap_exit':summary.returncode}
            results.append(result);print(json.dumps(result),flush=True)
        finally:
            process.terminate();process.wait(timeout=5)
(OUT/'results.json').write_text(json.dumps(results,indent=2))
print('Summary MiB:',json.dumps({name:round(statistics.mean(r['rss_median_bytes'] for r in results if r['scenario']==name)/1048576,2) for name,_ in scenarios}),flush=True)
