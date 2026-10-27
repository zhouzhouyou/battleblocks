"""Compare release packages with PNG, PNG+WebP decoder, and lossless WebP.

Uses isolated copies under .build-tools/image-formats; never edits game sources.
Run with the Python recorded in .build-tools/python-path.txt (Pillow required).
"""
from pathlib import Path
import argparse
import gzip
import json
import os
import re
import shutil
import subprocess
import sys
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / '.build-tools/image-formats'
CARGO = shutil.which('cargo') or str(Path.home() / '.cargo/bin/cargo')
WEBP_DEP = '\n# Unify the existing image decoder dependency; no new crate.\nimage = { version = "=0.24.9", default-features = false, features = ["webp"] }\n'


def run_variant(name, webp, decoder, png_ref):
    folder = WORK / name
    if folder.exists():
        shutil.rmtree(folder)
    folder.mkdir(parents=True)
    for filename in ['Cargo.toml', 'Cargo.lock', 'build.rs']:
        shutil.copy2(ROOT / filename, folder / filename)
    for directory in ['src', 'assets', 'tools']:
        shutil.copytree(ROOT / directory, folder / directory)
    (folder / '.build-tools').symlink_to(ROOT / '.build-tools', target_is_directory=True)
    if png_ref:
        for candidate in (folder / 'assets').rglob('*.webp'):
            candidate.unlink()
        paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', png_ref, '--', 'assets'], cwd=ROOT, text=True).splitlines()
        for path in paths:
            if path.endswith('.png'):
                destination = folder / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(subprocess.check_output(['git', 'show', f'{png_ref}:{path}'], cwd=ROOT))
    # Shared output cache avoids rebuilding all dependencies for each variant.
    (folder / 'target').symlink_to(WORK / 'target', target_is_directory=True)
    manifest = (folder / 'Cargo.toml').read_text()
    # The production manifest may already select WebP after an earlier comparison.
    manifest = re.sub(r'\n# Unify the existing image decoder dependency; no new crate\.\n', '\n', manifest)
    manifest = re.sub(r'^image = .*\n', '', manifest, flags=re.M)
    if decoder:
        manifest = manifest.replace('\n[target.', WEBP_DEP + '\n[target.', 1)
    (folder / 'Cargo.toml').write_text(manifest)
    image_sizes = []
    for source in sorted((folder / 'assets').rglob('*.png')):
        image = Image.open(source).convert('RGBA')
        candidate = source.with_suffix('.webp')
        image.save(candidate, lossless=True, exact=True, method=6)
        decoded = Image.open(candidate).convert('RGBA')
        assert decoded.size == image.size and decoded.tobytes() == image.tobytes(), source
        image_sizes.append({'path': str(source.relative_to(folder)), 'png': source.stat().st_size, 'webp': candidate.stat().st_size})
        if not webp:
            candidate.unlink()
    for source in (folder / 'src').rglob('*.rs'):
        text = source.read_text()
        text = text.replace('Some(ImageFormat::Png)', 'None')
        text = re.sub(r'(include_bytes!\("[^"\n]+)\.webp("\))', r'\1.png\2', text)
        if webp:
            text = re.sub(r'(include_bytes!\("[^"\n]+)\.png("\))', r'\1.webp\2', text)
        (source).write_text(text)
    # Only root dependency metadata changes; every transitive version stays pinned.
    lock = (folder / 'Cargo.lock').read_text()
    root_pattern = r'(name = "battleblocks"\n.*?dependencies = \[)(.*?)(\n\])'
    def root_dependencies(match):
        dependencies = match[2].replace('\n "image",', '')
        if decoder:
            dependencies = dependencies.replace('\n "cpal",', '\n "cpal",\n "image",')
        return match[1] + dependencies + match[3]
    lock = re.sub(root_pattern, root_dependencies, lock, count=1, flags=re.S)
    (folder / 'Cargo.lock').write_text(lock)
    env = {**os.environ, 'CARGO_TARGET_DIR': str(WORK / 'target')}
    for args in [['build', '--release', '--locked', '--bin', 'battleblocks'],
                 ['build', '--release', '--locked', '--bin', 'battleblocks', '--target', 'wasm32-unknown-unknown']]:
        subprocess.run([CARGO, *args], cwd=folder, env=env, check=True)
    subprocess.run([sys.executable, 'tools/package.py'], cwd=folder, env=env, check=True)
    result = json.loads((folder / 'dist/measurements.json').read_text())
    result['images'] = image_sizes
    result['image_bytes'] = sum(row['webp' if webp else 'png'] for row in image_sizes)
    result['native_gzip_bytes'] = len(gzip.compress((folder / 'target/release/battleblocks').read_bytes(), mtime=0))
    result['wasm_gzip_bytes'] = len(gzip.compress((folder / 'target/wasm32-unknown-unknown/release/battleblocks.wasm').read_bytes(), mtime=0))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--png-ref', help='Commit containing the original PNG inputs, required after production assets use WebP')
    args = parser.parse_args()
    if not args.png_ref and not list((ROOT / 'assets').rglob('*.png')):
        parser.error('Supply --png-ref with the last PNG commit')
    WORK.mkdir(parents=True, exist_ok=True)
    result = {'method': 'All 24 runtime PNG images, including original sprite atlases, tiles, defeat screen and awakening skins. Lossless WebP method=6, exact RGBA equality; identical release settings and source code in isolated copies.', 'variants': {}}
    for name, webp, decoder in [('png', False, False), ('png_webp_decoder', False, True), ('webp', True, True)]:
        print('MEASURING', name, flush=True)
        result['variants'][name] = run_variant(name, webp, decoder, args.png_ref)
        (WORK / 'results.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({name: {k: v for k, v in row.items() if k != 'images'} for name, row in result['variants'].items()}, indent=2))


if __name__ == '__main__':
    main()
