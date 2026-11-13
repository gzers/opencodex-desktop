"""Verify frozen source, received evidence, bounded host tests and cleanup."""
from pathlib import Path
import hashlib,json,subprocess,sys,gzip,base64,re
r=Path(__file__).resolve().parent;code=Path(sys.argv[1]).resolve()
def load(name):return json.loads((r/name).read_bytes())
def sha(raw):return hashlib.sha256(raw).hexdigest()
p=subprocess.run([sys.executable,str(r/'verify_received.py'),str(code)],capture_output=True,check=True)
received=json.loads(p.stdout);assert received['accepted'] is False
source=load('integrated-source.json');assert len(source['files'])==4
for f in source['files']:
 raw=(code/f['path']).read_bytes();assert len(raw)==f['bytes'] and sha(raw)==f['sha256'];assert raw==subprocess.check_output(['git','show',source['commit']+':'+f['path']],cwd=code)
for name,passed,ignored in [('selection',13,0),('terminal',21,2),('native-shim',2,0)]:
 v=load(name+'.json');assert v['exitCode']==0 and not v['windowsNativeExecutedHere'];text=(r/(name+'.log')).read_text();assert re.search(r'test result: ok[.] '+str(passed)+r' passed; 0 failed; '+str(ignored)+r' ignored',text),name
for name in ['clippy','format','diff-check','shim-msvc-typecheck']:assert load(name+'.json')['exitCode']==0,name
cleanup=load('cleanup.json');assert cleanup['removed'] and not Path(cleanup['temporaryRoot']).exists() and not cleanup['dailyConfigurationOrPathChanged']
files=json.loads(gzip.decompress((r/'unit-temp-bytes.json.gz').read_bytes()));assert len(files)==cleanup['archivedFiles'] and sum(f['bytes'] for f in files)==cleanup['archivedBytes']
for f in files:
 raw=base64.b64decode(f['base64']);assert len(raw)==f['bytes'] and sha(raw)==f['sha256']
if (r/'manifest.json').exists():
 m=load('manifest.json');assert not m['accepted'] and len(m['files'])==m['fileCount'] and sum(v['bytes'] for v in m['files'])==m['totalBytes']
 for f in m['files']:
  path=r/f['path'];assert path.is_relative_to(r) and not path.is_symlink();raw=path.read_bytes();assert len(raw)==f['bytes'] and sha(raw)==f['sha256'],f['path']
print(json.dumps({'integratedCode':source['commit'],'receivedFiles':received['receivedFiles'],'rawNativeFiles':139,'hostPassed':36,'newHostRegressionTests':4,'originalIgnored':2,'strictHostClippy':True,'shimWindowsMetadataOnly':True,'nativeShimProductSwitchDone':False,'powershellAcceptance':False,'officialWindowsCommandsPassed':False,'fullMsiUninstallPassed':False,'temporaryRootRemoved':True,'windowsGeneratedBytesRetained':26869205653,'accepted':False},indent=2))
