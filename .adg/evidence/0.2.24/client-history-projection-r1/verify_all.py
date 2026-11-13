from pathlib import Path
import json,hashlib,re,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());expected=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in expected;expected.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==expected
unique=set()
for name in ['projection-controls-final','projection-comparison','history-regression','index-regression','rollover-regression']:
 assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
 log=(base/(name+'.log')).read_text();names={n for n,status in re.findall(r'^test (\S+) \.\.\. (.*)$',log,re.M) if not status.startswith('ignored')};summary=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)[-1];assert len(names)==int(summary[0]);unique|=names
assert len(unique)==107
for name in ['strict-clippy','formatting']:assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
source=json.loads((base/'source-final.json').read_text());build=json.loads((base/'source-build.json').read_text());assert source['files']==build['files'] and len(source['files'])==4 and source['actualLibraryTestBinarySha256']==build['actualLibraryTestBinarySha256'] and source['committedSource']==manifest['sourceCommit']
comparison=json.loads(subprocess.check_output([sys.executable,str(base/'verify_comparison.py')],text=True));assert comparison==json.loads((base/'verified-comparison.json').read_text())
assert comparison['counts']['wholeHistoricalPasses']==[4,2] and comparison['incompleteObservationResourceTest']['finalHandles']==comparison['incompleteObservationResourceTest']['initialHandles']
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['temporaryUnitRootRemoved'] and cleanup['unitTmpEmptyBeforeRemoval'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['officialServiceStartedByUnitSuite'] is False
print(json.dumps({'accepted':False,'manifestFiles':len(expected),'manifestBytes':sum(v['bytes'] for v in manifest['files']),'sourceCommit':manifest['sourceCommit'],'uniquePassingFunctions':len(unique),'wholeHistoricalPasses':[4,2],'syntheticIndexObservationMeanChangePercent':comparison['meanChangePercent'],'incompleteObservationsWithoutFdGrowth':32,'originalNativeFailurePreserved':True,'scope':'sealed projection unit/reference validation only; independent release/native work is not claimed passed here'},indent=2))
