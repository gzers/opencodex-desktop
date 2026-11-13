from pathlib import Path
import json,hashlib,gzip
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(label):return gzip.decompress((base/(label+'.txt.gz')).read_bytes()).decode()
for label in ['cost-partition','stopped-partition','transaction-regression','command-regression','clippy','format','diff-check']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
assert '1 passed; 0 failed' in log('stopped-partition') and '4 passed; 0 failed; 0 ignored' in log('transaction-regression') and '4 passed; 0 failed; 1 ignored' in log('command-regression')
prefix='RETIREMENT_TRANSACTION_COST ';value=json.loads(next(v for v in log('stopped-partition').splitlines() if prefix in v).split(prefix,1)[1]);samples=value['samples'];assert len(samples)==2 and [v['secondGeneration'] for v in samples]==[False,True]
keys=['transaction_storage_begin','transaction_source','transaction_admission','transaction_sql','transaction_candidate','transaction_commit','transaction_postcommit','registration_facts']
for sample in samples:
 m=sample['metrics'];assert all(m[key]['calls']==1 for key in keys)
 partition=sum(m[key]['inclusive_nanos'] for key in keys);assert partition==sample['partitionNanos'] and partition<=sample['totalNanos'] and sample['unattributedNanos']==sample['totalNanos']-partition
 assert partition/sample['totalNanos']>0.999
second=samples[1];assert second['metrics']['historical_begin']['calls']==3 and second['metrics']['historical_finish']['calls']==3 and second['metrics']['historical_full_pass']['calls']==6
source=json.loads((base/'tested-source.json').read_text());assert source['sourceCommit']==manifest['sourceCommit'] and len(source['files'])==5 and source['nativeServiceRounds']==0
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['unitTmpEmpty'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['removed'] and cleanup['nativeServiceRounds']==0
for item in cleanup['archived']:
 raw=(base/item['archive']).read_bytes();raw=gzip.decompress(raw) if item['compressed'] else raw;assert len(raw)==item['rawBytes'] and hashlib.sha256(raw).hexdigest()==item['rawSha256']
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'differentTestsPassed':9,'firstGenerationSyntheticMs':round(samples[0]['totalNanos']/1e6,3),'secondGenerationSyntheticMs':round(second['totalNanos']/1e6,3),'secondExclusiveStageMs':{key:round(second['metrics'][key]['inclusive_nanos']/1e6,3) for key in keys},'partitionCoverageOver99_9Percent':True,'originalFullHistoryObservations':3,'nativeServiceRounds':0,'normalStop10NotAccepted':True,'ownedTemporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
