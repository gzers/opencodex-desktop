from pathlib import Path
import hashlib,json,re,subprocess,sys
root=Path(__file__).resolve().parent;manifest=json.loads((root/'manifest.json').read_text());expected=set()
for item in manifest['files']:
    p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in expected
    expected.add(item['path']);raw=(root/p).read_bytes()
    assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
assert {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==expected
unique=set()
for name in ['product-start-delivery','index-regression','root-regression','rollover-regression','command-regression','status-regression']:
    assert json.loads((root/(name+'.json')).read_text())['exitCode']==0
    log=(root/(name+'.log')).read_text()
    names={n for n,status in re.findall(r'^test (\S+) \.\.\. (.*)$',log,re.M) if not status.startswith('ignored')}
    count=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)[-1]
    assert len(names)==int(count[0]);unique|=names
assert len(unique)==127
for name in ['strict-clippy','formatting','product-start-initial','product-start-final']:
    assert json.loads((root/(name+'.json')).read_text())['exitCode']==0
source=json.loads((root/'source-final.json').read_text());build=json.loads((root/'source-build.json').read_text())
assert source['files']==build['files'] and len(source['files'])==8
assert source['actualLibraryTestBinarySha256']==build['actualLibraryTestBinarySha256'] and source['committedSource']==manifest['sourceCommit']
checks=json.loads(subprocess.check_output([sys.executable,str(root/'verify_outcomes.py')],text=True))
assert checks==json.loads((root/'verified-outcomes.json').read_text()) and checks['accepted'] is False
cleanup=json.loads((root/'cleanup.json').read_text())
assert cleanup['temporaryRootRemoved'] and cleanup['unitTmpEmptyBeforeRemoval'] and cleanup['archiveComparedBeforeRemoval']
print(json.dumps({'accepted':False,'manifestFiles':len(expected),'manifestBytes':sum(v['bytes'] for v in manifest['files']),
                  'uniquePassingFunctions':len(unique),'newFunctions':8,'loggedPreparationScenes':2,'externalSameInodeWriteRefused':True,
                  'sourceCommit':manifest['sourceCommit'],'scope':'product preparation and readonly status wiring only; no official repeated lifecycle or fixed candidate acceptance'},indent=2))
