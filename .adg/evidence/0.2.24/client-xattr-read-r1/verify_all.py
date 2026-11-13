from pathlib import Path
import json,gzip,hashlib
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(name):return gzip.decompress((base/(name+'.txt.gz')).read_bytes()).decode()
for name,wanted in [('xattr-complete','6 passed; 0 failed; 0 ignored'),('originals-regression','305 passed; 0 failed; 9 ignored'),('coordination-regression','27 passed; 0 failed; 6 ignored'),('command-regression','4 passed; 0 failed; 1 ignored')]:
 assert json.loads((base/(name+'.json')).read_text())['exitCode']==0 and wanted in log(name)
for name in ['clippy','format','diff-check']:assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
prefix='XATTR_FULL_HISTORY_COMPARISON ';line=next(v for v in log('xattr-complete').splitlines() if prefix in v);samples=json.loads(line.split(prefix,1)[1])['samples'];assert [v['reference'] for v in samples]==[True,False,False,True]
for key in ['frame','frame_archives','capture_file','root_provenance','root_namespace','protection','historical_begin','historical_finish','historical_full_pass']:
 assert len({v['metrics'][key]['calls'] for v in samples})==1,key
assert all(v['metrics']['frame']['calls']==48 and v['metrics']['frame_archives']['calls']==48 and v['metrics']['historical_begin']['calls']==3 and v['metrics']['historical_finish']['calls']==3 and v['metrics']['historical_full_pass']['calls']==6 for v in samples)
assert [v['metrics']['xattr_native_call']['calls'] for v in samples]==[187405,103631,103631,187405]
source=json.loads((base/'tested-source.json').read_text());assert source['sourceCommit']==manifest['sourceCommit'] and len(source['files'])==4 and not source['fixedCandidate'] and source['officialServiceRounds']==0
assert json.loads((base/'review-p1.json').read_text())['result']=='initial proposal blocked'
review=json.loads((base/'review-final.json').read_text());assert review['initialP1Resolved'] and review['actualDiffReviewed'] and review['blockingFindings']==[] and not review['reviewerEditedOrExecuted']
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['unitTmpEmpty'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['removed'] and cleanup['officialServiceRounds']==0
for item in cleanup['archived']:
 raw=(base/item['archive']).read_bytes();raw=gzip.decompress(raw) if item['compressed'] else raw;assert len(raw)==item['rawBytes'] and hashlib.sha256(raw).hexdigest()==item['rawSha256']
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'differentAffectedTestsPassed':336,'modulePassed':305,'moduleOriginallyIgnored':9,'coordinationPassed':27,'coordinationOriginallyIgnored':6,'commandPassed':4,'commandOriginallyIgnored':1,'resourceForkFullTailAndOverflowRegressionPassed':True,'calibratedFrameCount':48,'xattrNativeCalls':[187405,103631],'registrationMs':[round(v['elapsedNanos']/1e6,3) for v in samples],'sourceAndProtectionObservationCountsUnchanged':True,'officialServiceRounds':0,'normalStop10NotAccepted':True,'unitTemporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
