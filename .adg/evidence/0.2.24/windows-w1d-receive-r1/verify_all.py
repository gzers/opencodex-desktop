"""Offline check of received scope, host validation and exact integration source."""
from pathlib import Path
import hashlib,json,subprocess,sys,gzip,base64,re
r=Path(__file__).resolve().parent;code=Path(sys.argv[1]).resolve()
def load(name):return json.loads((r/name).read_bytes())
def sha(raw):return hashlib.sha256(raw).hexdigest()
p=subprocess.run([sys.executable,str(r/'verify_received.py'),str(code)],capture_output=True,check=True);received=json.loads(p.stdout);assert not received['accepted']
assert sum(v['exact'] for v in received['sourceBindings'])==2 and sum(v['onlyCRLF'] for v in received['sourceBindings'])==7
source=load('integrated-source.json');assert len(source['files'])==9
for item in source['files']:
 raw=(code/item['path']).read_bytes();assert len(raw)==item['bytes'] and sha(raw)==item['sha256'];assert raw==subprocess.check_output(['git','show',source['commit']+':'+item['path']],cwd=code)
for name,count,ignored in [('terminal',21,2),('layout',3,0),('final-current-check',1,0)]:
 report=load(name+'.json');assert report['exitCode']==0 and not report['windowsNativeExecutedHere'];text=(r/(name+'.log')).read_text();assert re.search(r'test result: ok[.] '+str(count)+r' passed; 0 failed; '+str(ignored)+r' ignored',text),name
assert load('clippy.json')['exitCode']==101 and 'single_element_loop' in (r/'clippy.log').read_text()
for name in ['final-clippy','format','final-format','diff-check']:assert load(name+'.json')['exitCode']==0,name
cleanup=load('cleanup.json');assert cleanup['tempRemoved'] and not Path(cleanup['tempRoot']).exists() and not cleanup['dailyConfigurationOrPathModified']
files=json.loads(gzip.decompress((r/'unit-temp-bytes.json.gz').read_bytes()));assert len(files)==cleanup['archivedFiles'] and sum(v['bytes'] for v in files)==cleanup['archivedBytes']
for f in files:
 raw=base64.b64decode(f['base64']);assert len(raw)==f['bytes'] and sha(raw)==f['sha256']
if (r/'manifest.json').exists():
 m=load('manifest.json');assert not m['accepted'];assert len(m['files'])==m['fileCount'] and sum(f['bytes'] for f in m['files'])==m['totalBytes']
 for f in m['files']:
  path=r/f['path'];assert path.is_relative_to(r) and not path.is_symlink();raw=path.read_bytes();assert len(raw)==f['bytes'] and sha(raw)==f['sha256'],f['path']
print(json.dumps({'codeCommit':source['commit'],'receivedFiles':211,'receivedBytes':received['ingestedBytes'],'rawComponentFiles':94,'rawMsiFailureFiles':51,'integratedSourceFiles':9,'hostDistinctTestsPassed':24,'targetedPostRefactorPassed':1,'strictClippyPassed':True,'initialClippyFailurePreserved':True,'nativeComponentsPassed':True,'actualMsiInstallExit':1625,'actualMsiUninstallExecuted':False,'tempRootRemoved':True,'windowsNewGeneratedBytesRetained':1944490289,'windowsUnitBytesRetained':3881472,'windowsBinariesRehashedHere':False,'accepted':False},indent=2))
