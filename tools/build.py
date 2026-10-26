"""Bootstrap cached font tools, then test/build/package the prototype."""
from pathlib import Path
import argparse, hashlib, os, shutil, subprocess, sys, urllib.request
ROOT=Path(__file__).resolve().parents[1]
CACHE=ROOT/'.build-tools'
FONT_URL='https://raw.githubusercontent.com/google/fonts/main/ofl/notosanssc/NotoSansSC%5Bwght%5D.ttf'
FONT_SHA256='a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da'
FONTTOOLS='4.67.0'
parser=argparse.ArgumentParser();parser.add_argument('--setup-only',action='store_true');group=parser.add_mutually_exclusive_group();group.add_argument('--native-only',action='store_true');group.add_argument('--web-only',action='store_true');args=parser.parse_args()
if sys.version_info<(3,10): raise SystemExit('Font build tools require Python 3.10 or newer.')
CACHE.mkdir(exist_ok=True)
source=CACHE/'NotoSansSC.ttf'
if not source.exists():
    print('Downloading font source (cached, excluded from game packages)',flush=True)
    data=urllib.request.urlopen(FONT_URL).read()
    if hashlib.sha256(data).hexdigest()!=FONT_SHA256:raise SystemExit('Font source checksum changed; update the pinned source intentionally.')
    source.write_bytes(data)
if hashlib.sha256(source.read_bytes()).hexdigest()!=FONT_SHA256:raise SystemExit('Cached font checksum mismatch')
site=CACHE/'site'
# Rebuild the cache from scratch on a version change; pip --target keeps old package folders.
if not (site/f'fonttools-{FONTTOOLS}.dist-info').exists():
    shutil.rmtree(site,ignore_errors=True)
    subprocess.run([sys.executable,'-m','pip','install','--target',str(site),f'fonttools=={FONTTOOLS}'],check=True)
path_file=CACHE/'python-path.txt'
if not path_file.exists() or path_file.read_text()!=sys.executable:path_file.write_text(sys.executable)
if args.setup_only:raise SystemExit(0)
cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
rustup=shutil.which('rustup') or str(Path.home()/'.cargo/bin/rustup')
if not args.native_only:subprocess.run([rustup,'target','add','wasm32-unknown-unknown'],check=True)
commands=[[cargo,'test','--locked']]
if not args.web_only:commands.append([cargo,'build','--release','--locked','--bin','battleblocks'])
if not args.native_only:commands.append([cargo,'build','--release','--locked','--bin','battleblocks','--target','wasm32-unknown-unknown'])
package=[sys.executable,str(ROOT/'tools/package.py')]
if args.native_only:package.append('--native-only')
if args.web_only:package.append('--web-only')
commands.append(package)
for command in commands:subprocess.run(command,cwd=ROOT,check=True)
