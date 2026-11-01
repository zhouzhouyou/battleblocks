"""Exercise source chains, target-side passive filtering and temporary statuses in the real loop."""
from pathlib import Path
import json, subprocess, tempfile, time
ROOT=Path(__file__).resolve().parents[1]
CLEAN=[[(x+2*y)%7 for x in range(8)] for y in range(8)]
def layout(kind,row=0):
    board=[r[:] for r in CLEAN];board[row][:4]=[kind,0,kind,kind];return board
with tempfile.TemporaryDirectory(prefix='battleblocks-events-') as temp:
    log=Path(temp)/'events.jsonl'
    process=subprocess.Popen([str(ROOT/'target/debug/battleblocks'),'--log',str(log),'--command','pause on'],cwd=ROOT,stdin=subprocess.PIPE,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,text=True)
    def records():
        try:return [json.loads(line) for line in log.read_text().splitlines() if line]
        except (FileNotFoundError,json.JSONDecodeError):return []
    def wait(predicate):
        until=time.monotonic()+20
        while time.monotonic()<until:
            if process.poll() is not None:raise AssertionError(process.communicate()[1])
            data=records()
            if predicate(data):return data
            time.sleep(.025)
        raise AssertionError('timeout: '+str(records()[-2:]))
    def action(command,kind):
        marker=records()[-1]['seq'] if records() else 0
        process.stdin.write(command+'\n');process.stdin.flush()
        data=wait(lambda r:any(e['seq']>marker and e['event']==kind for e in r))
        return next(e for e in data if e['seq']>marker and e['event']==kind)
    def edit(command):return action(command,'debug_edit')
    def load(board):return edit('board load '+' '.join(str(v) for row in board for v in row))
    def undo(before):assert action('undo player','rollback')['after']==before
    def swap(row=0):
        move=action(f'move 1 {row+1} 2 {row+1}','move')
        data=wait(lambda r:any(e['seq']>move['seq'] and e['event']=='resolution' and e['action']==move['action'] for e in r))
        wave=next(e for e in data if e['seq']>move['seq'] and e['event']=='resolution' and e['action']==move['action'])
        return move,wave
    try:
        wait(lambda r:any(e['event']=='debug_edit' for e in r))
        edit('hero player 10');edit('hero enemy 8');edit('hp player 400');edit('hp enemy 400');edit('mp player 180');load(layout(1));edit('pause off')
        move,wave=swap();assert (wave['after']['hp'],wave['after']['enemy_hp'])==(418,418)
        undo(move['before'])
        frenzy=action('cast 1','skill');assert any(s['id']=='Frenzy' for s in frenzy['after']['new_role_states'][0]['statuses'])
        move,wave=swap();assert (wave['after']['hp'],wave['after']['enemy_hp'])==(418,382)
        damage=[e['payload'] for e in records() if e['event']=='reward_event' and e['action']==move['action'] and e['payload']['resource']=='Damage']
        assert all(event['payload']['source']['kind']!='unspecified' for event in records() if event['event']=='reward_event')
        assert damage and all(e['source']=={'kind':'status','id':'Frenzy'} for e in damage)
        assert all([c['source']['kind'] for c in e['source_chain']]==['tile_clear','passive','status'] for e in damage)
        undo(move['before']);undo(frenzy['before'])
        load(layout(2));move,wave=swap();assert wave['after']['enemy_hp']==400
        assert any(e['event']=='ability_trigger' and e['action']==move['action'] and 'Dwarf.evade' in e['detail'] for e in records())
        undo(move['before'])
        edit('hero enemy 2');edit('hp player 1');edit('pause on')
        attack=[row[:] for row in CLEAN];attack[0][:3]=[2,1,2];attack[1][1]=2;load(attack);marker=edit('turn enemy');edit('pause off')
        data=wait(lambda r:any(e['seq']>marker['seq'] and e['event']=='turn_start' and not e['after']['enemy_turn'] for e in r))
        turn=next(e for e in data if e['seq']>marker['seq'] and e['event']=='turn_start' and not e['after']['enemy_turn']);assert turn['after']['awakened'][0]
        edit('hero enemy 8');edit('hp player 200');edit('hp enemy 400');edit('mp player 180');load(layout(1))
        move,wave=swap();assert (wave['after']['hp'],wave['after']['enemy_hp'])==(200,364)
        undo(move['before'])
        rational=action('cast 1','skill');assert any(s['id']=='Rational' for s in rational['after']['new_role_states'][0]['statuses'])
        move,wave=swap();assert (wave['after']['hp'],wave['after']['enemy_hp'])==(218,418)
        undo(move['before']);load(layout(2,5));move,wave=swap(5)
        assert (wave['after']['hp'],wave['after']['enemy_hp'])==(200,364),'reason must leave normal attacks intact'
        undo(move['before']);undo(rational['before'])
        print('PASS: normal/frenzy/awakened/reason event chains, nearest-source dwarf filtering, feedback and full status rollback')
    finally:
        process.terminate()
        try:process.communicate(timeout=5)
        except subprocess.TimeoutExpired:process.kill();process.communicate()
