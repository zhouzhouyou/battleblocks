"""Check tracked resources against the approved final binary inventory."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
from repository_policy import output_violation

ROOT = Path(__file__).resolve().parents[1]
MEDIA = {'.webp', '.png', '.jpg', '.jpeg', '.gif', '.mp4', '.mov', '.wav', '.ogg', '.ttf', '.woff', '.woff2', '.zip', '.gz', '.pdf'}


def check(staged=False):
    paths = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).decode().split('\0')[:-1]
    def read(name):
        if staged:
            return subprocess.check_output(['git', 'show', ':' + name], cwd=ROOT)
        return (ROOT / name).read_bytes()

    def exists(name):
        return name in paths if staged else (ROOT / name).exists()

    approved = json.loads(read('config/asset-manifest.json'))
    inputs = json.loads(read('config/repository-inputs.json'))
    errors = []
    for entry in ['README.md', 'AGENTS.md']:
        if not exists(entry) or 'docs/LAYOUT.md' not in read(entry).decode('utf-8'):
            errors.append(f'{entry} must link docs/LAYOUT.md')
    if not exists('docs/LAYOUT.md'):
        errors.append('Missing authoritative docs/LAYOUT.md')
    actual = set()
    runtime_hashes = {meta['sha256'] for name, meta in approved.items() if name.startswith('assets/')}
    for name in paths:
        path = Path(name)
        reason = output_violation(name, inputs)
        if reason:
            errors.append(f'Forbidden generated/one-off file: {name}: {reason}')
        if name.startswith(('asset-sources/', 'assets/awakened/', '.local/')) or 'references' in path.parts:
            errors.append(f'Forbidden location: {name}')
        if name.startswith('assets/') and (len(path.parts) < 3 or path.parts[1] not in {'roles', 'effects', 'environment', 'ui', 'audio'}):
            errors.append(f'Unclassified asset: {name}')
        if name.startswith('assets/') and any(part in {'source', 'design', 'animation', 'gallery'} for part in path.parts):
            errors.append(f'Non-runtime resource in assets: {name}')
        if path.suffix.lower() in MEDIA:
            actual.add(name)
            if name in approved and not name.startswith('assets/') and approved[name]['sha256'] in runtime_hashes:
                errors.append(f'Duplicate of canonical assets content: {name}; reference assets directly')
            if re.search(r'preview|contact|candidate|comparison|reference', path.name, re.I):
                errors.append(f'Review/reference media: {name}')
            if name not in approved:
                errors.append(f'Unapproved binary: {name}')
            elif not exists(name) or hashlib.sha256(read(name)).hexdigest() != approved[name]['sha256']:
                errors.append(f'Binary differs from inventory: {name}')
        if name.startswith('docs/') and path.suffix == '.md' and exists(name):
            if re.search(r'(?i)(?<![a-z])v\d+', name + '\n' + read(name).decode('utf-8')):
                errors.append(f'Iteration identifier in documentation: {name}')
    errors.extend(f'Missing declared reproduction input: {name}' for name in sorted(set(inputs) - set(paths)))
    errors.extend(f'Missing tracked binary: {name}' for name in sorted(set(approved) - actual))
    if errors:
        raise SystemExit('\n'.join(errors))
    target = 'staged' if staged else 'tracked'
    print(f'PASS: {len(paths)} {target} files; {len(actual)} approved binaries; no forbidden files')


def check_history():
    approved = json.loads((ROOT / 'config/asset-manifest.json').read_text())
    ids = {}
    for name in approved:
        contents = (ROOT / name).read_bytes()
        ids[name] = hashlib.sha1(b'blob ' + str(len(contents)).encode() + b'\0' + contents).hexdigest()
    inputs = json.loads((ROOT / 'config/repository-inputs.json').read_text())
    commits = subprocess.check_output(['git', 'rev-list', 'HEAD'], cwd=ROOT, text=True).splitlines()
    seen = set()
    for commit in commits:
        tree = subprocess.check_output(['git', 'ls-tree', '-r', '-z', commit], cwd=ROOT).split(b'\0')
        for entry in tree:
            if not entry:
                continue
            metadata, name = entry.split(b'\t', 1)
            name = name.decode()
            oid = metadata.split()[2].decode()
            reason = output_violation(name, inputs)
            if reason:
                raise SystemExit(f'Forbidden generated/one-off history: {commit} {name}: {reason}')
            if name.startswith(('asset-sources/', 'assets/awakened/', '.local/')) or '/references/' in name or name == 'docs/art/ART-DIRECTION-F-ARCHIVE.md':
                raise SystemExit(f'Forbidden path in history: {commit} {name}')
            if Path(name).suffix.lower() in MEDIA:
                if ids.get(name) != oid:
                    raise SystemExit(f'Unapproved binary revision in history: {commit} {name}')
                seen.add((name, oid))
    print(f'PASS: all {len(commits)} historical trees; {len(seen)} final binary paths; zero rejected resource revisions')


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--history', action='store_true')
    parser.add_argument('--staged', action='store_true', help='Check index contents before committing')
    args = parser.parse_args()
    if args.staged and args.history:
        parser.error('--staged and --history are separate checks')
    check(staged=args.staged)
    if args.history:
        check_history()
