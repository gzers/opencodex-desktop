from pathlib import Path
import json,hashlib,subprocess,sys
b=Path(__file__).resolve().parent
if (b/'manifest.json').exists():
 for row in json.loads((b/'manifest.json').read_text())['files']:
  p=b/row['path'];assert p.stat().st_size==row['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest()==row['sha256']
functions=set();counts={}
for name in ['history-delivery-tests','retirement-delivery-tests','index-delivery-tests']:
 assert json.loads((b/(name+'.json')).read_text())['exitCode']==0
 log=(b/(name+'.log')).read_text();assert ' passed; 0 failed;' in log
 passed=set(line.split(' ... ',1)[0][5:] for line in log.splitlines() if line.startswith('test ') and ' ... ' in line and ' ... ignored' not in line);counts[name]=len(passed);functions.update(passed)
assert len(functions)==73 and counts=={'history-delivery-tests':14,'retirement-delivery-tests':37,'index-delivery-tests':36}
assert json.loads((b/'strict-clippy-delivery.json').read_text())['exitCode']==0
raw=json.loads(subprocess.check_output([sys.executable,str(b/'verify_raw_history.py')],text=True));assert raw==json.loads((b/'verified-raw-history.json').read_text())
summary=json.loads((b/'verification-summary.json').read_text());assert summary['uniqueAffectedFunctionsPassed']==73 and summary['historyCases']==14 and not summary['nextGenerationAndRepeatedProductStartAccepted']
assert json.loads((b/'cleanup.json').read_text())['ownedUnitTmpEmpty']
print(json.dumps({'accepted':False,'verified':True,'uniqueAffectedFunctions':73,'historyCases':14,'exactIndependentSourceAndRawClosureVerified':True,'historicalObservationCreatesNoStartOrMovementPermission':True,'currentRootStrictAndNewGenerationUnimplemented':True},indent=2))
