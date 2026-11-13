from pathlib import Path
import json,gzip,hashlib
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(name):return gzip.decompress((base/(name+'.txt.gz')).read_bytes()).decode()
for name,passed,ignored in [('lease-directory-tests',2,0),('regression-1',29,6),('regression-2',35,0),('regression-3',4,1),('retained-readback-boundary',1,0)]:
 assert json.loads((base/(name+'.json')).read_text())['exitCode']==0 and f'{passed} passed; 0 failed; {ignored} ignored' in log(name)
for name in ['clippy-final','format-final','diff-final']:assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
prefix='NATIVE_LEASE_DIRECTORY_COMPARISON ';d=json.loads(next(v for v in log('lease-directory-tests').splitlines() if prefix in v).split(prefix,1)[1]);assert d['fourLocksHeldThenNormallyReleased'];samples=d['samples'];assert [v['reference'] for v in samples]==[True,False,False,True]
assert [v['metrics']['protection']['calls'] for v in samples]==[134,106,106,134]
assert [v['metrics']['xattr_native_call']['calls'] for v in samples]==[214,170,170,214]
assert all(v['metrics']['capture_file']['calls']==1 for v in samples)
s=json.loads((base/'tested-source.json').read_text());assert s['sourceCommit']==m['sourceCommit'] and len(s['files'])==5 and not s['fixedCandidate'] and s['officialServiceRounds']==0
assert json.loads((base/'review.json').read_text())['blockingFindings']==[]
c=json.loads((base/'cleanup.json').read_text());assert c['unitTmpEmpty'] and c['archiveComparedBeforeRemoval'] and c['removed'] and c['officialServiceRounds']==0
for row in c['archived']:
 b=(base/row['archive']).read_bytes();b=gzip.decompress(b) if row['compressed'] else b;assert len(b)==row['rawBytes'] and hashlib.sha256(b).hexdigest()==row['rawSha256']
print(json.dumps({'accepted':False,'sourceCommit':m['sourceCommit'],'differentAffectedTestsPassed':69,'fourSqliteLocksHeldAndReleasedProved':True,'protectionCalls':[134,106],'nativeXattrCalls':[214,170],'wholeRetiredInspectionDeletionNotAdopted':True,'finalPlanReloadBoundaryRegressionPassed':True,'officialServiceRounds':0,'normalStop10NotAccepted':True,'allOwnedTemporaryRootsRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in m['files'])},indent=2))
