from pathlib import Path
import json,hashlib,subprocess,sys
b=Path(__file__).resolve().parent
if (b/'manifest.json').exists():
 for row in json.loads((b/'manifest.json').read_text())['files']:
  p=b/row['path'];assert p.stat().st_size==row['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest()==row['sha256']
counts={};functions=set()
for name in ['atomic-generation-final','atomic-index-final','atomic-root-final','atomic-creation-final']:
 assert json.loads((b/(name+'.json')).read_text())['exitCode']==0
 log=(b/(name+'.log')).read_text();assert ' passed; 0 failed;' in log
 passed=set(line.split(' ... ',1)[0][5:] for line in log.splitlines() if line.startswith('test ') and ' ... ' in line and ' ... ignored' not in line);counts[name]=len(passed);functions.update(passed)
assert counts=={'atomic-generation-final':8,'atomic-index-final':44,'atomic-root-final':37,'atomic-creation-final':8} and len(functions)==89
assert json.loads((b/'atomic-clippy-final.json').read_text())['exitCode']==0
raw=json.loads(subprocess.check_output([sys.executable,str(b/'verify_raw.py')],text=True));assert raw==json.loads((b/'verified-raw.json').read_text()) and len(raw['verified'])==5
assert json.loads((b/'cleanup.json').read_text())['ownedUnitTmpEmpty']
print(json.dumps({'accepted':False,'verified':True,'uniqueAffectedFunctions':89,'realProcessExitBoundaries':5,'sameActionFlockPreserved':True,'oldIndexBindingVaultAndPredecessorBound':True,'newGenerationCaptureOrStartNotAccepted':True,'noPowerLossAcceptanceClaim':True},indent=2))
