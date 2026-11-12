from pathlib import Path
import json,hashlib,subprocess,sys
b=Path(__file__).resolve().parent
if (b/'manifest.json').exists():
 for r in json.loads((b/'manifest.json').read_text())['files']:
  p=b/r['path'];assert p.stat().st_size==r['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest()==r['sha256']
functions=set()
for name in ['final-image-tests','selection-regression','repair-regression','removal-regression']:
 assert json.loads((b/(name+'.json')).read_text())['exitCode']==0
 log=(b/(name+'.log')).read_text();assert ' passed; 0 failed;' in log
 functions.update(l.split(' ... ',1)[0][5:] for l in log.splitlines() if l.startswith('test ') and ' ... ' in l and ' ... ignored' not in l)
assert len(functions)==33
for name in ['release-image-tests','release-selection-tests','original-release-image-negative','original-release-selection-negative']:
 assert json.loads((b/(name+'.json')).read_text())['exitCode']==0 and ' passed; 0 failed;' in (b/(name+'.log')).read_text()
assert json.loads((b/'strict-clippy-final.json').read_text())['exitCode']==0
base=json.loads((b/'baseline/image-scan-baseline.json').read_text());assert base['entries']==15646 and base['fileBytes']==645549518 and base['treeSha256']=='a29dedc203311ca4d6d4005db94b4a10d4fb84409283669e25272f080756b790'
source=json.loads((b/'source-final.json').read_text())
for label,reverse in [('final-comparison',False),('reverse-comparison',True)]:
 report=json.loads((b/label/'image-scan-comparison.json').read_text());assert report['rowsExactlyEqual'] and report['entries']==15646 and report['fileCount']==13393 and report['treeSha256']==base['treeSha256'] and report['parallelFirst']==reverse
 assert report['parallelMetrics']['stages']['file-read-hash'][0]==13393
 runner=json.loads((b/label/'runner-summary.json').read_text());assert runner['exitCode']==0 and not runner['ownedProcessesRemaining'] and runner['sourceFiles']==source['files']
native=json.loads(subprocess.check_output([sys.executable,str(b/'verify_native.py')],text=True));assert native==json.loads((b/'verified-native.json').read_text())
last=json.loads((b/'native-stop-release-final/source.json').read_text());assert last['baseCommit']==source['commit'] and last['files']==[] and last['binarySha256']==source['releaseLibraryBinarySha256']
print(json.dumps({'accepted':False,'verified':True,'debugUniqueTests':33,'releaseNegativeTests':21,'serialParallelRowsAndAllByteDigestsEqual':True,'cacheOrderLimitationRetained':True,'releaseFirstRoot30_5_10PassedTwice':True,'debugStopFailureRetained':True,'nextStartAndFullVersionNotAccepted':True},indent=2))
