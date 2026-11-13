from pathlib import Path
import re,json,gzip
base=Path(__file__).resolve().parent;out=base/'native-partition-r1';lines=gzip.decompress((out/'native.stderr.txt.gz').read_bytes()).decode().splitlines()
pos=[i for i,s in enumerate(lines) if s.startswith('ROOT_STOP_ENTRY stage=load-ready')];assert len(pos)==2
parse=lambda s:dict(re.findall(r'(\w+)=([^ ]+)',s))
entries=[parse(s) for s in lines if s.startswith('ROOT_STOP_ENTRY')];steps=[parse(s) for s in lines[pos[1]:] if s.startswith('ROOT_STOP_STEP')]
for stage in ['artifacts.admission-before','artifacts.guard-image','artifacts.plan-open','artifacts.authority-initial','artifacts.apply-retaining','artifacts.admission-after','artifacts.final-time']:
 row=next(s for s in steps if s['stage']==stage);assert row['code']=='ok' and row['expired_after']=='false' and int(row['remaining_after_us'])>0
apply=next(s for s in steps if s['stage']=='artifacts.apply-retaining');final=next(s for s in steps if s['stage']=='artifacts.final-time')
expired=[s for s in steps if s['expired_after']=='true'];assert len(expired)==1 and expired[0]['stage']=='admission.entry-time' and expired[0]['code']=='timeout'
summary=json.loads((out/'native/summary.json').read_text());cycles=summary['rootCycles'];head=cycles['secondClosed']
assert head['state']=='root_retirement_prepared' and head['revision']==17 and head['completionSha256'] is None
log=gzip.decompress((out/'manager.txt.gz').read_bytes()).decode();assert 'failed phase=root-retire code=timeout' in log and 'duration_ms=10996' in log
assert not any('ROOT_STOP_AUTHORITY_REFUSAL' in s or 'ROOT_STOP_WRITER_REFUSAL' in s for s in lines[pos[1]:])
print(json.dumps({'accepted':False,'sourceCommit':json.loads((base/'compiled-source.json').read_text())['baseCommit'],'actualNativeRounds':1,'firstCycleMs':{k:summary[k] for k in ['startElapsedMs','statusElapsedMs','stopElapsedMs']},'secondCycleMs':{k:cycles[k] for k in ['secondStartElapsedMs','secondLiveStatusElapsedMs','secondStopElapsedMs']},'entryLoadAndPreStop':entries,'artifactApply':apply,'artifactEndBeforePlan':final,'rootRetireEntryRefusal':expired[0],'registeredState':'root_retirement_prepared','revision':17,'productInternalStopMs':10996,'originalRootMovementNotStarted':True,'scope':'This new round completed artifact restore within original budget but root-plan/index preparation consumed remaining budget; failed result remains. No claim prior untraced phases had this cause.'},indent=2))
