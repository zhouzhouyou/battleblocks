"""Measure unique reachable Git blobs, grouped by resource purpose."""
import argparse
from collections import defaultdict
import json
from pathlib import Path
import subprocess


def category(name):
    parts = Path(name).parts
    if name.startswith('assets/roles/'):
        return 'assets/roles/' + (parts[3] if len(parts) > 4 else 'other')
    if name.startswith('assets/'):
        return '/'.join(parts[:2])
    if name.startswith('assets-source/'):
        return '/'.join(parts[:2])
    if name.startswith('asset-sources/'):
        return 'old asset-sources'
    if name.startswith('analysis/'):
        return 'analysis/media' if Path(name).suffix in {'.webp', '.png', '.gif', '.mp4', '.ogg', '.wav', '.ttf', '.gz'} else 'analysis/records'
    return parts[0] if len(parts) > 1 else 'root files'


def audit(ref):
    lines = subprocess.check_output(['git', 'rev-list', '--objects', ref], text=True).splitlines()
    paths = {}
    for line in lines:
        oid, _, name = line.partition(' ')
        paths.setdefault(oid, name)
    objects = subprocess.check_output(['git', 'cat-file', '--batch-check=%(objectname) %(objecttype) %(objectsize)'],
                                      input='\n'.join(paths) + '\n', text=True).splitlines()
    groups = defaultdict(lambda: {'bytes': 0, 'blobs': 0})
    largest = []
    for line in objects:
        oid, kind, size = line.split()
        if kind != 'blob':
            continue
        size = int(size)
        name = paths[oid]
        group = groups[category(name)]
        group['bytes'] += size
        group['blobs'] += 1
        largest.append({'path': name, 'bytes': size, 'oid': oid})
    return {'ref': ref, 'commit': subprocess.check_output(['git', 'rev-parse', ref], text=True).strip(),
            'commits': int(subprocess.check_output(['git', 'rev-list', '--count', ref])),
            'unique_blob_bytes': sum(g['bytes'] for g in groups.values()),
            'groups': dict(sorted(groups.items(), key=lambda kv: -kv[1]['bytes'])),
            'largest_blobs': sorted(largest, key=lambda x: -x['bytes'])[:25]}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ref', default='HEAD')
    parser.add_argument('--out', type=Path)
    args = parser.parse_args()
    result = audit(args.ref)
    text = json.dumps(result, ensure_ascii=False, indent=2)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(text + '\n')
    print(text)
