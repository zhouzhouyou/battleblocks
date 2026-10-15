"""Exercise native stdin commands, action logs and rollback through the real game loop."""
from pathlib import Path
import json, subprocess, tempfile, time
ROOT=Path(__file__).resolve().parents[1]
board=[[ (x+2*y)%7 for x in range(8)] for y in range(8)]
board[0][:3]=[2,1,2];board[1][1]=2
with tempfile.TemporaryDirectory(prefix='battleblocks-debug-') as temp:
    log=Path(temp)/'events.jsonl'
    args=[str(ROOT/'target/debug/battleblocks'),'--log',str(log)]
    commands=['pause on','board load '+' '.join(str(v) for row in board for v in row),'hp player 650','mp player 300','pause off','move 2 1 2 2']
    for command in commands:args+=['--command',command]
    process=subprocess.Popen(args,cwd=ROOT,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    def records():
        return [json.loads(line) for line in log.read_text().splitlines() if line] if log.exists() else []
    def wait(predicate):
        until=time.monotonic()+15
        while time.monotonic()<until:
            if process.poll() is not None:raise AssertionError('Game exited: '+process.communicate()[1])
            try:
                data=records()
            except json.JSONDecodeError:
                time.sleep(.02);continue
            if predicate(data):return data
            time.sleep(.03)
        raise AssertionError('Timeout waiting for event; log tail: '+str(records()[-3:]))
    def send(line):process.stdin.write(line+'\n');process.stdin.flush()
    try:
        data=wait(lambda r:any(e['event']=='turn_start' and not e['after']['enemy_turn'] for e in r))
        player_move=next(e for e in data if e['event']=='move' and e['actor']=='player')
        assert any(e['event']=='move' and e['actor']=='enemy' for e in data)
        assert any(e['event']=='resolution' and e['actor']=='player' for e in data)
        send('undo player')
        data=wait(lambda r:any(e['event']=='rollback' for e in r));rollback=next(e for e in data if e['event']=='rollback')
        assert rollback['after']==player_move['before'], 'Rollback must restore complete pre-move state'
        send('cast 1 2 3')
        data=wait(lambda r:any(e['event']=='skill' for e in r));cast=next(e for e in data if e['event']=='skill')
        assert cast['before']['mp']-cast['after']['mp']==100
        attack=[4,0,12,0,7,0,7]
        expected=60+sum(attack[row[x]] for row in cast['before']['board'] for x in (1,2))
        assert cast['after']['enemy_hp']==cast['before']['enemy_hp'], 'Charge damage waits for visual impact'
        data=wait(lambda r:any(e['event']=='skill_hit' and e['action']==cast['action'] for e in r))
        hit=next(e for e in data if e['event']=='skill_hit' and e['action']==cast['action'])
        actual=hit['before']['enemy_hp']-hit['after']['enemy_hp']
        assert actual==min(expected,hit['before']['enemy_hp']), (expected,actual)
        assert f'actual_damage={actual}' in hit['detail']
        assert len(cast['after']['pending'])==2*len(board)
        send('undo player')
        data=wait(lambda r:sum(e['event']=='rollback' for e in r)>=2)
        assert [e for e in data if e['event']=='rollback'][-1]['after']==cast['before']
        send('tile 8 8 6')
        data=wait(lambda r:any(e['event']=='debug_edit' and e['detail']=='tile 8 8 6' for e in r))
        edit=next(e for e in data if e['event']=='debug_edit' and e['detail']=='tile 8 8 6');assert edit['after']['board'][7][7]==6
        send('undo')
        data=wait(lambda r:sum(e['event']=='rollback' for e in r)>=3)
        assert [e for e in data if e['event']=='rollback'][-1]['after']==edit['before']
        send('hp player 500')
        wait(lambda r:any(e['event']=='debug_edit' and e['detail']=='hp player 500' for e in r))
        send('mp player 300')
        wait(lambda r:sum(e['event']=='debug_edit' and e['detail']=='mp player 300' for e in r)>=2)
        send('cast 2')
        data=wait(lambda r:any(e['event']=='skill' and 'Guard' in e['detail'] for e in r))
        guard=next(e for e in data if e['event']=='skill' and 'Guard' in e['detail'])
        assert guard['after']['hp']==640 and guard['after']['mp']==150
        assert guard['after']['action_done'] and not guard['after']['armed']
        send('undo player')
        data=wait(lambda r:sum(e['event']=='rollback' for e in r)>=4)
        assert [e for e in data if e['event']=='rollback'][-1]['after']==guard['before']
        assert [e['seq'] for e in data]==list(range(1,len(data)+1))
        print(f'PASS: {len(data)} JSONL events; player/enemy moves, cascades, cast, board edit, full-state rollbacks')
    finally:
        process.terminate()
        try:process.communicate(timeout=5)
        except subprocess.TimeoutExpired:process.kill();process.communicate()
