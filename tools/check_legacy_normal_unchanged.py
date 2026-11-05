"""Verify original normal artwork, timings and registration without old commit IDs."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1]
def registry_digest(data):
    normal=data.decode().split('pub const SPRITES:')[1].split('pub const AWAKENED:')[0]
    entries=re.findall(r'SpriteAsset \{.*?\n    \}',normal,re.S)[:7]
    normalized=re.sub(r'include_bytes!\("[^"]+"\)','include_bytes!("RESOURCE")','\n'.join(entries))
    return hashlib.sha256(normalized.encode()).hexdigest()
def check(read=lambda name:(ROOT/name).read_bytes()):
    lock=json.loads((ROOT/'config/normal-art-lock.json').read_text())
    for name,expected in lock['files'].items():
        assert hashlib.sha256(read(name)).hexdigest()==expected, f'Normal resource changed: {name}'
    assert registry_digest(read('src/sprites/registry.rs'))==lock['normal_registry_sha256'], 'Normal timing/registration changed'
    return {'normal_artwork_bytes_metadata_and_registry':'pass','files':len(lock['files'])}
def mutation_checks():
    changes={
        'duration':('assets-source/roles/knight-legacy/animation/normal.json',lambda b:b.replace(b'0.13',b'0.12')),
        'order':('assets-source/roles/goblin-legacy/animation/normal.json',lambda b:b.replace(b'2,',b'0,')),
        'pixels':('assets/roles/elf-legacy/runtime/normal.webp',lambda b:b[:-1]+bytes([b[-1]^1])),
        'registry':('src/sprites/registry.rs',lambda b:b.replace(b'w: 300.',b'w: 301.',1)),
    }
    for label,(path,mutate) in changes.items():
        original=(ROOT/path).read_bytes();changed=mutate(original);assert changed!=original
        try:check(lambda name:changed if name==path else (ROOT/name).read_bytes())
        except AssertionError:continue
        raise AssertionError(f'Guard accepted {label} regression')
    return list(changes)
if __name__=='__main__':
    result=check();result['rejected_mutations']=mutation_checks();print(json.dumps(result,indent=2))
