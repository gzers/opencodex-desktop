from pathlib import Path
import json,gzip,hashlib
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
def log(name):return gzip.decompress((base/(name+'.txt.gz')).read_bytes()).decode()
for label,text in [('regression-1','13 passed; 0 failed; 1 ignored'),('regression-2','11 passed; 0 failed; 0 ignored'),('final-coordination','27 passed; 0 failed; 6 ignored'),('regression-4','4 passed; 0 failed; 1 ignored')]:
 assert json.loads((base/(label+'.json')).read_text())['exitCode']==0 and text in log(label)
for label in ['final-clippy','final-format','final-diff-check']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
source=json.loads((base/'tested-source.json').read_text());assert source['sourceCommit']==manifest['sourceCommit'] and len(source['files'])==3 and source['officialServiceRounds']==0
image_source=json.loads((base/'image-only-tested-source.json').read_text());assert len(image_source['files'])==2
for item in image_source['files']:assert next(v for v in source['files'] if v['path']==item['path'])==item
out=base/'official-image-path';runner=json.loads((out/'runner-summary.json').read_text());assert runner['exitCode']==0 and runner['exactOneTestPassed'] and runner['serviceActions']==0 and runner['allNetworkDenied'] and runner['dailyHomeDenied'] and runner['binarySha256']==image_source['testBinarySha256']
assert '1 passed; 0 failed; 0 ignored' in log('official-image-path/stdout')
comparison=json.loads((out/'image-path-comparison.json').read_text());assert comparison['rowsExactlyEqual'] and comparison['treeSha256']=='a29dedc203311ca4d6d4005db94b4a10d4fb84409283669e25272f080756b790' and comparison['officialArchiveSha256']=='b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'
samples=comparison['samples'];assert [v['reference'] for v in samples]==[True,False,False,True]
for v in samples:assert v['fileCount']==13393 and v['entryCount']==15646 and v['metrics']['stages']['file-read-hash'][0]==13393 and v['metrics']['stages']['existing-path-boundary'][0]==17901
for key in ['existing-path-boundary','directory-path-admission','entry-inspect-name','file-admission-open','file-read-hash','file-total']:assert len({v['metrics']['stages'][key][0] for v in samples})==1
cleanup=json.loads((out/'cleanup.json').read_text());assert cleanup['resultBytesComparedBeforeRemoval'] and cleanup['ownedProcessesAbsent'] and cleanup['removed'] and cleanup['serviceActions']==0 and not cleanup['coordinatorsOpenedOrCreated']
cleanup=json.loads((base/'cleanup-final.json').read_text());assert cleanup['unitTmpEmpty'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['removed'] and cleanup['officialServiceRounds']==0 and cleanup['officialReadonlyImageRounds']==1
for item in cleanup['archived']:
 raw=(base/item['archive']).read_bytes();raw=gzip.decompress(raw) if item['compressed'] else raw;assert len(raw)==item['rawBytes'] and hashlib.sha256(raw).hexdigest()==item['rawSha256']
assert json.loads((base/'review.json').read_text())['blockingFindings']==[]
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'performanceCommit':source['performanceCommit'],'differentUnitRegressionTestsPassed':55,'officialReadonlyImageTestsPassed':1,'officialServiceRounds':0,'fullFileCount':13393,'allPathBoundaries':17901,'completeTreeDigestUnchanged':True,'officialImageABBAms':[round(v['elapsedNanos']/1e6,3) for v in samples],'nativeSyscallReduction':'static per-successful-component 1+2d to 1+d; inspect count unchanged, not measured syscalls','normalStop10NotAccepted':True,'allOwnedTemporaryRootsRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
