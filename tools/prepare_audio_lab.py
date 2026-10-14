"""Publish the audio experiment next to the local Web game; no game backend changes."""
from pathlib import Path
import shutil
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'dist/web/audio-lab';OUT.mkdir(parents=True,exist_ok=True)
for name in ['index.html','worklet.js','score.json']:
 shutil.copy2(ROOT/'analysis/audio-lab'/name,OUT/name)
shutil.copy2(ROOT/'assets/bgm.ogg',OUT/'bgm.ogg')
print('Audio lab: http://127.0.0.1:8765/audio-lab/')
