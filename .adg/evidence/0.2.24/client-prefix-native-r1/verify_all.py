from pathlib import Path
import json,hashlib,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());wanted=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in wanted;wanted.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==wanted
source=json.loads((base/'compiled-source.json').read_text());assert source['baseCommit']==manifest['sourceCommit'] and source['cleanSource'] and len(source['files'])==18
runner=json.loads((base/'native-prefix-r1/runner-summary.json').read_text());assert runner['binarySha256']==source['binarySha256'] and runner['exitCode']==101 and runner['diagnosticPassed'] is False
for label in ['release-build-final','release-regression']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
assert '4 passed; 0 failed; 1 ignored' in (base/'release-regression.log').read_text()
for script,record in [('verify_failed.py','verified-failure.json'),('verify_progress.py','verified-progress.json')]:assert json.loads(subprocess.check_output([sys.executable,str(base/script)],text=True))==json.loads((base/record).read_text())
progress=json.loads((base/'verified-progress.json').read_text());assert progress['revision']==17 and progress['registeredState']=='root_retirement_prepared' and progress['artifactEndBeforePlan']['code']=='ok' and int(progress['artifactEndBeforePlan']['remaining_after_us'])>0
cleanup=json.loads((base/'cleanup-final.json').read_text());assert cleanup['allOwnedRootsRemoved'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['rawProfileFiles']==77 and cleanup['normalStopBudgetUnchanged'] and cleanup['actualNativeRounds']==1
print(json.dumps({'accepted':False,'manifestFiles':len(wanted),'manifestBytes':sum(v['bytes'] for v in manifest['files']),'sourceCommit':manifest['sourceCommit'],'actualNewNativeRounds':1,'nativeResult':'failed-second-normalStop-root-retire-entry-deadline','firstCycleStartStatusStopMs':[8305,236,8285],'secondCycleStartStatusStopMs':[13465,1220,12143],'artifactRestoreCompletedWithinBudget':True,'registeredHead':'root_retirement_prepared17','rootMovedOrCompleted':False,'rawProfileFilesVerified':77,'exactNewCoordinatorsRemoved':3,'globalLUnchanged':True,'previousFailuresUnchanged':True,'scope':'new hypothesis round retained as failure, no timeout relaxation or fixed candidate acceptance'},indent=2))
