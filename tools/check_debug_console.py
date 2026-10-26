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
    # Only the JSONL log is checked. An undrained stdout pipe fills up (about 4 KiB on Windows) and blocks the game.
    process=subprocess.Popen(args,cwd=ROOT,stdin=subprocess.PIPE,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,text=True,encoding='utf-8',errors='replace')
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
        send('ai 2')  # Difficulty is locked once the battle starts.
        send('tile 8 8 6')
        data=wait(lambda r:any(e['event']=='debug_edit' and e['detail']=='tile 8 8 6' for e in r))
        edit=next(e for e in data if e['event']=='debug_edit' and e['detail']=='tile 8 8 6');assert edit['after']['board'][7][7]==6
        assert edit['before']['difficulty']==edit['after']['difficulty']==1
        assert not any(e['event']=='debug_edit' and e['detail']=='ai 2' for e in data)
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
        assert guard['after']['hp']==640 and guard['after']['mp']==50
        assert guard['after']['action_done'] and not guard['after']['armed']
        expected_cells=[[x,y] for y,row in enumerate(guard['before']['board']) for x,tile in enumerate(row) if tile==1]
        assert guard['after']['pending']==expected_cells and expected_cells
        data=wait(lambda r:any(e['event']=='resolution' and e['action']==guard['action'] for e in r))
        consumed=next(e for e in data if e['event']=='resolution' and e['action']==guard['action'])
        assert consumed['before']['hp']==consumed['after']['hp']==640
        assert consumed['after']['mp']==50
        assert 'reward=false' in consumed['detail']

        send('undo player')
        data=wait(lambda r:sum(e['event']=='rollback' for e in r)>=4)
        assert [e for e in data if e['event']=='rollback'][-1]['after']==guard['before']
        assert [e['seq'] for e in data]==list(range(1,len(data)+1))
        # Check the new role rules and HP payment through the real game loop.
        def edit_command(command):
            count=sum(e['event']=='debug_edit' and e['detail']==command for e in records())
            send(command)
            return wait(lambda r:sum(e['event']=='debug_edit' and e['detail']==command for e in r)>count)
        def action(command,event):
            count=sum(e['event']==event for e in records())
            send(command)
            data=wait(lambda r:sum(e['event']==event for e in r)>count)
            return [e for e in data if e['event']==event][-1]
        def rollback_to(before):
            undone=action('undo player','rollback')
            assert undone['after']==before
        edit_command('hero player 6')
        edit_command('hp player 200')
        edit_command('mp player 25')
        clean=[[(x+2*y)%7 for x in range(8)] for y in range(8)]
        edit_command('board load '+' '.join(str(v) for row in clean for v in row))
        paid=action('cast 2 1 1','skill')
        heal_values=[2,6,0,0,3,3,0]
        healing=2*sum(heal_values[clean[y][x]] for y in range(2) for x in range(2))
        assert paid['after']['hp']==310
        assert paid['after']['awakened']==[True,False]
        assert any(e['event']=='awakening' and 'cause=blood_payment' in e['detail'] and e['action']==paid['action'] for e in records())
        assert paid['after']['mp']==0
        assert 'blood_cost=50' in paid['detail'] and 'cost=25' in paid['detail']
        # Next player turn uses the awakened exchange rate, including odd MP deficits.
        wait(lambda r:any(e['event']=='turn_start' and not e['after']['enemy_turn'] and e['seq']>paid['seq'] for e in r))
        edit_command('hp player 100')
        edit_command('mp player 25')
        edit_command('board load '+' '.join(str(v) for row in clean for v in row))
        efficient=action('cast 2 1 1','skill')
        assert efficient['before']['awakened'][0]
        assert 'blood_cost=13' in efficient['detail']
        assert efficient['after']['hp']==100-13+healing
        assert efficient['after']['mp']==0
        rollback_to(efficient['before'])
        rollback_to(paid['before'])
        for role in (0,1):
            edit_command(f'hero player {role}')
            layout=[row[:] for row in clean]
            if role==0:
                layout[0][:4]=[6,2,6,6]
                command='move 1 1 2 1'
            else:
                layout[0][:3]=[6,6,2];layout[1][3]=6
                command='move 3 1 4 2'
            edit_command('board load '+' '.join(str(v) for row in layout for v in row))
            moved=action(command,'move')
            if role==0:
                for y in range(8):
                    assert moved['after']['board'][y][0]==layout[y][1]
                    assert moved['after']['board'][y][1]==layout[y][0]
            else:
                assert moved['after']['board'][0][:3]==[6,6,6]
            rollback_to(moved['before'])
        # A no-swap board must survive while Charge is affordable.
        edit_command('hero player 0')
        edit_command('mp player 100')
        edit_command('board load '+' '.join(str(v) for row in clean for v in row))
        refreshes=sum(e['event']=='board_refresh' for e in records())
        rejected=action('move 1 1 2 1','move_rejected')
        assert rejected['after']['board']==clean
        assert sum(e['event']=='board_refresh' for e in records())==refreshes
        escaped=action('cast 1 1 1','skill')
        assert escaped['before']['board']==clean
        rollback_to(escaped['before'])
        # With neither an exchange nor a payable skill, refresh for this actor.
        refreshes=sum(e['event']=='board_refresh' for e in records())
        edit_command('mp player 0')
        data=wait(lambda r:sum(e['event']=='board_refresh' for e in r)>refreshes)
        refreshed=[e for e in data if e['event']=='board_refresh'][-1]
        assert refreshed['before']['board']==clean
        assert refreshed['after']['board']!=clean
        assert 'No role-legal swap or usable skill' in refreshed['detail']
        # HP payment also keeps a no-swap board available for casting.
        edit_command('hero player 6')
        edit_command('hp player 101')
        edit_command('mp player 0')
        edit_command('board load '+' '.join(str(v) for row in clean for v in row))
        refreshes=sum(e['event']=='board_refresh' for e in records())
        rejected=action('move 1 1 2 1','move_rejected')
        assert rejected['after']['board']==clean
        assert sum(e['event']=='board_refresh' for e in records())==refreshes
        escaped=action('cast 2 1 1','skill')
        assert 'blood_cost=100' in escaped['detail']
        assert escaped['before']['board']==clean
        rollback_to(escaped['before'])
        # A lethal Charge is interrupted by awakening; undo restores both full bars.
        edit_command('hero player 0')
        edit_command('hero enemy 0')
        edit_command('hp enemy 20')
        edit_command('mp player 100')
        edit_command('board load '+' '.join(str(v) for row in clean for v in row))
        lethal=action('cast 1 1 1','skill')
        data=wait(lambda r:any(e['event']=='skill_hit' and e['action']==lethal['action'] for e in r))
        hit=next(e for e in data if e['event']=='skill_hit' and e['action']==lethal['action'])
        assert hit['after']['enemy_hp']==385
        assert hit['after']['awakened']==[False,True]
        assert any(e['event']=='awakening' and e['action']==lethal['action'] for e in data)
        rollback_to(lethal['before'])
        # Wizard reserve is clamped before awakening and unlocks on the actual hit.
        edit_command('hero enemy 2')
        data=edit_command('mp enemy 800')
        assert data[-1]['after']['enemy_mp']==250
        edit_command('hp enemy 20')
        edit_command('mp player 100')
        unlock=action('cast 1 1 1','skill')
        data=wait(lambda r:any(e['event']=='skill_hit' and e['action']==unlock['action'] for e in r))
        hit=next(e for e in data if e['event']=='skill_hit' and e['action']==unlock['action'])
        assert hit['after']['awakened']==[False,True]
        assert hit['after']['enemy_mp']==250
        data=edit_command('mp enemy 800')
        assert data[-1]['after']['enemy_mp']==400
        rollback_to(unlock['before'])
        # At the locked full reserve, the wizard's clear heals normally.
        edit_command('hero player 2')
        edit_command('hp player 300')
        edit_command('mp player 250')
        healing_board=[row[:] for row in clean];healing_board[0][:3]=[1,1,1]
        edit_command('board load '+' '.join(str(v) for row in healing_board for v in row))
        wave=action('resolve','resolution')
        assert wave['after']['hp']-wave['before']['hp']==18
        assert wave['after']['mp']==250
        data=records()
        assert [e['seq'] for e in data]==list(range(1,len(data)+1))
        print(f'PASS: {len(data)} JSONL events; awakening (lethal/HP payment), role swaps, cascades, skills and complete phase rollbacks')
    finally:
        process.terminate()
        try:process.communicate(timeout=5)
        except subprocess.TimeoutExpired:process.kill();process.communicate()
