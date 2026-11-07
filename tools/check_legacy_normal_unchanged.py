"""Verify original normal artwork, timings and registration without old commit IDs."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1]
def registry_digest(data):
    normal=data.decode().replace('\r\n','\n').split('pub const SPRITES:')[1].split('pub const AWAKENED:')[0]
    entries=re.findall(r'SpriteAsset \{.*?\n    \}',normal,re.S)[:7]
    normalized=re.sub(r'include_bytes!\("[^"]+"\)','include_bytes!("RESOURCE")','\n'.join(entries))
    # Web's external-asset wrapper changes storage, not the protected playback metadata.
    normalized=re.sub(r'asset: crate::asset!\("[^"]+"\)', 'bytes: include_bytes!("RESOURCE")', normalized)
    return hashlib.sha256(normalized.encode()).hexdigest()
def check(read=lambda name:(ROOT/name).read_bytes()):
    lock=json.loads((ROOT/'config/normal-art-lock.json').read_text())
    for name,expected in lock['files'].items():
        data=read(name)
        if name.endswith('.json'): data=data.replace(b'\r\n',b'\n')
        assert hashlib.sha256(data).hexdigest()==expected, f'Normal resource changed: {name}'
    registry=read('src/sprites/registry.rs')
    assert registry_digest(registry)==lock['normal_registry_sha256'], 'Normal timing/registration changed'
    normal=registry.decode().split('pub const SPRITES:')[1].split('pub const AWAKENED:')[0]
    paths=re.findall(r'crate::asset!\("([^"]+)"\)',normal)[:7]
    expected_paths=[name.removeprefix('assets/') for name in lock['files'] if name.endswith('.webp')]
    assert paths==expected_paths, 'Normal resource routing changed'
    return {'normal_artwork_bytes_metadata_and_registry':'pass','files':len(lock['files'])}
def mutation_checks():
    changes={
        'duration':('assets-source/roles/knight-legacy/animation/normal.json',lambda b:b.replace(b'0.13',b'0.12')),
        'order':('assets-source/roles/goblin-legacy/animation/normal.json',lambda b:b.replace(b'2,',b'0,')),
        'pixels':('assets/roles/elf-legacy/runtime/normal.webp',lambda b:b[:-1]+bytes([b[-1]^1])),
        'registry':('src/sprites/registry.rs',lambda b:b.replace(b'w: 300.',b'w: 301.',1)),
        'routing':('src/sprites/registry.rs',lambda b:b.replace(b'roles/imp/runtime/normal.webp',b'roles/chort/runtime/normal.webp',1)),
    }
    for label,(path,mutate) in changes.items():
        original=(ROOT/path).read_bytes();changed=mutate(original);assert changed!=original
        try:check(lambda name:changed if name==path else (ROOT/name).read_bytes())
        except AssertionError:continue
        raise AssertionError(f'Guard accepted {label} regression')
    return list(changes)
if __name__=='__main__':
    result=check();result['rejected_mutations']=mutation_checks();print(json.dumps(result,indent=2))
