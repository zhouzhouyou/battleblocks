"""Stable locations for original character artwork."""
LEGACY_ROLES = {'knight': 'knight-legacy', 'elf': 'elf-legacy', 'wizzard': 'wizard', 'goblin': 'goblin-legacy', 'imp': 'imp', 'chort': 'chort', 'big_zombie': 'zombie', 'big_demon_ro': 'demon'}

def legacy_role(name):
    return LEGACY_ROLES[name]

from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def report_path(role):
    directory = ROOT / '.local/production' / role
    directory.mkdir(parents=True, exist_ok=True)
    return directory / 'report.json'
