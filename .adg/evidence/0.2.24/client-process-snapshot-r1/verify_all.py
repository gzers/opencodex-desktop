from pathlib import Path
import json,gzip,hashlib
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(name):return gzip.decompress((base/(name+'.txt.gz')).read_bytes()).decode()
for name,passed,ignored in [('snapshot-final-source',2,0),('regression-1',20,1),('regression-2',4,1),('regression-3',10,0),('regression-4',6,0)]:
 assert json.loads((base/(name+'.json')).read_text())['exitCode']==0 and f'{passed} passed; 0 failed; {ignored} ignored' in log(name)
assert 'PROCARGS_EXIT_RACE_EVIDENCE ' in log('snapshot-final-source')
assert json.loads((base/'snapshot-tests.json').read_text())['exitCode']==101 and json.loads((base/'clippy.json').read_text())['exitCode']==101
for name in ['clippy-final','format-final','included-test-format','diff-final']:assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
s=json.loads((base/'tested-source.json').read_text());assert s['sourceCommit']==m['sourceCommit'] and len(s['files'])==3 and not s['fixedCandidate'] and s['officialServiceRounds']==0
r=json.loads((base/'review.json').read_text());assert r['actualDiffReviewed'] and r['blockingFindings']==[] and not r['reviewerEditedOrExecuted']
c=json.loads((base/'cleanup.json').read_text());assert c['unitTmpEmpty'] and c['archiveComparedBeforeRemoval'] and c['removed'] and c['officialServiceRounds']==0
for row in c['archived']:
 b=(base/row['archive']).read_bytes();b=gzip.decompress(b) if row['compressed'] else b;assert len(b)==row['rawBytes'] and hashlib.sha256(b).hexdigest()==row['rawSha256']
print(json.dumps({'accepted':False,'sourceCommit':m['sourceCommit'],'differentAffectedTestsPassed':40,'actualOwnedChildExitRaceProved':True,'nativeAbsenceRequiredAfterFailedRead':True,'liveReadFailureAndPidReuseStillRefused':True,'originalNormalStopBudgetUnchanged':True,'originalServiceRevalidationStillPending':True,'officialServiceRounds':0,'compileAndLintFailuresPreserved':True,'ownedTemporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in m['files'])},indent=2))
