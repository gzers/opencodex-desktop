"""Offline evidence verifier; never runs native Windows code or edits registry."""
from pathlib import Path
import base64,gzip,hashlib,json,re,subprocess,sys
here=Path(__file__).resolve().parent
code=Path(sys.argv[1]).resolve()
def load(name):return json.loads((here/name).read_bytes())
def sha(raw):return hashlib.sha256(raw).hexdigest()
source=load('integrated-source.json')
assert source['changedFileCount']==len(source['files'])==8
for f in source['files']:
    raw=(code/f['path']).read_bytes();assert len(raw)==f['bytes'] and sha(raw)==f['sha256']
    assert raw==subprocess.check_output(['git','show',source['commit']+':'+f['path']],cwd=code)
for name in ['terminal-final','backend-uninstall','console-client','clippy','format-final','ui-terminal','ui-typecheck','diff-check']:
    r=load(name+'.json');assert r['exitCode']==0 and r['windowsNativeExecuted'] is False,name
assert load('format.json')['exitCode']==1
for name,passed,ignored in [('terminal-final',21,2),('backend-uninstall',13,0),('console-client',2,0)]:
    text=(here/(name+'.log')).read_text()
    assert re.search(r'test result: ok[.] '+str(passed)+r' passed; 0 failed; '+str(ignored)+r' ignored',text),name
text=(here/'ui-terminal.log').read_text();assert re.search(r'Tests\s+11 passed',text)
cleanup=load('cleanup.json');assert cleanup['temporaryRootRemoved'] and not Path(cleanup['ownedTemporaryRoot']).exists()
files=json.loads(gzip.decompress((here/'compiler-cache-bytes.json.gz').read_bytes()))
assert len(files)==cleanup['archivedFiles']==289
assert sum(f['bytes'] for f in files)==cleanup['archivedBytes']==4453578
for f in files:
    raw=base64.b64decode(f['base64']);assert len(raw)==f['bytes'] and sha(raw)==f['sha256']
assert cleanup['dailyPathOrConfigurationChanged'] is False
manifest=here/'manifest.json'
if manifest.exists():
    m=load('manifest.json');assert m['accepted'] is False
    assert len(m['files'])==m['fileCount']
    assert sum(v['bytes'] for v in m['files'])==m['totalBytes']
    for item in m['files']:
        path=here/item['path'];assert path.is_relative_to(here) and not path.is_symlink()
        raw=path.read_bytes();assert len(raw)==item['bytes'] and sha(raw)==item['sha256'],item['path']
print(json.dumps({'sourceCommit':source['commit'],'sourceFiles':8,'backendDistinctPassed':36,'newTestsPassed':13,'frontendPassed':11,'originalIgnored':2,'strictClippyPassed':True,'formatFailureRetained':True,'finalFormatPassed':True,'temporaryRootRemoved':True,'cacheFilesArchived':289,'windowsProductCompiledHere':False,'windowsNativeAcceptance':False,'accepted':False},indent=2))
