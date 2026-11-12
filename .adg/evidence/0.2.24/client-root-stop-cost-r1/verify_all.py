from pathlib import Path
import json,subprocess,sys,hashlib
b=Path(__file__).resolve().parent
if (b/'manifest.json').exists():
 for row in json.loads((b/'manifest.json').read_text())['files']:
  p=b/row['path'];assert p.stat().st_size==row['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest()==row['sha256']
r=json.loads(subprocess.check_output([sys.executable,str(b/'verify_native.py')],text=True));assert r==json.loads((b/'verified-native.json').read_text())
functions=set()
for name in ['final-root-stop','final-root-move','final-originals']:
 assert json.loads((b/(name+'.json')).read_text())['exitCode']==0
 log=(b/(name+'.log')).read_text();assert ' passed; 0 failed;' in log
 functions.update(l.split(' ... ',1)[0][5:] for l in log.splitlines() if l.startswith('test ') and ' ... ' in l and ' ... ignored' not in l)
assert len(functions)==52
s=json.loads((b/'source-final.json').read_text());n=json.loads((b/'facts-coalesced/source.json').read_text());assert s['files']==sorted(n['files'],key=lambda r:r['path']) and s['actualLibraryTestBinarySha256']==n['binarySha256']
assert json.loads((b/'strict-clippy-facts.json').read_text())['exitCode']==0
trace=(b/'cost-diagnostic/native.stderr.log').read_text();assert 'label=root-authority phase=registration segment_us=124597' in trace and 'label=root-source phase=ready-stopped-frames segment_us=67493' in trace
trace=(b/'facts-coalesced/native.stderr.log').read_text();assert 'expected=released actual=released' in trace and 'expected=relocated actual=relocated' in trace
assert json.loads((b/'prearchive-cleanup.json').read_text())['allOriginalToolHandlesTerminal']
print(json.dumps({'accepted':False,'verified':True,'uniqueAffectedFunctionsPassed':52,'nativeFailedRounds':4,'finalCurrentSourceAndBinaryMatch':True,'finalDiskReadbackDoesNotCloseIndependentIndex':True,'originalStop10StillFailing':True},indent=2))
