from pathlib import Path
import json,hashlib,subprocess,sys
b=Path(__file__).resolve().parent
if (b/'manifest.json').exists():
 for row in json.loads((b/'manifest.json').read_text())['files']:
  p=b/row['path'];assert p.stat().st_size==row['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest()==row['sha256']
r=json.loads(subprocess.check_output([sys.executable,str(b/'verify_native.py')],text=True));assert r==json.loads((b/'verified-native.json').read_text())
functions=set()
for name in ['accepted-prefix-tests','accepted-index-tests','accepted-root-tests']:
 assert json.loads((b/(name+'.json')).read_text())['exitCode']==0
 log=(b/(name+'.log')).read_text();assert ' passed; 0 failed;' in log
 functions.update(l.split(' ... ',1)[0][5:] for l in log.splitlines() if l.startswith('test ') and ' ... ' in l and ' ... ignored' not in l)
assert len(functions)==87
s=json.loads((b/'source-final.json').read_text());native=json.loads((b/'original-context/source.json').read_text());assert s['files']==sorted(native['files'],key=lambda r:r['path']) and s['actualLibraryBinarySha256']==native['binarySha256'] and s['productionOriginalsSourceUnchanged']
assert json.loads((b/'strict-clippy-accepted.json').read_text())['exitCode']==0
assert json.loads((b/'experiment-disposition.json').read_text())['disposition']=='rejected, not eligible acceptance source'
assert json.loads((b/'cleanup-audit.json').read_text())['auditLimitationExplicit']
print(json.dumps({'accepted':False,'verified':True,'finalSourcePassingFunctions':87,'allFourStopFailuresRetained':True,'perFrameChecksAndCompleteHelperAdmissionKept':True,'rejectedExperimentExcluded':True,'cleanupManifestLimitationExplicit':True,'completeVersionOrStopAccepted':False},indent=2))
