"""Repository-wide distinction between reproduction inputs and generated output."""
from pathlib import Path
import re

ONE_OFF_DOCUMENTS = {
    'docs/REPOSITORY-CLEANUP.md', 'docs/MEMORY_ANALYSIS.md',
    'docs/ASSET_RENDERING.md', 'docs/AUDIO_OPTIMIZATION.md', 'docs/IMAGE-FORMATS.md',
    'experiments/memory-comparison/REPORT.md', 'experiments/memory-comparison/AUDIT.md',
}


def output_violation(name, inputs):
    path = Path(name)
    if name.startswith(('analysis/', '.local/', 'logs/', 'dist/', 'target/', '.build-tools/')):
        return 'generated output directory'
    if name in ONE_OFF_DOCUMENTS:
        return 'one-off report or cleanup record'
    if name.startswith('experiments/') and any(part == 'data' or part.startswith('results') for part in path.parts):
        return 'experiment run output'
    if path.suffix.lower() in {'.log', '.jsonl', '.csv', '.sqlite', '.db'} or name.endswith('.jsonl.gz'):
        return 'log, raw sample or runtime database'
    if path.suffix.lower() == '.json' and name not in inputs:
        return 'undeclared structured file (only documented reproduction inputs are allowed)'
    if name.startswith('docs/') and re.search(r'cleanup|single-run|run-report|audit-report', path.stem, re.I):
        return 'one-off execution record in docs'
    return None
