from pathlib import Path
import json,gzip,hashlib,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
source=json.loads((base/'compiled-source.json').read_text());assert source['baseCommit']==manifest['sourceCommit'] and source['cleanSource'] and len(source['files'])==21 and source['binarySha256']=='9a5eb08ad9ff15ccd57a76eeb9e97b9a16374ab99990b1efeb4d507914fe2971'
assert json.loads((base/'release-build-final.json').read_text())['exitCode']==0
for script,record in [('verify_failed.py','verified-failure.json'),('verify_progress.py','verified-progress.json')]:assert json.loads(subprocess.check_output([sys.executable,str(base/script)],text=True))==json.loads((base/record).read_text())
out=base/'native-partition-r1';lines=gzip.decompress((out/'native.stderr.txt.gz').read_bytes()).decode().splitlines();prefix='ROOT_REGISTRATION_COST ';costs=[json.loads(v.split(prefix,1)[1]) for v in lines if v.startswith(prefix)];assert len(costs)==2 and all(v['passed'] and not v['accepted'] for v in costs)
record=json.loads((base/'verified-native-partition.json').read_text());keys=['transaction_storage_begin','transaction_source','transaction_admission','transaction_sql','transaction_candidate','transaction_commit','transaction_postcommit','registration_facts']
for i,cost in enumerate(costs):
 m=cost['metrics'];sample=record['samples'][i];assert sample['totalNanos']==m['registration_total']['inclusive_nanos'];assert all(m[key]['calls']==1 for key in keys)
 assert sample['exclusiveStageNanos']=={key:m[key]['inclusive_nanos'] for key in keys}
 assert sample['partitionNanos']==sum(sample['exclusiveStageNanos'].values()) and 0.999<sample['partitionNanos']/sample['totalNanos']<=1
 assert sample['unattributedNanos']==sample['totalNanos']-sample['partitionNanos']
 for key,count in sample['authenticationCounts'].items():assert count==m.get(key,{}).get('calls',0)
assert record['samples'][0]['authenticationCounts']['historical_begin']==0
assert record['samples'][1]['authenticationCounts']['historical_begin']==3 and record['samples'][1]['authenticationCounts']['historical_finish']==3 and record['samples'][1]['authenticationCounts']['historical_full_pass']==6 and record['samples'][1]['authenticationCounts']['frame']==48
cleanup=json.loads((base/'cleanup-final.json').read_text());assert cleanup['actualNativeRounds']==1 and cleanup['archiveComparedBeforeRemoval'] and cleanup['temporaryNativeRootRemoved'] and cleanup['rawProfileFiles']==77 and cleanup['exactNewCoordinatorsRemoved']==3 and cleanup['globalLUnchangedAndReleased'] and cleanup['normalStopBudgetUnchanged']
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'actualNativeRounds':1,'firstCycleStartStatusStopMs':[8129,226,7604],'secondCycleStartStatusStopMs':[12812,1231,11681],'secondRegistrationTotalMs':round(record['samples'][1]['totalNanos']/1e6,3),'secondExclusiveStageMs':{key:round(value/1e6,3) for key,value in record['samples'][1]['exclusiveStageNanos'].items()},'secondRegistrationFrameCount':48,'syntheticLighterFixtureFrameCount':30,'retainedState':'RootRetirementPrepared17','rootMoved':False,'rawProfileFilesVerified':77,'allOwnedTemporaryRootsRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
