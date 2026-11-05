"""Repository-wide distinction between reproduction inputs and generated output."""
from pathlib import Path
import re

ONE_OFF_DOCUMENTS = {
    'docs/REPOSITORY-CLEANUP.md', 'docs/MEMORY_ANALYSIS.md',
    'docs/art/ASSET-INVENTORY.md',
    'docs/AWAKENING-BALANCE.md',
    'docs/AWAKENING.md',
    'docs/PASSIVE-ABILITIES.md',
    'docs/PASSIVES.md',
    'docs/BALANCE.md',
    'docs/ASSET_RENDERING.md', 'docs/AUDIO_OPTIMIZATION.md', 'docs/IMAGE-FORMATS.md',
    'experiments/memory-comparison/REPORT.md', 'experiments/memory-comparison/AUDIT.md',
}


DUPLICATE_RESOURCES = {
    'assets-source/audio/crush.wav',
    'assets-source/ui/balanced.webp',
    'assets-source/ui/damage.webp',
    'assets-source/ui/dm.webp',
    'assets-source/ui/hd.webp',
    'assets-source/ui/hm.webp',
    'assets-source/ui/hp.webp',
    'assets-source/ui/mp.webp',
    'experiments/memory-comparison/assets/bgm.ogg',
}


def output_violation(name, inputs):
    path = Path(name)
    if name.startswith(('analysis/', '.local/', 'logs/', 'dist/', 'target/', '.build-tools/')):
        return 'generated output directory'
    if name in DUPLICATE_RESOURCES:
        return 'duplicate of canonical assets resource'
    if name in ONE_OFF_DOCUMENTS:
        return 'historical, duplicate or one-off document'
    if name.startswith('experiments/') and any(part == 'data' or part.startswith('results') for part in path.parts):
        return 'experiment run output'
    if path.suffix.lower() in {'.log', '.jsonl', '.csv', '.sqlite', '.db'} or name.endswith('.jsonl.gz'):
        return 'log, raw sample or runtime database'
    if path.suffix.lower() == '.json' and name not in inputs:
        return 'undeclared structured file (only documented reproduction inputs are allowed)'
    if name.startswith(('docs/', 'openspec/')) and (name.startswith('openspec/') or any(part.lower() in {'plans', 'plan', 'openspec', 'proposals'} for part in path.parts) or re.search(r'cleanup|single-run|run-report|audit-report|(?:^|[-_])plan(?:$|[-_])', path.stem, re.I)):
        return 'one-off execution record in docs'
    return None
