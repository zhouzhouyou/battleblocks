"""Exercise portal setup, gravity, passive sources and rollback through the canonical CLI."""
from pathlib import Path
import json, os, subprocess, tempfile
ROOT = Path(__file__).resolve().parents[1]
BIN = Path(os.environ.get('BATTLEBLOCKS_BIN', ROOT / 'target/debug/battleblocks'))
clean = [[(x + 2*y) % 7 for x in range(8)] for y in range(8)]
with tempfile.TemporaryDirectory(prefix='battleblocks-portals-') as directory:
    log = Path(directory) / 'events.jsonl'
    # All casts use normal costs/legality; no duplicate gravity or reward model here.
    commands = ['pause on', 'hero player 12', 'hero enemy 7', 'mp player 160',
                'board load ' + ' '.join(str(v) for row in clean for v in row),
                'pause off', 'cast 1 2 3 2 5', 'cast 1 2 3 5 5',
                'cast 2 1 1', 'undo player', 'cast 2 8 1', 'undo player', 'undo player']
    args = [str(BIN), '--headless', '--log', str(log)]
    for command in commands:
        args += ['--command', command]
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    records = [json.loads(line) for line in log.read_text().splitlines()]
    casts = [e for e in records if e['event'] == 'skill']
    assert len(casts) == 3, [e['detail'] for e in casts]
    portal, up, right = casts
    assert portal['after']['mp'] == 105 and not portal['after']['action_done']
    assert portal['before']['board'] == portal['after']['board']
    gates = portal['after']['board_mechanics']['portals']
    assert gates == [{'a': [1, 2, True], 'b': [4, 4, True], 'owner': False}], gates
    assert up['after']['board_mechanics']['gravity'] == 'Up'
    assert up['after']['board_mechanics']['portals'] == gates
    assert up['after']['mp'] == 80
    assert right['after']['board_mechanics']['gravity'] == 'Right'
    assert right['after']['board_mechanics']['portals'] == []
    undos = [e for e in records if e['event'] == 'rollback']
    assert len(undos) == 3
    assert undos[0]['after'] == up['before']
    assert undos[1]['after'] == right['before']
    assert undos[2]['after'] == portal['before']
    # Create a real matching wave below a gate and observe passive reward provenance.
    wave = [row[:] for row in clean]
    wave[3][1:4] = [2, 2, 2]
    wave[4][4] = 0
    commands = ['pause on', 'hero player 12', 'hero enemy 7', 'mp player 160',
                'board load ' + ' '.join(str(v) for row in clean for v in row),
                'pause off', 'cast 1 2 3 5 5', 'pause on',
                'board load ' + ' '.join(str(v) for row in wave for v in row),
                'resolve', 'pause off']
    args = [str(BIN), '--headless', '--log', str(log)]
    for command in commands:
        args += ['--command', command]
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    records = [json.loads(line) for line in log.read_text().splitlines()]
    rewards = [e['payload'] for e in records if e['event'] == 'reward_event'
               and e['payload']['source'].get('id') == 'PortalYield']
    assert rewards, 'no passive yield from actual gravity resolution'
    assert all(e['source_chain'][0]['source']['kind'] == 'portal_transit' for e in rewards)
    assert all(e['amount'] == 1 for e in rewards)
    # Placing gates on a stable board must immediately resolve the new remote line.
    remote = [row[:] for row in clean]
    for x, y in [(1, 1), (1, 2), (4, 5)]:
        remote[y][x] = 6
    commands = ['pause on', 'hero player 12', 'hero enemy 12', 'mp player 160',
                'board load ' + ' '.join(str(v) for row in remote for v in row),
                'pause off', 'cast 1 2 3 5 5', 'undo player']
    args = [str(BIN), '--headless', '--log', str(log)]
    for command in commands:
        args += ['--command', command]
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    records = [json.loads(line) for line in log.read_text().splitlines()]
    cast = next(e for e in records if e['event'] == 'skill')
    assert cast['after']['pending'] == [[1, 1], [1, 2], [4, 5]], cast['after']['pending']
    assert cast['after']['mp'] == 105 and not cast['after']['action_done']
    resolution = next(e for e in records if e['event'] == 'resolution')
    assert 'damage=21' in resolution['detail'], resolution['detail']
    assert resolution['action'] == cast['action']
    rollback = next(e for e in records if e['event'] == 'rollback')
    assert rollback['after'] == cast['before']
    print('PASS: portal geometry, costs, gravity, immediate remote clears, passive source chains and complete rollback')
