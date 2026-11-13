from pathlib import Path
import re,json
base=Path(__file__).resolve().parent;out=base/'native-ready-r1';text=(out/'native.stderr.log').read_text();lines=text.splitlines()
entry_positions=[i for i,l in enumerate(lines) if l.startswith('ROOT_STOP_ENTRY stage=load-ready ')]
assert len(entry_positions)==2
fields=lambda line:dict(re.findall(r'(\w+)=([^ ]+)',line))
entry=[]
for pos in entry_positions:
 pair=[fields(lines[pos]),fields(lines[pos+1])]
 assert [p['stage'] for p in pair]==['load-ready','pre-stop'] and all(p['passed']=='true' and int(p['remaining_us'])>0 for p in pair)
 entry.append(pair)
second=lines[entry_positions[1]:]
steps=[fields(l) for l in second if l.startswith('ROOT_STOP_STEP ')]
assert steps
for stage in ['artifacts.admission-before','artifacts.guard-image','artifacts.plan-open','artifacts.authority-initial']:
 row=next(r for r in steps if r['stage']==stage)
 assert row['code']=='ok' and row['expired_after']=='false' and int(row['remaining_after_us'])>0,stage
live=[r for r in steps if r['stage']=='artifact-authority.live'];assert live
crossing=live[-1];assert crossing['code']=='ok' and crossing['expired_after']=='true' and int(crossing['remaining_before_us'])>0 and int(crossing['remaining_after_us'])==0
apply=next(r for r in steps if r['stage']=='artifacts.apply-retaining')
assert apply['code']=='client_session_unconfirmed' and apply['expired_after']=='true' and int(apply['remaining_after_us'])==0
assert any(l=='ROOT_STOP_AUTHORITY_REFUSAL code=lifecycle_timeout mapped_to=permission-denied' for l in second)
assert not any('ROOT_STOP_WRITER_REFUSAL ' in l for l in second)
summary=json.loads((out/'native/summary.json').read_text());cycles=json.loads((out/'native/root-cycles.json').read_text());runner=json.loads((out/'runner-summary.json').read_text())
assert runner['exitCode']==101 and runner['diagnosticPassed'] is False and cycles['secondClosed']['revision']==16
assert summary['stop']['result']=='stopped' and cycles['secondStop']['result'] is None and cycles['secondStop']['failure']['code']=='client_original_restore_pending'
print(json.dumps({'accepted':False,'actualNativeRounds':1,'sourceCommit':json.loads((base/'compiled-source.json').read_text())['baseCommit'],
 'entryLoadAndPreStop':entry,'artifactApply':apply,'lastLiveCheckCrossingDeadline':crossing,
 'underlyingRefusal':'lifecycle_timeout','oldMappedIoRefusal':'permission-denied','outerMappedSessionRefusal':'client_session_unconfirmed',
 'causeForThisRound':'Original bound deadline expired during artifact authority live-binding observation; the following original pair/authority check refused. No writer-refusal trace occurred. This does not classify untraced previous rounds.',
 'firstCycleMs':{k:summary[k] for k in ['startElapsedMs','statusElapsedMs','stopElapsedMs']},
 'secondCycleMs':{k:cycles[k] for k in ['secondStartElapsedMs','secondLiveStatusElapsedMs','secondStopElapsedMs']},
 'retainedState':'restore_prepared','revision':16,'noCompletionInvented':cycles['secondClosed']['completionSha256'] is None,
 'scope':'Exact diagnostic cause from this failed release-library round; no timeout reset, successful normalStop or whole-version acceptance'},indent=2))
