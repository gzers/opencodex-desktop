from pathlib import Path
import hashlib,json,re,subprocess,sys
root=Path(__file__).resolve().parent
manifest=json.loads((root/'manifest.json').read_text());expected=set()
for item in manifest['files']:
    path=Path(item['path']);assert not path.is_absolute() and '..' not in path.parts and item['path'] not in expected
    expected.add(item['path']);raw=(root/path).read_bytes()
    assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
assert {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==expected
unique=set()
for name in ['rollover-final','index-regression','root-regression','first-originals-regression','creation-regression']:
    assert json.loads((root/(name+'.json')).read_text())['exitCode']==0
    log=(root/(name+'.log')).read_text()
    names={name for name,status in re.findall(r'^test (\S+) \.\.\. (.*)$',log,re.M) if not status.startswith('ignored')}
    summary=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)[-1]
    assert len(names)==int(summary[0]);unique|=names
assert len(unique)==118
for name in ['strict-clippy','formatting','initial-storage','rollover-initial']:
    assert json.loads((root/(name+'.json')).read_text())['exitCode']==0
build=json.loads((root/'source-build.json').read_text());final=json.loads((root/'source-final.json').read_text())
assert len(build['files'])==10 and build['files']==final['files']
assert final['actualLibraryTestBinarySha256']==build['actualLibraryTestBinarySha256']
assert final['committedSource']==manifest['sourceCommit']
raw=json.loads(subprocess.check_output([sys.executable,str(root/'verify_raw.py')],text=True))
assert raw==json.loads((root/'verified-raw.json').read_text())
assert len(raw['verified'])==4 and raw['accepted'] is False
cleanup=json.loads((root/'cleanup.json').read_text())
assert cleanup['temporaryRootRemoved'] and cleanup['unitTmpEmptyBeforeRemoval'] and cleanup['archiveComparedBeforeRemoval']
print(json.dumps({'accepted':False,'manifestFiles':len(expected),'manifestBytes':sum(v['bytes'] for v in manifest['files']),
                  'uniquePassingFunctions':len(unique),'verifiedStorageGenerations':4,'actualThirdGenerationExitScenes':3,
                  'sourceCommit':manifest['sourceCommit'],'scope':'multigeneration storage/index facts only; production Start/status and official repeated lifecycle not accepted'},indent=2))
