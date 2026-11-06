"""Check or apply pixel-exact compression of approved runtime WebP assets."""
from pathlib import Path
import argparse
import hashlib
import io
import json
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / '.build-tools/art-site'))
try:
    from PIL import Image, features
except ImportError:
    raise SystemExit('Run tools/build.py --setup-only to initialize the WebP tools.')


def signature(data):
    with Image.open(io.BytesIO(data)) as image:
        frames = []
        for index in range(image.n_frames):
            image.seek(index)
            rgba = image.convert('RGBA')
            frames.append((rgba.tobytes(), image.info.get('duration')))
        return image.size, image.info.get('loop'), frames


def recompress(data):
    with Image.open(io.BytesIO(data)) as image:
        info = image.info.copy()
        frames, durations = [], []
        for index in range(image.n_frames):
            image.seek(index)
            frames.append(image.convert('RGBA'))
            durations.append(image.info.get('duration', 0))
    options = dict(lossless=True, exact=True, method=6, quality=100)
    for name in ['icc_profile', 'exif', 'xmp']:
        if info.get(name):
            options[name] = info[name]
    if len(frames) > 1:
        options.update(save_all=True, append_images=frames[1:], duration=durations,
                       loop=info.get('loop', 0), minimize_size=True)
    output = io.BytesIO()
    frames[0].save(output, 'WEBP', **options)
    candidate = output.getvalue()
    if signature(candidate) != signature(data):
        raise ValueError('WebP recompression changed pixels, dimensions, frame count, timing or loop')
    return candidate


def optimize(root=ROOT, write=False):
    manifest_path = root / 'config/asset-manifest.json'
    manifest = json.loads(manifest_path.read_text())
    locks = json.loads((root / 'config/normal-art-lock.json').read_text())['files']
    cache_path = root / '.local/asset-compression/cache.json'
    fingerprint = f'lossless-exact-m6-q100:{Image.__version__}:{features.version("webp")}'
    cache = json.loads(cache_path.read_text()) if cache_path.exists() else {}
    checked, protected, saved, pending, updates = 0, 0, 0, [], []
    for path in sorted((root / 'assets').rglob('*.webp')):
        name = path.relative_to(root).as_posix()
        if name not in manifest:
            raise ValueError(f'Unapproved runtime resource: {name}; adopt it in the asset manifest first')
        original = path.read_bytes()
        digest = hashlib.sha256(original).hexdigest()
        locked = name in locks
        if locked and digest != locks[name]:
            raise ValueError(f'Protected original artwork changed: {name}')
        key = fingerprint + ('locked:' if locked else '') + digest
        checked += 1
        if cache.get(name) == key:
            protected += int(locked)
            continue
        candidate = recompress(original)
        if locked:
            protected += 1
        elif len(candidate) < len(original):
            if not write:
                pending.append(f'{name}: can save {len(original) - len(candidate)} bytes')
                continue
            saved += len(original) - len(candidate)
            updates.append((path, candidate))
            digest = hashlib.sha256(candidate).hexdigest()
            manifest[name] = dict(sha256=digest, bytes=len(candidate))
            key = fingerprint + digest
        cache[name] = key
    if pending:
        raise ValueError('\n'.join(pending) + '\nRun tools/optimize_webp.py --write, review and validate the asset changes.')
    if write and saved:
        for path, candidate in updates:
            path.write_bytes(candidate)
        manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
    cache_path.parent.mkdir(parents=True, exist_ok=True)
    cache_path.write_text(json.dumps(cache, indent=2) + '\n')
    print(f'PASS: {checked} runtime WebP files; {protected} byte-locked originals; {saved} bytes saved')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--check', action='store_true', help='Reject assets that still have a smaller exact encoding')
    mode.add_argument('--write', action='store_true', help='Apply only smaller exact encodings and update existing manifest entries')
    args = parser.parse_args()
    try:
        optimize(write=args.write)
    except ValueError as error:
        raise SystemExit(str(error))
