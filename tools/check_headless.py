"""Verify the canonical headless model and debug console without opening a window."""
from pathlib import Path
import json, os, subprocess, tempfile

ROOT = Path(__file__).resolve().parents[1]
BIN = Path(os.environ.get("BATTLEBLOCKS_BIN", ROOT / "target/debug/battleblocks"))
clean = [[(x + 2 * y) % 7 for x in range(8)] for y in range(8)]
move_board = [row[:] for row in clean]
move_board[0][:4] = [2, 1, 2, 2]
move_board[1][1] = 2
match_board = [row[:] for row in clean]
match_board[0][:3] = [0, 0, 0]
commands = [
    "pause on",
    "ai 2",
    "lang zh",
    "status",
    "hero player 0",
    "hero enemy 1",
    "hp player 650",
    "mp player 300",
    "seed 12345",
    "shuffle",
    "tile 1 1 0",
    "buff player bonus 1",
    "buff player armed on",
    "buff player bonus 0",
    "buff player armed off",
    "board load " + " ".join(str(v) for row in move_board for v in row),
    "pause off",
    "move 2 1 2 2",
    "undo player",
    "mp player 300",
    "cast 1 2 3",
    "undo player",
    "pause on",
    "turn enemy",
    "pause off",
    "pause on",
    "board load " + " ".join(str(v) for row in match_board for v in row),
    "resolve",
    "pause off",
    "status",
    "restart",
]
with tempfile.TemporaryDirectory(prefix="battleblocks-headless-") as temp:
    log = Path(temp) / "events.jsonl"
    script = Path(temp) / "script.txt"
    script.write_text("\n".join(commands) + "\n")
    result = subprocess.run(
        [str(BIN), "--headless", "--log", str(log), "--script", str(script)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=30,
    )
    assert result.returncode == 0, result.stderr + result.stdout
    assert "HEADLESS_OK" in result.stdout, result.stdout
    assert "ai=2" in result.stdout, result.stdout
    records = [json.loads(line) for line in log.read_text().splitlines() if line]
    assert records and records[0]["event"] == "session_start"
    edits = [e["detail"] for e in records if e["event"] == "debug_edit"]
    for expected in (
        "hero player 0",
        "hero enemy 1",
        "hp player 650",
        "mp player 300",
        "seed 12345",
        "shuffle",
        "tile 1 1 0",
        "buff player bonus 1",
        "buff player armed on",
        "turn enemy",
        "resolve",
    ):
        assert expected in edits, (expected, edits)
    move = next(e for e in records if e["event"] == "move" and e["actor"] == "player")
    assert any(e["event"] == "resolution" and e["action"] == move["action"] for e in records)
    rollbacks = [e for e in records if e["event"] == "rollback"]
    assert rollbacks and rollbacks[0]["after"] == move["before"]
    cast = next(e for e in records if e["event"] == "skill" and "Charge" in e["detail"])
    assert cast["before"]["mp"] - cast["after"]["mp"] == 100
    assert cast["after"]["cooldowns"][0][0] == 2
    hit = next(e for e in records if e["event"] == "skill_hit" and e["action"] == cast["action"])
    assert "actual_damage=" in hit["detail"]
    assert any(e["event"] == "restart" for e in records)
    assert "调试控制台" not in result.stdout
    assert "reward_event" not in result.stdout

    batch = subprocess.run(
        [
            str(BIN),
            "--headless",
            "--autoplay",
            "--samples",
            "1",
            "--seed",
            "77",
            "--level",
            "1",
            "--roster",
            "8,7",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=30,
    )
    assert batch.returncode == 0, batch.stderr + batch.stdout
    assert "hero,wins,games,win_percent" in batch.stdout, batch.stdout
    assert "games=2" in batch.stdout, batch.stdout
    assert "roster=Dwarf,Knight" in batch.stdout, batch.stdout
    assert "pairwise_win_counts=" in batch.stdout, batch.stdout

    print(
        "PASS: headless core generated "
        f"{len(records)} JSONL events across console edits, move, cast, rollback, AI turn and resolve; "
        "autoplay produced a four-column CSV for two matches"
    )
