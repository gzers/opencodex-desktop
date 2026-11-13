from pathlib import Path
import json,hashlib,re,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());wanted=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in wanted;wanted.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==wanted
unique=set()
for label in ['prefix-controls','session-regression','index-regression','semantic-controls-final']:
 assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
 log=(base/(label+'.log')).read_text();names={n for n,s in re.findall(r'^test (\S+) \.\.\. (.*)$',log,re.M) if not s.startswith('ignored')};c=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)[-1];assert len(names)==int(c[0]);unique|=names
assert len(unique)==98
for label in ['clippy-delivery','format-delivery']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
source=json.loads((base/'source-final.json').read_text());build=json.loads((base/'source-build.json').read_text());prior=json.loads((base/'source-before-semantic-strengthening.json').read_text())
assert len(source['files'])==3 and source['files']==build['files'] and source['committedSource']==manifest['sourceCommit'] and source['actualLibraryTestBinarySha256']==build['actualLibraryTestBinarySha256']
assert [a['path'] for a,b in zip(prior['files'],source['files']) if a['sha256']!=b['sha256']]==['apps/desktop/tauri/src/infrastructure/macos_client_root_stop_binding_tests.rs']
comparison=json.loads(subprocess.check_output([sys.executable,str(base/'verify_comparison.py')],text=True));assert comparison==json.loads((base/'verified-comparison.json').read_text())
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['temporaryRootRemoved'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['unitTmpEmptyBeforeRemoval'] and cleanup['officialServiceStarted'] is False
print(json.dumps({'accepted':False,'manifestFiles':len(wanted),'manifestBytes':sum(v['bytes'] for v in manifest['files']),'sourceCommit':manifest['sourceCommit'],'uniquePassingFunctions':len(unique),'newFunctions':2,'secondGenerationFrameReads':[24,20],'sameObservationOnly':True,'semanticDependencyPrecisionVerifiedOnFinalTestBuild':True,'previousNativeStopFailureStillEffective':True,'scope':'index scope-prefix deduplication; no new official service or fixed candidate acceptance'},indent=2))
