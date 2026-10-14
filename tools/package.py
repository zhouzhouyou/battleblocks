"""Package the native host build and Web build with only their resolved dependency licenses."""
from pathlib import Path
import argparse,gzip,hashlib,json,platform,shutil,subprocess,sys,zipfile
ROOT=Path(__file__).resolve().parents[1];DIST=ROOT/'dist';DIST.mkdir(exist_ok=True)
parser=argparse.ArgumentParser();group=parser.add_mutually_exclusive_group();group.add_argument('--native-only',action='store_true');group.add_argument('--web-only',action='store_true');args=parser.parse_args()
cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
host=next(line.split(': ',1)[1] for line in subprocess.check_output([rustc,'-vV'],text=True).splitlines() if line.startswith('host:'))
results={'macroquad':'0.4.16','native_target':host}
def licenses_for(target,out):
    if out.exists():shutil.rmtree(out)
    out.mkdir(parents=True)
    metadata=json.loads(subprocess.check_output([cargo,'metadata','--format-version','1','--locked','--filter-platform',target],cwd=ROOT,text=True))
    nodes={n['id']:n for n in metadata['resolve']['nodes']};root=metadata['resolve']['root'];needed=set();pending=[root]
    while pending:
        node=pending.pop()
        if node in needed:continue
        needed.add(node);pending.extend(dep['pkg'] for dep in nodes[node]['deps'])
    manifest=[]
    for package in metadata['packages']:
        if package['id'] not in needed or package['id']==root:continue
        directory=Path(package['manifest_path']).parent;dest=out/(package['name']+'-'+package['version']);dest.mkdir()
        for file in directory.rglob('*'):
            if file.is_file() and file.name.lower().startswith(('license','copying','notice')):
                target_file=dest/file.relative_to(directory);target_file.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(file,target_file)
        manifest.append({'name':package['name'],'version':package['version'],'license':package.get('license')})
    (out/'DEPENDENCIES.json').write_text(json.dumps(manifest,indent=2))
    shutil.copy2(ROOT/'assets/FONT-LICENSE.txt',out/'FONT-LICENSE.txt')
    (out/'ASSETS.txt').write_text('Artwork/audio originate from the original BattleBlocks repository. Original third-party redistribution rights have not been independently verified.\n')
def zip_directory(name,directory):
    with zipfile.ZipFile(DIST/name,'w',zipfile.ZIP_DEFLATED,compresslevel=9,strict_timestamps=False) as archive:
        for file in sorted(directory.rglob('*')):
            if file.is_file():archive.write(file,file.relative_to(directory.parent))
if not args.native_only:
    web=DIST/'web'
    if web.exists():shutil.rmtree(web)
    web.mkdir()
    registry=next((Path.home()/'.cargo/registry/src').glob('*/miniquad-0.4.11'))
    (web/'mq_js_bundle.js').write_bytes((registry/'js/gl.js').read_bytes()+b'\n'+(ROOT/'src/audio/web.js').read_bytes())
    shutil.copy2(ROOT/'target/wasm32-unknown-unknown/release/battleblocks.wasm',web/'battleblocks.wasm')
    for file in ['bgm.ogg','crush.wav']:shutil.copy2(ROOT/'assets'/file,web/file)
    (web/'index.html').write_text('''<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>BattleBlocks</title><style>html,body{margin:0;background:#191b26;width:100%;height:100%;overflow:hidden}canvas{width:100%;height:100%;display:block}</style></head><body><canvas id="glcanvas" tabindex="1"></canvas><script src="mq_js_bundle.js"></script><script>load('battleblocks.wasm');</script></body></html>''')
    page=web/'index.html'
    page.write_text(page.read_text().replace('src="mq_js_bundle.js"','src="mq_js_bundle.js?v='+hashlib.sha256((web/'mq_js_bundle.js').read_bytes()).hexdigest()[:12]+'"').replace("load('battleblocks.wasm')","load('battleblocks.wasm?v="+hashlib.sha256((web/'battleblocks.wasm').read_bytes()).hexdigest()[:12]+"')"))
    licenses_for('wasm32-unknown-unknown',web/'licenses');zip_directory('web.zip',web)
    results.update(wasm_bytes=(web/'battleblocks.wasm').stat().st_size,web_startup_gzip_bytes=sum(len(gzip.compress((web/name).read_bytes(),compresslevel=9,mtime=0)) for name in ['index.html','mq_js_bundle.js','battleblocks.wasm','crush.wav']),web_bgm_bytes=(web/'bgm.ogg').stat().st_size,web_zip_bytes=(DIST/'web.zip').stat().st_size)
if not args.web_only:
    executable='battleblocks.exe' if sys.platform=='win32' else 'battleblocks';binary=ROOT/'target/release'/executable
    if sys.platform=='darwin':
        directory=DIST/'BattleBlocks.app'
        if directory.exists():shutil.rmtree(directory)
        mac=directory/'Contents/MacOS';mac.mkdir(parents=True);shutil.copy2(binary,mac/executable)
        (directory/'Contents/Info.plist').write_text('''<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleExecutable</key><string>battleblocks</string><key>CFBundleIdentifier</key><string>local.battleblocks.game</string><key>CFBundleName</key><string>BattleBlocks</string><key>CFBundlePackageType</key><string>APPL</string><key>NSHighResolutionCapable</key><true/></dict></plist>''')
        licenses_for(host,directory/'Contents/Resources/licenses');name='macos-'+platform.machine()+'.zip'
    else:
        directory=DIST/('battleblocks-'+sys.platform+'-'+platform.machine())
        if directory.exists():shutil.rmtree(directory)
        directory.mkdir();shutil.copy2(binary,directory/executable);licenses_for(host,directory/'licenses');name=directory.name+'.zip'
    zip_directory(name,directory);results.update(native_executable_bytes=binary.stat().st_size,native_package_bytes=sum(p.stat().st_size for p in directory.rglob('*') if p.is_file()),native_zip_bytes=(DIST/name).stat().st_size,native_zip=name)
(DIST/'measurements.json').write_text(json.dumps(results,indent=2));print(json.dumps(results,indent=2))
