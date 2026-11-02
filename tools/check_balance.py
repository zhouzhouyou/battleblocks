"""Real-process checks of the evaluation adapter, replay, cache and error paths."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

from balance.engine import Engine

ROOT = Path(__file__).resolve().parents[1]


def main():
    binary = ROOT / "target/release" / ("battleblocks.exe" if sys.platform == "win32" else "battleblocks")
    engine = Engine(binary, 2)
    jobs = [((7, 8), 1200042, 1, 20000), ((9, 11), 1200043, 2, 20000)]
    first = engine.blocks(jobs)
    assert first == engine.blocks(list(reversed(jobs)))[::-1], "process ordering changed seeded outcomes"
    assert all(game["termination"] == "win" for pair in first for game in pair)
    stalled = engine.block((7, 8), 23, 1, 1)
    assert all(game["termination"] == "aborted" for game in stalled)
    for text in ("7 8 1 3 100\n", "7 8 0 1 100\n", "7 7 1 1 100\n"):
        result = subprocess.run([str(binary), "--headless", "--evaluate-jobs"], input=text, text=True, capture_output=True, timeout=30)
        assert result.returncode == 2 and "EVALUATION_ERROR" in result.stderr
        assert not result.stdout
    with tempfile.TemporaryDirectory() as tmp:
        directory = Path(tmp)
        command = [sys.executable, str(ROOT / "tools/evaluate_balance.py"), "--binary", str(binary),
                   "--roster", "7,8,11", "--budget", "12", "--seed", "1290001",
                   "--cache", str(directory / "cache.sqlite"), "--out", str(directory / "out")]
        subprocess.run(command, check=True, timeout=120)
        path = directory / "out/report.json"
        initial = json.loads(path.read_text(encoding="utf-8"))
        subprocess.run(command, check=True, timeout=120)
        replay = json.loads(path.read_text(encoding="utf-8"))
        assert initial["new_games"] == 12 and replay["new_games"] == 0
        assert replay["reused_games"] == 12 and initial["heroes"] == replay["heroes"]
        failed = subprocess.run(command + ["--budget", "13"], capture_output=True, timeout=30)
        assert failed.returncode == 2
        failed = subprocess.run(command + ["--max-steps", "1"], capture_output=True, timeout=120)
        assert failed.returncode == 2
        aborted = json.loads(path.read_text(encoding="utf-8"))
        assert aborted["aborted"] == 12 and all(h["status"] == "needs_samples" for h in aborted["heroes"])
    print("PASS: canonical jobs, per-process replay, step-limit distinction, invalid inputs, CLI budget/cache/reports")


if __name__ == "__main__":
    main()
