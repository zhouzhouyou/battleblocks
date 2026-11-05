"""Compare normal artwork bytes and text metadata with the pre-Issue-27 baseline."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--baseline', default='500673e2326acffb7ff0fbf971607f3c55ae2b53')
parser.add_argument('--candidate', help='Git revision; default checks current files')
args = parser.parse_args()


def read(path, revision=None):
    return subprocess.check_output(['git','show',f'{revision}:{path}'],cwd=ROOT) if revision else (ROOT/path).read_bytes()


report = {'baseline': args.baseline, 'candidate': args.candidate or 'working-tree', 'files': {}}
failed = []
for path in ('assets/wizzard.webp','assets/wizzard.json','assets/hero-thumbnails.webp'):
    before,after = read(path,args.baseline),read(path,args.candidate)
    if path.endswith('.json'):
        # Git stores LF; a Windows checkout/export may use CRLF.
        before,after = before.replace(b'\r\n',b'\n'),after.replace(b'\r\n',b'\n')
    equal = before == after
    report['files'][path] = {'unchanged': equal, 'comparison':'LF-normalized text' if path.endswith('.json') else 'exact bytes',
                           'sha256': hashlib.sha256(after).hexdigest()}
    if not equal: failed.append(path)
before,after = (read('src/sprites/registry.rs',ref).decode().replace('\r','').split('pub const AWAKENED:')[0]
                for ref in (args.baseline,args.candidate))
report['normal_registry_unchanged'] = before == after
if before != after: failed.append('normal SPRITES registry')
report['passed'] = not failed
report['changed'] = failed
print(json.dumps(report,ensure_ascii=False,indent=2))
raise SystemExit(1 if failed else 0)
