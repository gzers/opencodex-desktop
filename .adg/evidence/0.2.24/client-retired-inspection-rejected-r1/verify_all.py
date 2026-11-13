from pathlib import Path
import json,gzip,hashlib
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
source=json.loads((base/'rejected-source.json').read_text());assert source['baseCommit']==m['sourceCommit'] and not source['candidateCommitted']
assert json.loads((base/'closed-tests.json').read_text())['exitCode']==0 and '2 passed; 0 failed' in gzip.decompress((base/'closed-tests.txt.gz').read_bytes()).decode()
r=json.loads((base/'review-rejection.json').read_text());assert r['initialTestsDoNotRefuteFinding'] and r['codeRestoredToBase']
c=json.loads((base/'cleanup.json').read_text());assert c['unitTmpEmpty'] and c['archiveComparedBeforeRemoval'] and c['removed'] and c['officialServiceRounds']==0
for row in c['archived']:
 b=(base/row['archive']).read_bytes();b=gzip.decompress(b) if row['compressed'] else b;assert len(b)==row['rawBytes'] and hashlib.sha256(b).hexdigest()==row['rawSha256']
print(json.dumps({'accepted':False,'baseCommit':m['sourceCommit'],'disposition':'rejected-uncommitted-candidate-restored','twoStaticCandidateTestsPassedButInsufficient':True,'latePlanReadDetectionBoundaryUnprovenEquivalent':True,'officialServiceRounds':0,'ownedTemporaryRootRemoved':True,'manifestFiles':len(seen)},indent=2))
