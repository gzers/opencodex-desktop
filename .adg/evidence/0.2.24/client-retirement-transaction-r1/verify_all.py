from pathlib import Path
import hashlib,json,re
base=Path(__file__).resolve().parent
manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
source=json.loads((base/'tested-source.json').read_text());assert source['sourceCommit']==manifest['sourceCommit'] and len(source['files'])==4 and not source['isFixedAppCandidate']
expected={'transaction-final':(4,0),'comparison-final':(1,0),'regression-1':(7,0),'regression-2':(9,0),'regression-3':(1,0),'command-regression':(4,1)}
for label,(passed,ignored) in expected.items():
 assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
 assert f'{passed} passed; 0 failed; {ignored} ignored' in (base/(label+'.txt')).read_text()
for label in ['clippy','format','diff-check']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
line=next(s for s in (base/'comparison-final.txt').read_text().splitlines() if s.startswith('RETIREMENT_TRANSACTION_COMPARISON '))
comparison=json.loads(line.split(' ',1)[1]);assert comparison['accepted'] is False and len(comparison['samples'])==4
for s in comparison['samples']:
 assert s['metrics']['historical_begin']['calls']==(5 if s['reference'] else 3)
 assert s['metrics']['historical_finish']['calls']==(5 if s['reference'] else 3)
 assert s['metrics']['historical_full_pass']['calls']==(10 if s['reference'] else 6)
assert '0 passed; 0 failed' in (base/'regression-4.txt').read_text()
review=json.loads((base/'review-final.json').read_text());assert review['actualDiffReviewed'] and review['blockingFindings']==[] and not review['reviewerRanTestsOrService']
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['actualNativeRounds']==0 and cleanup['unitTmpEmpty'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['removed']
for item in cleanup['archived']:
 raw=(base/item['archive']).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'uniqueAffectedTestsPassed':25,'actualNewNativeRounds':0,'fullHistoryAudits':[5,3],'comparisonMs':[round(s['elapsedNanos']/1e6,3) for s in comparison['samples']],'preSqlCandidatePostcommitIntegrityRetained':True,'normalStop10NotAccepted':True,'zeroTestAttemptExcluded':True,'ownedTemporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
