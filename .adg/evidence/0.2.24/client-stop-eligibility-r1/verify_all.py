from pathlib import Path
import json,gzip,hashlib
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(name):return gzip.decompress((base/(name+'.txt.gz')).read_bytes()).decode()
for name,passed,ignored in [('eligibility-all-final',6,0),('regression-1',8,0),('regression-2',8,0),('regression-3',6,0),('regression-4',4,1),('regression-5',11,0),('index-regression',28,2)]:
 assert json.loads((base/(name+'.json')).read_text())['exitCode']==0 and f'{passed} passed; 0 failed; {ignored} ignored' in log(name)
assert json.loads((base/'eligibility-tests.json').read_text())['exitCode']==101 and '2 passed; 2 failed' in log('eligibility-tests') and 'current session action is busy' in log('eligibility-tests')
assert json.loads((base/'eligibility-two-generations.json').read_text())['exitCode']==101 and 'is private' in log('eligibility-two-generations')
for name in ['clippy-final','format-final','diff-final']:assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
prefix='STOP_ELIGIBILITY_AUDIT_COUNTS ';line=next(v for v in log('eligibility-all-final').splitlines() if prefix in v);report=json.loads(line.split(prefix,1)[1]);keys=['frame','frame_archives','historical_begin','historical_finish','historical_full_pass','transaction_source','transaction_candidate','transaction_postcommit']
for modes in [('ordinary','projected'),('secondOrdinary','secondProjected')]:
 for key in keys:assert report[modes[0]].get(key,{}).get('calls',0)==report[modes[1]].get(key,{}).get('calls',0),key
assert report['ordinary']['frame']['calls']==12 and report['secondOrdinary']['frame']['calls']==72
assert report['secondProjected']['historical_begin']['calls']==5 and report['secondProjected']['historical_finish']['calls']==5 and report['secondProjected']['historical_full_pass']['calls']==10
s=json.loads((base/'tested-source.json').read_text());assert s['sourceCommit']==m['sourceCommit'] and len(s['files'])==8 and not s['fixedCandidate'] and s['officialServiceRounds']==0
r=json.loads((base/'review.json').read_text());assert r['actualDiffReviewed'] and r['blockingFindings']==[] and not r['reviewerEditedOrExecuted']
c=json.loads((base/'cleanup.json').read_text());assert c['unitTmpEmpty'] and c['archiveComparedBeforeRemoval'] and c['removed'] and c['officialServiceRounds']==0
for row in c['archived']:
 b=(base/row['archive']).read_bytes();b=gzip.decompress(b) if row['compressed'] else b;assert len(b)==row['rawBytes'] and hashlib.sha256(b).hexdigest()==row['rawSha256']
print(json.dumps({'accepted':False,'sourceCommit':m['sourceCommit'],'differentAffectedTestsPassed':71,'capacityFalseKeepsSuccessfulStoppedResult':True,'candidateMutationAndExpiryCannotReturnEligibility':True,'laterStartStillIndependentlyChecked':True,'firstCloseFramesWithoutAndWithProjection':[12,12],'fullSecondCloseFramesWithoutAndWithProjection':[72,72],'fullSecondHistoricalAuditsUnchanged':[5,5,10],'originalTransactionBoundariesKept':True,'fixtureLockAndCompileFailuresPreserved':True,'officialServiceRounds':0,'normalStop10NotAccepted':True,'ownedTemporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in m['files'])},indent=2))
