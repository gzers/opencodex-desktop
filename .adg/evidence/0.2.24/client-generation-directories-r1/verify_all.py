from pathlib import Path
import json,gzip,hashlib
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(name):return gzip.decompress((base/(name+'.txt.gz')).read_bytes()).decode()
for name,passed,ignored in [('private-children-final',3,0),('regression-1',315,9),('regression-2',29,6),('regression-3',4,1)]:
 assert json.loads((base/(name+'.json')).read_text())['exitCode']==0 and f'{passed} passed; 0 failed; {ignored} ignored' in log(name)
for name in ['clippy','format','diff-check']:assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
prefix='PRIVATE_CHILDREN_FULL_HISTORY_COMPARISON ';line=next(v for v in log('private-children-final').splitlines() if prefix in v);samples=json.loads(line.split(prefix,1)[1])['samples'];assert [v['reference'] for v in samples]==[True,False,False,True]
for key in ['frame','frame_archives','capture_file','root_provenance','root_namespace','historical_begin','historical_finish','historical_full_pass']:assert len({v['metrics'][key]['calls'] for v in samples})==1,key
assert all(v['metrics']['frame']['calls']==48 and v['metrics']['frame_archives']['calls']==48 and v['metrics']['root_provenance']['calls']==61 and v['metrics']['root_namespace']['calls']==60 and v['metrics']['historical_begin']['calls']==3 and v['metrics']['historical_finish']['calls']==3 and v['metrics']['historical_full_pass']['calls']==6 for v in samples)
assert [v['metrics']['protection']['calls'] for v in samples]==[61744,54373,54373,61744]
assert [v['metrics']['xattr_native_call']['calls'] for v in samples]==[103631,91157,91157,103631]
initial=json.loads(next(v for v in log('private-children-tests').splitlines() if prefix in v).split(prefix,1)[1]);assert initial['samples'][0]['reference'] and initial['samples'][3]['reference']
s=json.loads((base/'tested-source.json').read_text());assert s['sourceCommit']==m['sourceCommit'] and len(s['files'])==5 and not s['fixedCandidate'] and s['officialServiceRounds']==0
r=json.loads((base/'review.json').read_text());assert r['actualDiffReviewed'] and r['blockingFindings']==[] and not r['reviewerEditedOrExecuted']
c=json.loads((base/'cleanup.json').read_text());assert c['unitTmpEmpty'] and c['archiveComparedBeforeRemoval'] and c['removed'] and c['officialServiceRounds']==0
for row in c['archived']:
 b=(base/row['archive']).read_bytes();b=gzip.decompress(b) if row['compressed'] else b;assert len(b)==row['rawBytes'] and hashlib.sha256(b).hexdigest()==row['rawSha256']
print(json.dumps({'accepted':False,'sourceCommit':m['sourceCommit'],'differentAffectedTestsPassed':348,'originalsModulePassed':315,'coordinationPassed':29,'commandPassed':4,'fullFrameAndArchiveChecksKept':48,'fullProvenanceChecksKept':61,'namespaceChecksKept':60,'historyBeginFinishPassesKept':[3,3,6],'protectionCalls':[61744,54373],'xattrNativeCalls':[103631,91157],'initialABBAms':[round(v['elapsedNanos']/1e6,3) for v in initial['samples']],'finalABBAms':[round(v['elapsedNanos']/1e6,3) for v in samples],'officialServiceRounds':0,'normalStop10NotAccepted':True,'ownedTemporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in m['files'])},indent=2))
