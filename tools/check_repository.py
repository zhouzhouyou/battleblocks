"""Check current resources and historically approved resources without retaining obsolete files."""
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
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=ROOT)

    inventories = {}
    introductions = {}
    digests = {}

    def inventory(commit, tree, path):
        oid = tree.get(path)
        if oid is None:
            # Older commits predate declarations. Deleting an established inventory
            # must never turn off checks or fall back to today's inventory.
            if path not in introductions:
                additions = git('log', '--reverse', '--diff-filter=A', '--format=%H',
                                'HEAD', '--', path).decode().splitlines()
                if not additions:
                    raise SystemExit(f'Missing historical inventory: {commit} {path}')
                first = additions[0]
                ancestors = set(git('rev-list', first).decode().splitlines())
                introductions[path] = (git('rev-parse', f'{first}:{path}').decode().strip(), ancestors)
            oid, ancestors = introductions[path]
            if commit not in ancestors:
                raise SystemExit(f'Missing historical inventory: {commit} {path}')
        if oid not in inventories:
            inventories[oid] = json.loads(git('cat-file', 'blob', oid))
        return inventories[oid]

    commits = git('rev-list', 'HEAD').decode().splitlines()
    snapshots = []
    approved_revisions = set()
    for commit in commits:
        tree = {}
        for entry in git('ls-tree', '-r', '-z', commit).split(b'\0'):
            if not entry:
                continue
            metadata, name = entry.split(b'\t', 1)
            tree[name.decode()] = metadata.split()[2].decode()
        approved = inventory(commit, tree, 'config/asset-manifest.json')
        inputs = inventory(commit, tree, 'config/repository-inputs.json')
        snapshots.append((commit, tree, inputs))
        for name, oid in tree.items():
            if Path(name).suffix.lower() not in MEDIA:
                continue
            if oid not in digests:
                contents = git('cat-file', 'blob', oid)
                digests[oid] = (hashlib.sha256(contents).hexdigest(), len(contents))
            meta = approved.get(name, {})
            if ('config/asset-manifest.json' in tree
                    and (meta.get('sha256'), meta.get('bytes')) == digests[oid]):
                approved_revisions.add((name, oid))

    # Existing history was losslessly optimized before some inventories were
    # corrected. Require an actual snapshot where file and inventory agree;
    # a stale or fictitious inventory entry alone cannot approve any revision.
    seen = set()
    for commit, tree, inputs in snapshots:
        for name, oid in tree.items():
            reason = output_violation(name, inputs)
            if reason:
                raise SystemExit(f'Forbidden generated/one-off history: {commit} {name}: {reason}')
            if name.startswith(('asset-sources/', 'assets/awakened/', '.local/')) or '/references/' in name or name == 'docs/art/ART-DIRECTION-F-ARCHIVE.md':
                raise SystemExit(f'Forbidden path in history: {commit} {name}')
            if Path(name).suffix.lower() in MEDIA:
                if re.search(r'preview|contact|candidate|comparison|reference', Path(name).name, re.I):
                    raise SystemExit(f'Review/reference media in history: {commit} {name}')
                if (name, oid) not in approved_revisions:
                    raise SystemExit(f'Unapproved binary revision in history: {commit} {name}')
                seen.add((name, oid))
    print(f'PASS: all {len(commits)} historical trees; {len(seen)} approved binary revisions; no forbidden history')


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
