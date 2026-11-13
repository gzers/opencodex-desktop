from pathlib import Path
import json,hashlib,re,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());expected=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in expected;expected.add(item['path']);raw=(base/p).read_bytes()
 assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==expected
unique=set()
for label in ['plan-controls-final','history-comparison','history-regression','index-regression','rollover-regression']:
 assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
 log=(base/(label+'.log')).read_text();names={n for n,status in re.findall(r'^test (\S+) \.\.\. (.*)$',log,re.M) if not status.startswith('ignored')}
 counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)[-1];assert len(names)==int(counts[0]);unique|=names
assert len(unique)==104
for label in ['strict-clippy','formatting']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
source=json.loads((base/'source-final.json').read_text());build=json.loads((base/'source-build.json').read_text());assert source['files']==build['files'] and len(source['files'])==3
assert source['actualLibraryTestBinarySha256']==build['actualLibraryTestBinarySha256'] and source['committedSource']==manifest['sourceCommit']
comparison=json.loads(subprocess.check_output([sys.executable,str(base/'verify_comparison.py')],text=True));assert comparison==json.loads((base/'verified-comparison.json').read_text())
assert comparison['counts']['planLoads']==[6,2] and comparison['counts']['frames']==[8,8] and comparison['counts']['fullRootProvenance']==[10,10]
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['temporaryRootRemoved'] and cleanup['unitTmpEmptyBeforeRemoval'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['officialServiceStarted'] is False
print(json.dumps({'accepted':False,'manifestFiles':len(expected),'manifestBytes':sum(v['bytes'] for v in manifest['files']),'sourceCommit':manifest['sourceCommit'],'uniquePassingFunctions':len(unique),'planLoadsPerHistoricalOpen':[6,2],'frameValidations':[8,8],'fullRootProvenanceChecks':[10,10],'syntheticObservationMeanChangePercent':comparison['meanChangePercent'],'originalSecondStopFailureStillEffective':True,'scope':'readonly historical-plan optimization only; no official service round or fixed candidate acceptance'},indent=2))
