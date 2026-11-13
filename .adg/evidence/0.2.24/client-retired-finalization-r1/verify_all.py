from pathlib import Path
import json,gzip,hashlib,base64
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(name):return gzip.decompress((base/(name+'.txt.gz')).read_bytes()).decode()
for name,passed,ignored in [('retired-review-all',5,0),('regression-1',320,9),('regression-2',29,6),('regression-3',20,1),('regression-4',4,1),('final-native-entry-compile',4,1)]:
 assert json.loads((base/(name+'.json')).read_text())['exitCode']==0 and f'{passed} passed; 0 failed; {ignored} ignored' in log(name)
assert json.loads((base/'ui-recovery.json').read_text())['exitCode']==0 and '94 passed' in log('ui-recovery')
for name in ['final-clippy','format','typecheck','diff-check']:assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
assert json.loads((base/'root-review-compile.json').read_text())['exitCode']==101 and 'cannot be known at compilation time' in log('root-review-compile')
review=json.loads((base/'review-final.json').read_text());assert review['actualDiffReviewed'] and review['initialP1Resolved'] and review['validationP2Resolved'] and review['blockingFindings']==[] and not review['reviewerEditedOrExecuted']
assert json.loads((base/'review-p1.json').read_text())['result']=='initial production candidate blocked'
s=json.loads((base/'tested-source.json').read_text());assert s['sourceCommit']==m['sourceCommit'] and len(s['files'])==11 and not s['fixedCandidate'] and s['officialServiceRounds']==0
c=json.loads((base/'cleanup.json').read_text());assert c['unitTmpEmpty'] and c['archiveComparedBeforeRemoval'] and c['removed'] and c['officialServiceRounds']==0
for row in c['archived']:
 b=(base/row['archive']).read_bytes();b=gzip.decompress(b) if row['compressed'] else b;assert len(b)==row['rawBytes'] and hashlib.sha256(b).hexdigest()==row['rawSha256']
cache=json.loads((base/'ui-cache-cleanup.json').read_text());assert cache['archiveRoundtripAndFilesCompared'] and cache['onlyExactOwnedCacheRootsRemoved'] and cache['unitTmpNowEmpty']
compressed=(base/'ui-compiler-cache-bytes.json.gz').read_bytes();assert hashlib.sha256(compressed).hexdigest()==cache['archiveSha256'];raw=json.loads(gzip.decompress(compressed));assert len(raw['files'])==cache['rawFiles']==119
for row in raw['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts;b=base64.b64decode(row['rawBase64'],validate=True);assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
print(json.dumps({'accepted':False,'sourceCommit':m['sourceCommit'],'differentBackendRegressionTestsPassed':373,'frontendRecoveryTestsPassed':94,'retiredReviewTestsIncluded':5,'initialReviewP1AndEvidenceP2Resolved':True,'oldNormalStopFailureNotReclassified':True,'productionExplicitRecoveryNativeResultPending':True,'officialServiceRounds':0,'cacheFilesArchivedBeforeRemoval':119,'ownedTemporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in m['files'])},indent=2))
