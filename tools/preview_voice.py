"""Preview installed macOS female voices through the game's mechanical filter.

No models, downloads or cloud API. All generated files remain under .local/.
"""
from pathlib import Path
import argparse, shutil, subprocess, sys, tempfile, time, wave
ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--language', choices=['en', 'zh'], default='en')
parser.add_argument('--cue', choices=['victory', 'defeat'], default='victory')
args = parser.parse_args()
if sys.platform != 'darwin':
    raise SystemExit('This preview uses the macOS say engine.')
row = next(line.split('|') for line in (ROOT/'config/role-lore.txt').read_text().splitlines()
           if line.startswith(f'12|voice.{args.cue}|'))
text = row[3 if args.language == 'zh' else 2].split(' · ', 1)[-1]
listing = subprocess.check_output(['/usr/bin/say', '-v', '?'], text=True)
locale, preferred = ('zh_CN', 'Tingting') if args.language == 'zh' else ('en_US', 'Samantha')
voices = [line.split(locale, 1)[0].strip() for line in listing.splitlines() if locale in line]
voice = next((v for v in voices if v.startswith(preferred)), next(iter(voices), None))
if not voice:
    raise SystemExit(f'No installed {locale} voice; install one in macOS settings.')
out = ROOT/'.local/audio'
out.mkdir(parents=True, exist_ok=True)
raw, processed = [out/f'{args.language}-{kind}.wav' for kind in ('system', 'mechanical')]
rustc = shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
with tempfile.TemporaryDirectory(prefix='bb-voice-') as temp:
    temp = Path(temp)
    binary = temp/'filter'
    subprocess.run([rustc, '-O', str(ROOT/'tools/preview_voice.rs'), '-o', str(binary)], check=True)
    start = time.perf_counter()
    subprocess.run(['/usr/bin/say', '-v', voice, '-r', '165', '-o', str(raw),
                    '--file-format=WAVE', '--data-format=LEI16@22050', text], check=True)
    elapsed = time.perf_counter() - start
    with wave.open(str(raw), 'rb') as audio:
        assert audio.getnchannels() == 1 and audio.getsampwidth() == 2
        rate = audio.getframerate()
        (temp/'in.pcm').write_bytes(audio.readframes(audio.getnframes()))
    subprocess.run([str(binary), str(rate), str(temp/'in.pcm'), str(temp/'out.pcm')], check=True)
    with wave.open(str(processed), 'wb') as audio:
        audio.setparams((1, 2, rate, 0, 'NONE', 'not compressed'))
        audio.writeframes((temp/'out.pcm').read_bytes())
print(f'Voice: {voice}; synthesis: {elapsed:.3f} s; dialogue: {text}')
print(f'System: {raw}\nMechanical: {processed}')
