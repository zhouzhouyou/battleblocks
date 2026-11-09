"""Build-time provenance for the executable, never inferred from the runtime checkout."""
import hashlib
import json
from pathlib import Path
import sys


def digest(value):
    return hashlib.sha256(value).hexdigest()


def manifest(root, target):
    # Conservative shared invalidation: all Rust changes, dependencies and skill
    # definitions. Only hero/loadout rows are proven local to the two combatants.
    paths = sorted((root / "src").rglob("*.rs"))
    paths += [root / p for p in ("Cargo.toml", "Cargo.lock", "build.rs",
                                "tools/balance_fingerprint.py", "config/skills.txt", "config/projectiles.txt")]
    shared = hashlib.sha256(target.encode())
    for path in paths:
        shared.update(path.relative_to(root).as_posix().encode() + b"\0")
        shared.update(path.read_bytes().replace(b"\r\n", b"\n") + b"\0")
    def rows(name):
        return [line.strip() for line in (root / "config" / name).read_text(encoding="utf-8").splitlines()
                if line.strip() and not line.startswith("#")]
    loadouts = {int(line.split("|", 1)[0]): line for line in rows("ability-loadouts.txt")}
    heroes = []
    for i, row in enumerate(rows("heroes.txt")):
        heroes.append({"id": i, "name": row.split("|")[0],
                       "fingerprint": digest((row + "\n" + loadouts[i]).encode())})
    return {"schema": 1, "shared": shared.hexdigest(), "target": target, "heroes": heroes}


if __name__ == "__main__":
    Path(sys.argv[2]).write_text(json.dumps(manifest(Path(sys.argv[1]), sys.argv[3]),
                                          ensure_ascii=True), encoding="utf-8")
