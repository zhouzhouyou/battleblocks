"""Stable locations for original character artwork."""
LEGACY_ROLES = {'knight': 'knight-legacy', 'elf': 'elf-legacy', 'wizzard': 'wizard', 'goblin': 'goblin-legacy', 'imp': 'imp', 'chort': 'chort', 'big_zombie': 'zombie', 'big_demon_ro': 'demon'}

def legacy_role(name):
    return LEGACY_ROLES[name]
