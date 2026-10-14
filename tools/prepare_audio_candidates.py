"""Generate evaluation audio; does not replace the game's soundtrack. Requires ffmpeg/ffprobe."""
from pathlib import Path
import json, shutil, subprocess
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'analysis/audio-candidates';OUT.mkdir(parents=True,exist_ok=True)
ffmpeg=shutil.which('ffmpeg');ffprobe=shutil.which('ffprobe')
if not ffmpeg or not ffprobe:raise SystemExit('Install ffmpeg to generate audio candidates')
source=ROOT/'asset-sources/bgm.wav'
cases=[('bgm-mono.wav',['-ac','1','-ar','44100','-c:a','pcm_s16le']),('bgm-bitcrushed.ogg',['-ac','2','-ar','44100','-af','acrusher=bits=8:samples=4:mix=0.6','-c:a','vorbis','-strict','-2','-q:a','2']),('bgm-u8.wav',['-ac','1','-ar','22050','-c:a','pcm_u8'])]
for name,args in cases:
 subprocess.run([ffmpeg,'-v','error','-y','-i',str(source),*args,str(OUT/name)],check=True)
# The short effect is a candidate too: compression saves package bytes, not decoded samples.
subprocess.run([ffmpeg,'-v','error','-y','-i',str(ROOT/'assets/crush.wav'),'-ac','2','-c:a','vorbis','-strict','-2','-q:a','3',str(OUT/'crush.ogg')],check=True)
rows=[]
for p in [ROOT/'assets/bgm.ogg',ROOT/'assets/crush.wav',*(OUT/name for name,_ in cases),OUT/'crush.ogg']:
 info=json.loads(subprocess.check_output([ffprobe,'-v','error','-show_entries','format=duration,size:stream=codec_name,sample_rate,channels,bits_per_sample','-of','json',str(p)],text=True))
 duration=float(info['format']['duration']);stream=info['streams'][0]
 # This is the native backend's steady-state sample representation, excluding padding/capacity and backend overhead.
 rows.append(dict(file=str(p.relative_to(ROOT)),bytes=p.stat().st_size,duration=duration,stream=stream,native_pcm_bytes_estimate=round(duration*44100*2*4),web_pcm_48k_bytes_estimate=round(duration*48000*int(stream['channels'])*4)))
(OUT/'measurements.json').write_text(json.dumps(rows,indent=2));print(json.dumps(rows,indent=2))
