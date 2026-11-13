from pathlib import Path
import json,hashlib,subprocess,sys
b=Path(__file__).resolve().parent
if (b/'manifest.json').exists():
 for row in json.loads((b/'manifest.json').read_text())['files']:
  p=b/row['path'];assert p.stat().st_size==row['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest()==row['sha256']
v=json.loads(subprocess.check_output([sys.executable,str(b/'verify_native.py')],text=True));assert v==json.loads((b/'verified-native.json').read_text())
functions=set()
for name in ['descriptor-final-tests','descriptor-final-index','canonical-reopen-regression']:
 r=json.loads((b/(name+'.json')).read_text());assert r['exitCode']==0
 log=(b/(name+'.log')).read_text();assert ' passed; 0 failed;' in log
 functions.update(l.split(' ... ',1)[0][5:] for l in log.splitlines() if l.startswith('test ') and ' ... ' in l and ' ... ignored' not in l)
assert len(functions)==63
assert json.loads((b/'descriptor-delivery-exact.json').read_text())['exitCode']==0 and json.loads((b/'strict-clippy-delivery.json').read_text())['exitCode']==0
final=json.loads((b/'source-final.json').read_text());n=json.loads((b/'descriptor/source.json').read_text());different=[r['path'] for r in final['files'] if r['sha256']!=next(i['sha256'] for i in n['files'] if i['path']==r['path'])];assert different==['apps/desktop/tauri/src/infrastructure/macos_client_originals_tests.rs']
assert final['lastNativeBinarySha256']==n['binarySha256']
assert json.loads((b/'prearchive-cleanup.json').read_text())['allOriginalToolHandlesTerminal'] and json.loads((b/'descriptor-prearchive-cleanup.json').read_text())['allOriginalToolHandlesTerminal']
print(json.dumps({'accepted':False,'recordedEvidenceVerified':True,'directedUniquePassingFunctions':63,'nativeFailingRounds':4,'firstStartAndRealExitOnly':True,'Stop10StillFailing':True,'nextStartStillUnaccepted':True},indent=2))
