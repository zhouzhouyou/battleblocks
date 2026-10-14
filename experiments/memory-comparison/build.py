from pathlib import Path
import shutil,subprocess
ROOT=Path(__file__).resolve().parent
cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
for name in ['miniquad','macro-noaudio','macro-audio','std-control']:
 subprocess.run([cargo,'build','--release','--locked','--manifest-path',str(ROOT/name/'Cargo.toml')],check=True)
(ROOT/'results').mkdir(exist_ok=True)
subprocess.run(['clang','-O2','-fobjc-arc','-framework','Cocoa',str(ROOT/'cocoa-control/main.m'),'-o',str(ROOT/'results/cocoa-control')],check=True)
