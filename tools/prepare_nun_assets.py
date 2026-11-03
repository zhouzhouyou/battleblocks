"""Compatibility entry point for the current 12-pose production importer.

Historical 4-pose exporters and their source files remain available in Git.
The dwarf book and nun standees are unchanged independent assets.
"""
from prepare_character_animations import import_role, registry
from prepare_auxiliary_role_assets import import_standees

if __name__ == "__main__":
    import_standees()
    import_role('nun')
    registry()
