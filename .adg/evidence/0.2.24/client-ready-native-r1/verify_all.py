from pathlib import Path
import json,hashlib,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());wanted=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in wanted;wanted.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==wanted
source=json.loads((base/'compiled-source.json').read_text());assert source['baseCommit']==manifest['sourceCommit'] and source['cleanSource'] and len(source['files'])==18
runner=json.loads((base/'native-ready-r1/runner-summary.json').read_text());assert runner['binarySha256']==source['binarySha256'] and runner['exitCode']==101 and runner['diagnosticPassed'] is False
for label in ['release-build-final','release-regression']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
assert '4 passed; 0 failed; 1 ignored' in (base/'release-regression.log').read_text()
failure=json.loads(subprocess.check_output([sys.executable,str(base/'verify_failed.py')],text=True));assert failure==json.loads((base/'verified-failure.json').read_text()) and failure['directedTwoCyclesPassed'] is False
trace=json.loads(subprocess.check_output([sys.executable,str(base/'verify_trace.py')],text=True));assert trace==json.loads((base/'verified-trace.json').read_text())
assert trace['underlyingRefusal']=='lifecycle_timeout' and trace['lastLiveCheckCrossingDeadline']['expired_after']=='true' and trace['artifactApply']['remaining_after_us']=='0'
cleanup=json.loads((base/'cleanup-final.json').read_text());assert cleanup['allOwnedRootsRemoved'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['profileRawBytesComparedBeforeRemoval'] and cleanup['rawProfileFiles']==75 and cleanup['actualNativeRounds']==1 and cleanup['normalStopBudgetUnchanged']
print(json.dumps({'accepted':False,'manifestFiles':len(wanted),'manifestBytes':sum(v['bytes'] for v in manifest['files']),'sourceCommit':manifest['sourceCommit'],'actualNewNativeRounds':1,'nativeResult':'failed-second-normalStop-original-deadline-exhausted','firstCycleStartStatusStopMs':[8839,226,8608],'secondCycleStartStatusStopMs':[14110,1528,10729],'provenUnderlyingError':'lifecycle_timeout','remainingAtArtifactApplyUs':696169,'rawProfileFilesVerified':75,'exactNewCoordinatorsRemoved':3,'globalLUnchanged':True,'previousFailureEvidenceUnchanged':True,'scope':'diagnostic cause established for this failed release-library round; no budget change or fixed app acceptance'},indent=2))
