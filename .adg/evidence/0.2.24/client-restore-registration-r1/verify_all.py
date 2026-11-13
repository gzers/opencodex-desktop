from pathlib import Path
import gzip,hashlib,json
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(name):return gzip.decompress((base/(name+'.txt.gz')).read_bytes()).decode()
for label in ['registration-tests','registration-verified','index-regression','command-regression','clippy','format','diff-check']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
assert json.loads((base/'registration-final.json').read_text())['exitCode']==101 and 'error[E0277]' in log('registration-final')
for name,wanted in [('registration-verified','4 passed; 0 failed; 0 ignored'),('index-regression','28 passed; 0 failed; 2 ignored'),('command-regression','4 passed; 0 failed; 1 ignored')]:assert wanted in log(name)
prefix='RESTORE_REGISTRATION_COMPARISON ';line=next(v for v in log('registration-verified').splitlines() if prefix in v);samples=json.loads(line.split(prefix,1)[1])['samples'];assert [v['reference'] for v in samples]==[True,False,False,True]
assert [v['metrics']['historical_begin']['calls'] for v in samples]==[4,3,3,4]
assert [v['metrics']['historical_finish']['calls'] for v in samples]==[4,3,3,4]
assert [v['metrics']['historical_full_pass']['calls'] for v in samples]==[8,6,6,8]
source=json.loads((base/'tested-source.json').read_text());assert source['sourceCommit']==manifest['sourceCommit'] and len(source['files'])==4 and source['nativeServiceRounds']==0 and not source['fixedCandidate']
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['archiveComparedBeforeRemoval'] and cleanup['unitTmpEmpty'] and cleanup['removed'] and cleanup['officialServiceRounds']==0
for item in cleanup['archived']:
 raw=(base/item['archive']).read_bytes();raw=gzip.decompress(raw) if item['compressed'] else raw;assert len(raw)==item['rawBytes'] and hashlib.sha256(raw).hexdigest()==item['rawSha256']
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'differentTestsPassed':36,'newTestsPassed':4,'indexPassed':28,'indexOriginallyIgnored':2,'commandPassed':4,'commandOriginallyIgnored':1,'compileFailurePreserved':True,'fullHistoryObservations':[4,3],'historicalFullPasses':[8,6],'registrationMs':[round(v['elapsedNanos']/1e6,3) for v in samples],'officialServiceRounds':0,'normalStop10NotAccepted':True,'temporaryRootRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
