from pathlib import Path
import json,hashlib,re,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());wanted=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in wanted;wanted.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==wanted
unique=set()
for label in ['child-controls-final','history-comparison','pins-regression','restore-regression','index-regression','rollover-regression']:
 assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
 log=(base/(label+'.log')).read_text();names={n for n,s in re.findall(r'^test (\S+) \.\.\. (.*)$',log,re.M) if not s.startswith('ignored')};counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)[-1];assert len(names)==int(counts[0]);unique|=names
assert len(unique)==145
for label in ['strict-clippy','formatting']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
source=json.loads((base/'source-final.json').read_text());build=json.loads((base/'source-build.json').read_text());assert source['files']==build['files'] and len(source['files'])==4 and source['actualLibraryTestBinarySha256']==build['actualLibraryTestBinarySha256'] and source['committedSource']==manifest['sourceCommit']
comparison=json.loads(subprocess.check_output([sys.executable,str(base/'verify_comparison.py')],text=True));assert comparison==json.loads((base/'verified-comparison.json').read_text())
assert comparison['sourceAuthenticationUnchanged'] and comparison['counts']['fullProvenance']==[10,10] and comparison['counts']['pinnedOpen']==[68,30]
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['temporaryRootRemoved'] and cleanup['unitTmpEmptyBeforeRemoval'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['officialServiceStarted'] is False
print(json.dumps({'accepted':False,'manifestFiles':len(wanted),'manifestBytes':sum(v['bytes'] for v in manifest['files']),'sourceCommit':manifest['sourceCommit'],'uniquePassingFunctions':len(unique),'newFunctions':4,'directoryOpens':[68,30],'fullProvenanceChecks':[10,10],'syntheticMeanChangePercent':comparison['meanChangePercent'],'fileAuthenticationUnchanged':True,'previousNativeStopFailureStillEffective':True,'scope':'child directory acquisition optimization only; no new official service or fixed app acceptance'},indent=2))
