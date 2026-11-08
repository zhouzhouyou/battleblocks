"""Exercise both forms of the final legacy batch through the actual GUI Session.

This checks casts, awakening, zero targets, blood payment and rollback; visual
quality still requires reviewing the real window and authored pose sheets.
"""
from pathlib import Path
import argparse
import json
import os
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
CLEAN = [[(x + 2*y) % 7 for x in range(8)] for y in range(8)]
ATTACK = [row[:] for row in CLEAN]
ATTACK[0][:3] = [2, 1, 2]
ATTACK[1][1] = 2


def scenario(binary, role, awake, directory):
    log = directory / f'{role}-{"awake" if awake else "normal"}.jsonl'
    process = subprocess.Popen(
        [str(binary), '--log', str(log), '--command', 'pause on'], cwd=ROOT,
        stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
        text=True, encoding='utf-8', errors='replace')

    def records():
        try:
            return [json.loads(line) for line in log.read_text(encoding='utf-8').splitlines() if line]
        except (FileNotFoundError, json.JSONDecodeError):
            return []

    def wait(predicate):
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise AssertionError('game exited: ' + process.communicate()[1])
            rows = records()
            if predicate(rows):
                return rows
            time.sleep(.025)
        raise AssertionError(f'timeout for {role}/{awake}: {records()[-2:]}')

    def event(command, kind):
        marker = records()[-1]['seq'] if records() else 0
        process.stdin.write(command + '\n')
        process.stdin.flush()
        rows = wait(lambda rows: any(r['seq'] > marker and r['event'] == kind for r in rows))
        return next(r for r in rows if r['seq'] > marker and r['event'] == kind)

    def edit(command):
        return event(command, 'debug_edit')

    def load(board):
        edit('board load ' + ' '.join(str(v) for row in board for v in row))

    try:
        wait(lambda rows: any(r['event'] == 'debug_edit' for r in rows))
        edit(f'hero player {role}')
        edit('hero enemy 2')
        if awake:
            edit('mp enemy 0')
            edit('hp player 1')
            load(ATTACK)
            marker = edit('turn enemy')['seq']
            edit('pause off')
            rows = wait(lambda rows: any(r['seq'] > marker and r['event'] == 'turn_start'
                                         and not r['after']['enemy_turn'] for r in rows))
            turn = next(r for r in rows if r['seq'] > marker and r['event'] == 'turn_start'
                        and not r['after']['enemy_turn'])
            assert turn['after']['awakened'][0]
            edit('pause on')

        for slot in (1, 2):
            load(CLEAN)
            edit('mp player 1000')
            edit('hp player 1000')
            edit('pause off')
            cast = event(f'cast {slot} 1 1', 'skill')
            assert cast['before']['awakened'][0] == awake
            assert cast['after']['mp'] < cast['before']['mp']
            # Let the visible recovery finish; the real model also settles its pending targets.
            time.sleep(1.6)
            assert event('undo player', 'rollback')['after'] == cast['before']
            edit('pause on')

        if role == 4:
            load([[(x + 2*y) % 6 + 1 for x in range(8)] for y in range(8)])
            edit('mp player 1000')
            edit('pause off')
            cast = event('cast 1 1 1', 'skill')
            assert cast['detail'].startswith('Imp ')
            assert not cast['after']['pending']
            time.sleep(1.6)
            assert event('undo player', 'rollback')['after'] == cast['before']
        if role == 6:
            load(CLEAN)
            edit('mp player 0')
            edit('hp player 1000')
            edit('pause off')
            cast = event('cast 2 1 1', 'skill')
            assert 'blood_cost=0 ' not in cast['detail']
            time.sleep(1.6)
            assert event('undo player', 'rollback')['after'] == cast['before']
        print(f'PASS role={role} awake={awake}: both casts, settlement and complete rollback', flush=True)
    finally:
        process.terminate()
        process.communicate(timeout=10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path(os.environ.get('BATTLEBLOCKS_BIN', ROOT/'target/debug/battleblocks')))
    parser.add_argument('--out', type=Path, help='Optional local output directory for JSONL evidence')
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='battleblocks-legacy-animation-') as temp:
        directory = args.out or Path(temp)
        directory.mkdir(parents=True, exist_ok=True)
        for role in (4, 5, 6):
            for awake in (False, True):
                scenario(args.binary, role, awake, directory)


if __name__ == '__main__':
    main()
