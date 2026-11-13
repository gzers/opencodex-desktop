from pathlib import Path
import json,hashlib,re,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());expected=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in expected;expected.add(item['path'])
 raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==expected
unique=set()
for label in ['union-only-history','union-only-pins','union-only-originals','union-only-index','union-only-retirement','union-only-restore','union-only-creation','union-only-capture']:
 assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
 log=(base/(label+'.log')).read_text();names={n for n,status in re.findall(r'^test (\S+) \.\.\. (.*)$',log,re.M) if not status.startswith('ignored')}
 count=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)[-1];assert len(names)==int(count[0]);unique|=names
assert len(unique)==186
for label in ['union-only-clippy','union-only-format','union-only-capture-reference']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
for label in ['prefix-controls','pinned-namespace-prefix','pinned-namespace-corrected','strict-clippy','final-originals','final-index','isolated-2']:
 assert json.loads((base/(label+'.json')).read_text())['exitCode']==101,label
source=json.loads((base/'source-final.json').read_text());build=json.loads((base/'source-build.json').read_text());assert len(source['files'])==7 and source['files']==build['files']
assert source['actualLibraryTestBinarySha256']==build['actualLibraryTestBinarySha256'] and source['committedSource']==manifest['sourceCommit']
result=json.loads(subprocess.check_output([sys.executable,str(base/'verify_final_union.py')],text=True));assert result==json.loads((base/'verified-final-union.json').read_text())
assert result['scopedProvenanceReuseRejected'] and result['counts']['fullProvenance']==[10,10] and result['counts']['protection']==[6683,6299]
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['temporaryRootRemoved'] and cleanup['unitTmpEmptyBeforeRemoval'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['officialServiceStarted'] is False
print(json.dumps({'accepted':False,'manifestFiles':len(expected),'manifestBytes':sum(v['bytes'] for v in manifest['files']),'sourceCommit':manifest['sourceCommit'],'uniquePassingFunctions':len(unique),'finalImplementation':'ancestor union only; every frame full source verification unchanged','protectionCounts':[6683,6299],'syntheticObservationMeanChangePercent':result['meanChangePercent'],'rejectedProofReuseAndAllFailuresRetained':True,'originalSecondStopFailureStillEffective':True,'scope':'local measured/validated observation optimization; no official service round, fixed app or whole-version acceptance'},indent=2))
