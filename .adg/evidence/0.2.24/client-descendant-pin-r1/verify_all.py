from pathlib import Path
import gzip,hashlib,json,re
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(name):return gzip.decompress((base/(name+'.txt.gz')).read_bytes()).decode()
for name in ['originals-regression','descendant-final','command-regression','clippy','format','diff-check']:assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
assert json.loads((base/'descendant-tests.json').read_text())['exitCode']==101 and '2 passed; 1 failed' in log('descendant-tests')
assert '3 passed; 0 failed; 0 ignored' in log('descendant-final')
assert '290 passed; 0 failed; 9 ignored' in log('originals-regression')
assert '4 passed; 0 failed; 1 ignored' in log('command-regression')
line=next(s for s in log('originals-regression').splitlines() if 'RETIREMENT_DESCENDANT_COMPARISON ' in s)
samples=json.loads(line.split('RETIREMENT_DESCENDANT_COMPARISON ',1)[1])['samples'];assert [v['reference'] for v in samples]==[True,False,False,True]
for metric in ['frame','capture_file','retirement_plan_load','root_provenance','historical_begin','historical_finish','historical_full_pass']:
 assert len({v['metrics'][metric]['calls'] for v in samples})==1,metric
assert all(v['metrics']['historical_begin']['calls']==3 and v['metrics']['historical_finish']['calls']==3 and v['metrics']['historical_full_pass']['calls']==6 for v in samples)
assert [v['metrics']['protection']['calls'] for v in samples]==[31030,29423,29423,31030]
source=json.loads((base/'source-under-test.json').read_text());assert source['sourceCommit']==manifest['sourceCommit'] and len(source['files'])==6 and source['nativeOriginalServiceRounds']==0
review=json.loads((base/'review.json').read_text());assert review['actualDiffReviewed'] and review['blockingFindings']==[] and not review['reviewerRanTestsOrServices']
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['archiveComparedBeforeRemoval'] and cleanup['unitTmpEmpty'] and cleanup['removed'] and cleanup['originalServiceRounds']==0
for item in cleanup['archived']:
 raw=(base/item['archive']).read_bytes();raw=gzip.decompress(raw) if item['compressed'] else raw;assert len(raw)==item['rawBytes'] and hashlib.sha256(raw).hexdigest()==item['rawSha256']
remote=json.loads((base/'windows-remote-observation.json').read_text());assert remote['newContributionSinceIntegratedW0'] is False
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'differentTestsPassed':294,'modulePassed':290,'moduleOriginallyIgnored':9,'commandPassed':4,'commandOriginallyIgnored':1,'firstFixtureFailurePreserved':True,'completeHistoryAuditsPerRegistration':3,'historicalFullPasses':6,'registrationProtectionCalls':[31030,29423],'registrationMs':[round(v['elapsedNanos']/1e6,3) for v in samples],'actualNewOriginalServiceRounds':0,'normalStop10NotAccepted':True,'temporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
