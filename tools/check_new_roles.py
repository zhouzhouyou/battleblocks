"""Exercise the new characters through the actual native game loop and JSON log."""
from pathlib import Path
import json, re, subprocess, tempfile, time
ROOT=Path(__file__).resolve().parents[1]
CLEAN=[[(x+2*y)%7 for x in range(8)] for y in range(8)]
ATTACK=[row[:] for row in CLEAN];ATTACK[0][:3]=[2,1,2];ATTACK[1][1]=2
with tempfile.TemporaryDirectory(prefix='battleblocks-new-roles-') as directory:
 log=Path(directory)/'events.jsonl'
 process=subprocess.Popen([str(ROOT/'target/debug/battleblocks'),'--log',str(log),'--command','pause on'],cwd=ROOT,stdin=subprocess.PIPE,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,text=True)
 def records():
  try:return [json.loads(line) for line in log.read_text().splitlines() if line]
  except (FileNotFoundError,json.JSONDecodeError):return []
 def wait(predicate):
  until=time.monotonic()+30
  while time.monotonic()<until:
   if process.poll() is not None:raise AssertionError('game exited: '+process.communicate()[1])
   data=records()
   if predicate(data):return data
   time.sleep(.025)
  raise AssertionError('timeout: '+str(records()[-2:]))
 def send(text):process.stdin.write(text+'\n');process.stdin.flush()
 def event(command,kind,match=lambda e:True):
  previous=records()[-1]['seq'] if records() else 0
  send(command)
  data=wait(lambda data:any(e['seq']>previous and e['event']==kind and match(e) for e in data))
  return next(e for e in data if e['seq']>previous and e['event']==kind and match(e))
 def edit(command):return event(command,'debug_edit',lambda e:e['detail']==command)
 def load(board):return edit('board load '+' '.join(str(v) for row in board for v in row))
 def cast(slot,x=1,y=1):return event(f'cast {slot} {x} {y}','skill')
 def player_turn(after):
  data=wait(lambda data:any(e['seq']>after and e['event']=='turn_start' and not e['after']['enemy_turn'] for e in data))
  return next(e for e in data if e['seq']>after and e['event']=='turn_start' and not e['after']['enemy_turn'])
 def rollback(expected):
  undone=event('undo player','rollback');assert undone['after']==expected,'full state including RNG/pockets/cooldowns must restore'
 try:
  wait(lambda r:any(e['event']=='debug_edit' and e['detail']=='pause on' for e in r))
  edit('hero player 8');edit('hero enemy 2');edit('mp player 100');load(CLEAN);edit('pause off')
  jump=cast(1)
  assert 'Jump' in jump['detail'];assert jump['after']['mp']==jump['before']['mp']==100
  assert not jump['after']['action_done'] and any(s['id']=='ExtraReach' for s in jump['after']['new_role_states'][0]['statuses'])
  assert jump['after']['cooldowns'][0][18]==2
  forge=cast(2,1,4)
  assert forge['after']['mp']==50 and forge['after']['action_done']
  for y in (3,4):
   for x in (0,1):assert forge['after']['board'][y][x]==2
  turn=player_turn(forge['seq']);assert turn['after']['cooldowns'][0][18]==1
  assert not any(s['id']=='ExtraReach' for s in turn['after']['new_role_states'][0]['statuses'])
  print('PASS dwarf jump, reach expansion, forge, owner-turn cooldown',flush=True)

  # A swap clear checks each eliminated tile's current row for dwarf immunity.
  edit('hero player 8');edit('hero enemy 2');edit('pause on');load(ATTACK);marker=edit('turn enemy');edit('pause off')
  data=wait(lambda r:any(e['event']=='move' and e['actor']=='enemy' and e['seq']>marker['seq'] for e in r))
  move=next(e for e in data if e['event']=='move' and e['actor']=='enemy' and e['seq']>marker['seq'])
  assert move['after']['new_role_states'][1]['swap_clear'], (move['seq'],move['detail'],move['before']['enemy'],move['before']['board'])
  turn=player_turn(move['seq'])
  attacks=[4,0,12,0,7,0,7]
  for e in records():
   if move['seq']<e['seq']<turn['seq'] and e['event']=='resolution' and e['actor']=='enemy':
    coords=[(int(x)-1,int(y)-1) for x,y in re.findall(r'\((\d+), (\d+)\)',e['detail'].split(' reward=')[0])]
    scale=int(re.search(r'multiplier_percent=(\d+)',e['detail']).group(1))
    expected=sum(attacks[e['before']['board'][y][x]] for x,y in coords if y>=4)*scale//100
    actual=int(re.search(r' actual_damage=(\d+)',e['detail']).group(1))
    assert actual==expected,(e['detail'],expected,actual)
  print('PASS dwarf per-tile row damage immunity through cascades',flush=True)

  edit('hero player 9');edit('hero enemy 2');edit('mp player 100');load(CLEAN)
  buried=cast(1,3,3)
  assert buried['after']['mp']==65 and buried['after']['new_role_states'][0]['pockets'][0]==CLEAN[2][2]
  assert buried['after']['pending']==[[2,2]] and not buried['after']['reward']
  turn=player_turn(buried['seq'])
  pockets=turn['after']['new_role_states'][0]['pockets'];assert all(v is not None for v in pockets)
  enemy_direct=[]
  for e in records():
   if buried['seq']<e['seq']<turn['seq'] and e['event']=='resolution' and e['actor']=='enemy' and e['before']['chain']==1 and e['before']['reward']:
    enemy_direct+=e['after']['new_role_states'][1]['direct_tiles']
  assert pockets[1] in enemy_direct
  load(CLEAN)
  risen=cast(2,4,4)
  assert not risen['after']['action_done'] and risen['after']['cooldowns'][0][21]==0
  before=risen['before'];after=risen['after'];tile=before['new_role_states'][0]['pockets'][0]
  assert after['board'][3][3]==tile
  for y in range(4,8):assert after['board'][y][3]==before['board'][y-1][3]
  assert after['mp']==before['mp'] and after['new_role_states'][0]['pockets'][0] is None
  rollback(before)
  print('PASS priest entomb, mind vision, insertion, capacity, pure destruction, full rollback',flush=True)

  edit('hero player 7');edit('mp player 200');load(CLEAN)
  miracle=cast(2)
  assert miracle['after']['new_role_states'][0]['force_armed'] and not miracle['after']['action_done']
  forced=event('move 1 1 2 1','move')
  assert not forced['after']['action_done'] and any(s['id']=='ClearPenalty' for s in forced['after']['new_role_states'][0]['statuses'])
  for y in range(8):assert forced['after']['board'][y][0]==forced['before']['board'][y][1]
  rollback(forced['before']);rollback(miracle['before'])
  rush=cast(1,1,1)
  assert rush['after']['reward'] and len(rush['after']['pending'])==16
  assert rush['after']['cooldowns'][0][15]==2
  data=wait(lambda r:any(e['event']=='resolution' and e['action']==rush['action'] for e in r))
  resolved=next(e for e in data if e['event']=='resolution' and e['action']==rush['action'])
  attacks=[4,0,12,0,7,0,7]
  damage=sum(attacks[row[x]]+(3 if attacks[row[x]] else 0) for row in rush['before']['board'] for x in (0,1))
  assert f'damage={damage}' in resolved['detail']
  rollback(rush['before'])
  print('PASS knight force movement, normal elimination, strength and cooldown rollback',flush=True)

  # Awaken the new knight using a real enemy skill, then verify random Hammer.
  edit('hero enemy 2');edit('mp enemy 0');edit('hp player 1');edit('pause on');load(ATTACK);marker=edit('turn enemy');edit('pause off')
  turn=player_turn(marker['seq'])
  assert turn['after']['awakened'][0]
  edit('mp player 150');load(CLEAN)
  hammer=cast(3)
  assert 'Hammer' in hammer['detail'] and hammer['after']['action_done']
  assert hammer['after']['cooldowns'][0][17]==1 and len(hammer['after']['pending'])==4
  assert hammer['before']['rng_state']!=hammer['after']['rng_state']
  rollback(hammer['before'])
  again=cast(3);assert again['after']['pending']==hammer['after']['pending']
  rollback(again['before'])
  print('PASS knight awakening third slot, random hammer and RNG restoration',flush=True)

  # Copy a non-turn-ending resurrection from another priest. Its native cooldown
  # is zero, but the awakened copied slot must stay at one.
  copied=False
  for seed in range(1,9):
   edit('hero player 9');edit('hero enemy 9');edit('hp player 1');load(ATTACK);edit(f'seed {seed}')
   turn_edit=edit('turn enemy');turn=player_turn(turn_edit['seq'])
   assert turn['after']['awakened'][0]
   state=turn['after']['new_role_states'][0]
   if state['copied_skill']!=21:continue
   assert state['pockets'][0] is not None
   borrowed=cast(3,4,4)
   assert 'Copy' in borrowed['detail'] and 'Resurrect' in borrowed['detail']
   assert not borrowed['after']['action_done']
   assert borrowed['after']['cooldowns'][0][22]==1 and borrowed['after']['cooldowns'][0][21]==0
   assert borrowed['after']['mp']==borrowed['before']['mp']
   rollback(borrowed['before']);copied=True;break
  assert copied,'deterministic seeds must exercise copied resurrection'
  print('PASS priest awakening, copied native cost/end-turn rule, fixed copied-slot cooldown',flush=True)
  print(f'PASS: {len(records())} real new-character events',flush=True)
 finally:
  process.terminate()
  try:process.wait(timeout=5)
  except subprocess.TimeoutExpired:process.kill();process.wait()
