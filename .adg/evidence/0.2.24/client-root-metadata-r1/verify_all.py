from pathlib import Path
import gzip,hashlib,json
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(label):return gzip.decompress((base/(label+'.txt.gz')).read_bytes()).decode()
expected={'root-metadata-tests':(4,0),'regression-1':(11,0),'regression-2':(20,0),'regression-3':(9,0),'regression-4':(9,0),'regression-5':(4,0),'regression-6':(4,1)}
for name,(passed,ignored) in expected.items():
 assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
 assert f'{passed} passed; 0 failed; {ignored} ignored' in log(name)
for label in ['clippy','format','diff-check']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
prefix='ROOT_METADATA_REGISTRATION ';line=next(v for v in log('root-metadata-tests').splitlines() if prefix in v);samples=json.loads(line.split(prefix,1)[1])['samples'];assert [v['reference'] for v in samples]==[True,False,False,True]
for metric in ['frame','frame_archives','capture_file','root_provenance','root_namespace','historical_begin','historical_finish','historical_full_pass']:
 assert len({v['metrics'][metric]['calls'] for v in samples})==1,metric
assert [v['metrics']['protection']['calls'] for v in samples]==[40322,39334,39334,40322]
assert all(v['metrics']['root_provenance']['calls']==38 and v['metrics']['root_namespace']['calls']==37 and v['metrics']['frame_archives']['calls']==26 for v in samples)
review=json.loads((base/'review-first-last.json').read_text());assert review['result']=='rejected' and not review['reviewerEditedOrExecuted']
review=json.loads((base/'review-metadata.json').read_text());assert review['actualDiffReviewed'] and review['blockingFindings']==[] and not review['reviewerEditedOrExecuted']
source=json.loads((base/'tested-source.json').read_text());assert source['sourceCommit']==manifest['sourceCommit'] and len(source['files'])==5 and not source['fixedCandidate']
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['unitTmpEmpty'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['removed'] and cleanup['officialServiceRounds']==0
for item in cleanup['archived']:
 raw=(base/item['archive']).read_bytes();raw=gzip.decompress(raw) if item['compressed'] else raw;assert len(raw)==item['rawBytes'] and hashlib.sha256(raw).hexdigest()==item['rawSha256']
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'differentTestsPassed':58,'firstLastSourceReductionRejected':True,'perFrameSourceAndArchiveGatesKept':True,'registrationProtectionCalls':[40322,39334],'registrationMs':[round(v['elapsedNanos']/1e6,3) for v in samples],'officialServiceRounds':0,'normalStop10NotAccepted':True,'unitTemporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
