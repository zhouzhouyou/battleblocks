"""Verify captured real Session events and artifact hashes for Issue 28."""
from pathlib import Path
import hashlib,json

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'analysis/issue28-awake-animation/final'
capture=json.loads((OUT/'capture.json').read_text(encoding='utf-8'))
checks={}
for name,scene in capture['scenes'].items():
    for kind in ['video','log']:
        assert hashlib.sha256((OUT/scene[kind]).read_bytes()).hexdigest()==scene[kind+'_sha256'],name
    rows=[json.loads(s) for s in (OUT/scene['log']).read_text(encoding='utf-8').splitlines()]
    if 'skill_seq' not in scene:continue
    trigger=next(r for r in rows if r['seq']==scene['skill_seq'])
    after=[r for r in rows if r['seq']>=trigger['seq']]
    own=[r for r in after if r['before']['enemy_turn']==trigger['before']['enemy_turn']]
    checks[name]=['video/log hashes and trigger verified']
    if name.endswith('-rollback') and 'pending-' not in name:
        rollback=next(r for r in after if r['event']=='rollback')
        assert rollback['after']==trigger['before'],name
        checks[name].append('full rollback snapshot identical')
    if name.endswith('-pending-rollback'):
        assert len([r for r in own if r['event']=='skill_hit'])==1,name
        checks[name].append('exactly one player damage hit after pending rollback')
    if name.endswith('-terminal'):
        assert any(r['event']=='battle_end' for r in after),name
        checks[name].append('real terminal event')
    if name.endswith('-form-transition'):
        assert trigger['event']=='awakening' and 'side=player' in trigger['detail'],name
        assert not trigger['before']['awakened'][0] and trigger['after']['awakened'][0],name
        checks[name].append('real normal to awake transition')
    if name.endswith('-enemy'):
        assert trigger['event']=='skill' and trigger['before']['enemy_turn'] and trigger['before']['awakened'][1],name
        checks[name].append('actual awakened AI cast')
    if name.endswith('-no-target') or name.endswith('-no-chain'):
        assert not any(r['event']=='resolution' for r in own),name
        checks[name].append('no own clear resolution')
    if name.endswith('-blood-full'):
        assert 'actual_heal=0 ' in trigger['detail'],name
        checks[name].append('capped zero healing')
    if name.endswith('-followup'):
        assert any(r['event']=='skill' and r['detail'].startswith('Nature ') for r in own),name
        assert any(r['event']=='resolution' and 'multiplier_percent=200' in r['detail'] for r in own),name
        checks[name].append('next action consumes Strike bonus')
    if name.endswith('-delayed'):
        clears=[r for r in own if r['event']=='resolution']
        assert clears and clears[0]['time']-trigger['time']>1.4,name
        checks[name].append('actual clear delayed beyond original cast duration')
report={'source_commit':capture['source_commit'],'binary_sha256':capture['binary_sha256'],
        'verified_scene_count':len(capture['scenes']),'checks':checks}
(OUT/'scenario-checks.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print('PASS',len(capture['scenes']),'real recordings;',len(checks),'combat scenes')
