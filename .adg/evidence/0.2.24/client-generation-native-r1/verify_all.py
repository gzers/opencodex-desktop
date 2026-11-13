from pathlib import Path
import json,gzip,hashlib,subprocess,sys
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
source=json.loads((base/'compiled-source.json').read_text());assert source['baseCommit']==m['sourceCommit'] and source['cleanSource'] and len(source['files'])==28 and source['binarySha256']=='9639b54d824b4b64637076d1d8b0d02cda64bacdbd8846e5474b96bac12a5b4d'
assert json.loads((base/'release-build-final.json').read_text())['exitCode']==0
assert json.loads(subprocess.check_output([sys.executable,str(base/'verify_native_pass.py')],text=True))==json.loads((base/'verified-native.json').read_text())
out=base/'native-generation-r1';runner=json.loads((out/'runner-summary.json').read_text());assert runner['diagnosticPassed'] and runner['exitCode']==0 and runner['exactOneTestExecuted'] and runner['twoCycleCompletionMarker'] and runner['binarySha256']==source['binarySha256']
summary=json.loads((out/'native/summary.json').read_text());cycles=summary['rootCycles'];d=json.loads((base/'diagnosis.json').read_text());assert d['directedNativePassed'] and d['actualNativeRounds']==1 and d['stopActionAndStatusStartEligibilityAgrees']
assert [summary[k] for k in ['startElapsedMs','statusElapsedMs','stopElapsedMs']]==[7160,231,6016]
assert [cycles[k] for k in ['secondStartElapsedMs','secondLiveStatusElapsedMs','secondStopElapsedMs']]==[9749,785,9572]
stderr=gzip.decompress((out/'native.stderr.txt.gz').read_bytes()).decode();prefix='COORDINATION_TAIL_COST ';tails=[json.loads(v.split(prefix,1)[1]) for v in stderr.splitlines() if v.startswith(prefix)];assert len(tails)==6 and all(not v['expiredAfter'] for v in tails)
assert tails[2]==d['firstStopTail'] and tails[5]==d['secondStopTail'] and tails[5]['remainingAfterUs']==442132
prefix='ROOT_REGISTRATION_COST ';costs=[json.loads(v.split(prefix,1)[1]) for v in stderr.splitlines() if v.startswith(prefix)];partition=json.loads((base/'verified-native-partition.json').read_text());assert len(costs)==len(partition['samples'])==2
keys=['transaction_storage_begin','transaction_source','transaction_admission','transaction_sql','transaction_candidate','transaction_commit','transaction_postcommit','registration_facts']
for i,cost in enumerate(costs):
 assert cost['passed'];metrics=cost['metrics'];sample=partition['samples'][i];assert sample['totalNanos']==metrics['registration_total']['inclusive_nanos'] and sample['exclusiveStageNanos']=={key:metrics[key]['inclusive_nanos'] for key in keys}
 assert 0.999<sum(sample['exclusiveStageNanos'].values())/sample['totalNanos']<=1
 assert sample['frameChecks']==metrics['frame']['calls'] and sample['archiveChecks']==metrics['frame_archives']['calls']
assert partition['samples'][1]['frameChecks']==48 and partition['samples'][1]['historicalBegin']==3 and partition['samples'][1]['historicalFinish']==3 and partition['samples'][1]['historicalPasses']==6
c=json.loads((base/'cleanup-final.json').read_text());assert c['actualNativeRounds']==1 and c['directedDiagnosticPassed'] and c['archiveComparedBeforeRemoval'] and c['temporaryNativeRootRemoved'] and c['rawProfileFiles']==79 and c['exactNewCoordinatorsRemoved']==3 and c['globalLUnchangedAndReleased'] and c['normalStopBudgetUnchanged'] and c['previousFailuresUnchanged']
print(json.dumps({'accepted':False,'sourceCommit':m['sourceCommit'],'actualNativeRounds':1,'directedTwoCyclesPassed':True,'firstCycleStartStatusStopMs':[7160,231,6016],'secondCycleStartStatusStopMs':[9749,785,9572],'firstAndSecondStoppedStatusMs':[961,1840],'secondStopCoordinationRemainingUs':442132,'currentState':'RootAbsent18','completeClosedGenerations':2,'actionAndStatusStartEligibilityAgrees':True,'secondRegistrationMs':round(partition['samples'][1]['totalNanos']/1e6,3),'priorFailuresUnchanged':True,'rawProfileFilesVerified':79,'allOwnedTemporaryRootsRemoved':True,'fixedAppAndWholeVersionAccepted':False,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in m['files'])},indent=2))
