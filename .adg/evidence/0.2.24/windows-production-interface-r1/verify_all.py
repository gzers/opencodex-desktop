"""Offline source/tests/cleanup checks; not Windows production acceptance."""
from pathlib import Path
import subprocess,json,hashlib,re,gzip,base64,sys
r=Path(__file__).resolve().parent;code=Path(sys.argv[1]).resolve()
def load(name):return json.loads((r/name).read_bytes())
def sha(raw):return hashlib.sha256(raw).hexdigest()
source=load('integrated-source.json');assert len(source['files'])==6 and not source['productionWindowsConsumersConnected']
for f in source['files']:
 raw=(code/f['path']).read_bytes();assert len(raw)==f['bytes'] and sha(raw)==f['sha256'];assert raw==subprocess.check_output(['git','show',source['commit']+':'+f['path']],cwd=code)
for name,count,ignored in [('launch-final',4,0),('transport-final',12,0),('owners',20,1)]:
 report=load(name+'.json');assert report['exitCode']==0 and not report['windowsNativeExecutedHere'];text=(r/(name+'.log')).read_text();assert re.search(r'test result: ok[.] '+str(count)+r' passed; 0 failed; '+str(ignored)+r' ignored',text),name
assert load('clippy.json')['exitCode']==101 and 'dead-code' in (r/'clippy.log').read_text()
assert load('clippy-complete.json')['exitCode']==0 and load('format-complete.json')['exitCode']==0
cleanup=load('cleanup.json');assert cleanup['removed'] and not Path(cleanup['tempRoot']).exists() and not cleanup['dailyConfigurationOrPathWritten']
files=json.loads(gzip.decompress((r/'unit-temp-bytes.json.gz').read_bytes()));assert len(files)==cleanup['filesArchived'] and sum(v['bytes'] for v in files)==cleanup['bytesArchived']
for f in files:
 raw=base64.b64decode(f['base64']);assert len(raw)==f['bytes'] and sha(raw)==f['sha256']
if (r/'manifest.json').exists():
 m=load('manifest.json');assert not m['accepted'] and len(m['files'])==m['fileCount'] and sum(v['bytes'] for v in m['files'])==m['totalBytes']
 for item in m['files']:
  p=r/item['path'];assert p.is_relative_to(r) and not p.is_symlink();raw=p.read_bytes();assert len(raw)==item['bytes'] and sha(raw)==item['sha256']
print(json.dumps({'interfaceCode':source['commit'],'sourceFiles':6,'hostDistinctPassed':36,'newTests':7,'originalIgnored':1,'strictClippy':True,'initialDeadCodeFailurePreserved':True,'finalFormat':True,'temporaryRootRemoved':True,'actualWindowsProductionConsumersConnected':False,'windowsNativeExecutedHere':False,'accepted':False},indent=2))
