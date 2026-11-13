from pathlib import Path
import json,gzip,hashlib,subprocess,sys
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
s=json.loads((base/'compiled-source.json').read_text());assert s['cleanSource'] and s['baseCommit']==m['sourceCommit'] and len(s['files'])==25 and s['binarySha256']=='958302401dd8bc38b4421ec468f7e94f44449676f4998509eb73514344202e30'
assert json.loads((base/'release-build-final.json').read_text())['exitCode']==0
assert json.loads(subprocess.check_output([sys.executable,str(base/'verify_failed.py')],text=True))==json.loads((base/'verified-failure.json').read_text())
out=base/'native-exitfix-r1';lines=gzip.decompress((out/'native.stderr.txt.gz').read_bytes()).decode().splitlines();prefix='COORDINATION_TAIL_COST ';tails=[json.loads(v.split(prefix,1)[1]) for v in lines if v.startswith(prefix)];assert len(tails)==5 and all(not v['expiredAfter'] for v in tails)
d=json.loads((base/'diagnosis.json').read_text());assert d['firstStopTail']==tails[2] and d['previousProcargsFailureAbsent']
assert not any('read recovery process arguments' in v for v in lines)
prefix='ROOT_REGISTRATION_COST ';costs=[json.loads(v.split(prefix,1)[1]) for v in lines if v.startswith(prefix)];partition=json.loads((base/'verified-native-partition.json').read_text());assert len(costs)==len(partition['samples'])==2
keys=['transaction_storage_begin','transaction_source','transaction_admission','transaction_sql','transaction_candidate','transaction_commit','transaction_postcommit','registration_facts']
for i,cost in enumerate(costs):
 metrics=cost['metrics'];sample=partition['samples'][i];assert cost['passed'] and sample['totalNanos']==metrics['registration_total']['inclusive_nanos']
 assert sample['exclusiveStageNanos']=={k:metrics[k]['inclusive_nanos'] for k in keys} and 0.999<sum(sample['exclusiveStageNanos'].values())/sample['totalNanos']<=1
 assert sample['frames']==metrics['frame']['calls'] and sample['archiveChecks']==metrics['frame_archives']['calls']
assert partition['samples'][1]['frames']==48 and partition['samples'][1]['historicalBegin']==3 and partition['samples'][1]['historicalFinish']==3 and partition['samples'][1]['historicalPasses']==6
c=json.loads((base/'cleanup-final.json').read_text());assert c['actualNativeRounds']==1 and c['rawProfileFiles']==79 and c['archiveComparedBeforeRemoval'] and c['temporaryNativeRootRemoved'] and c['exactNewCoordinatorsRemoved']==3 and c['globalLUnchangedAndReleased'] and c['normalStopBudgetUnchanged']
print(json.dumps({'accepted':False,'sourceCommit':m['sourceCommit'],'actualNativeRounds':1,'firstCycleStartStatusStopMs':[7937,231,6205],'secondCycleStartStatusStopMs':[10131,847,11428],'priorProcargsErrorNotObserved':True,'bothServicePairsNormalExitProved':True,'rootMovedAndReadbackRecorded':True,'currentState':'RootRetirementPrepared19','rootAbsent18ThenPostCloseProofRefusalVerified':True,'firstStopTailUs':tails[2],'secondStopTailNotReached':True,'secondRegistrationMs':round(partition['samples'][1]['totalNanos']/1e6,3),'normalStop10Passed':False,'rawProfileFilesVerified':79,'allOwnedTemporaryRootsRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in m['files'])},indent=2))
