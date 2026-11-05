"""Byte, pixel and metadata guard for each Issue 28 normal form."""
from pathlib import Path
import hashlib
import io
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'.build-tools/art-site'))
from PIL import Image
BASE = 'c0aa438fa4077785706a26276795efbb93e886d6'
LOCK = ROOT/'analysis/issue28-awake-animation/research/normal-lock.json'
ROLES = ('knight', 'elf', 'goblin')
sha = lambda data: hashlib.sha256(data).hexdigest()


def check(read=lambda p: (ROOT/p).read_bytes()):
    baseline = lambda p: subprocess.check_output(['git', 'show', f'{BASE}:{p}'], cwd=ROOT)
    lock = json.loads(LOCK.read_text(encoding='utf-8'))
    for name in ROLES:
        for path in (f'asset-sources/{name}.webp', f'assets/{name}.webp', f'assets/{name}.json'):
            actual, expected = read(path), baseline(path)
            if path.endswith('.json'):
                actual, expected = actual.replace(b'\r\n', b'\n'), expected.replace(b'\r\n', b'\n')
            assert actual == expected, f'Normal bytes changed: {path}'
    for path in ('assets/report.json', 'assets/hero-thumbnails.webp', 'assets/heroes.txt',
                 'assets/skills.txt', 'assets/ability-loadouts.txt', 'assets/role-lore.txt', 'src/graphics.rs'):
        # Text is checked with Git's LF convention; image bytes remain exact.
        actual, expected = read(path), baseline(path)
        if not path.endswith('.webp'):
            actual, expected = actual.replace(b'\r\n', b'\n'), expected.replace(b'\r\n', b'\n')
        assert actual == expected, f'Protected file changed: {path}'
    registry = read('src/sprites/registry.rs').decode().replace('\r\n', '\n')
    normal = registry.split('pub const SPRITES:')[1].split('pub const AWAKENED:')[0]
    base_normal = baseline('src/sprites/registry.rs').decode().split('pub const SPRITES:')[1].split('pub const AWAKENED:')[0]
    assert normal == base_normal, 'Normal SPRITES registration changed'
    thumbs = Image.open(io.BytesIO(read('assets/hero-thumbnails.webp'))).convert('RGBA')
    for name, expected in lock['roles'].items():
        meta = json.loads(read(f'assets/{name}.json'))
        for key, value in meta.items():
            assert value == expected['metadata'][key], f'{name}: {key}'
        im = Image.open(io.BytesIO(read(f'assets/{name}.webp'))).convert('RGBA')
        w, h = meta['frame_size']
        hashes = [sha(im.crop((i*w, 0, (i+1)*w, h)).tobytes()) for i in meta['indices']]
        assert hashes == expected['ordered_frame_rgba_sha256'], f'{name}: frame pixels/order'
        assert len(set(hashes)) == expected['unique_frames'], f'{name}: unique poses'
        slot = expected['thumbnail_slot']
        assert sha(thumbs.crop((slot*40, 0, (slot+1)*40, 48)).tobytes()) == expected['thumbnail_rgba_sha256']
    return {'baseline': BASE, 'roles': list(ROLES), 'bytes_pixels_metadata_and_registry': 'pass'}


def mutation_checks():
    mutations = {
        'duration': ('assets/knight.json', lambda b: b.replace(b'0.13', b'0.12')),
        'order': ('assets/goblin.json', lambda b: b.replace(b'2,', b'0,')),
        'pixels': ('assets/elf.webp', lambda b: b[:-1]+bytes([b[-1]^1])),
        'registry': ('src/sprites/registry.rs', lambda b: b.replace(b'w: 300.', b'w: 301.', 1)),
    }
    for name, (path, mutate) in mutations.items():
        original = (ROOT/path).read_bytes()
        changed = mutate(original)
        assert changed != original
        try:
            check(lambda p: changed if p == path else (ROOT/p).read_bytes())
        except AssertionError:
            continue
        raise AssertionError(f'Guard accepted {name} regression')
    return list(mutations)


if __name__ == '__main__':
    result = check()
    result['rejected_mutations'] = mutation_checks()
    print(json.dumps(result, indent=2))
