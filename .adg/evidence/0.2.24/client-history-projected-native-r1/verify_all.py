from pathlib import Path
import json,hashlib,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());expected=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in expected;expected.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==expected
source=json.loads((base/'compiled-source.json').read_text());assert len(source['files'])==15 and source['cleanSource'] and source['baseCommit']==manifest['sourceCommit']
runner=json.loads((base/'native-projected-r1/runner-summary.json').read_text());assert runner['binarySha256']==source['binarySha256'] and runner['exitCode']==101 and runner['diagnosticPassed'] is False
assert runner['globalLUnchanged'] and runner['trackedGenerationsAbsent'] and not runner['ownedExecutablesRemaining']
for label in ['release-build-final','release-regression']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
assert '4 passed; 0 failed; 1 ignored' in (base/'release-regression.log').read_text()
result=json.loads(subprocess.check_output([sys.executable,str(base/'verify_failed.py')],text=True));assert result==json.loads((base/'verified-failure.json').read_text()) and result['directedTwoCyclesPassed'] is False
assert result['secondCycle']['stopMs']==10736 and result['secondCycle']['revision']==16 and result['firstCycle']['stopMs']==8749
cleanup=json.loads((base/'cleanup-final.json').read_text());assert cleanup['allOwnedRootsRemoved'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['profileRawBytesComparedBeforeRemoval'] and cleanup['actualNativeRounds']==1 and cleanup['normalStopBudgetUnchanged']
print(json.dumps({'accepted':False,'manifestFiles':len(expected),'manifestBytes':sum(v['bytes'] for v in manifest['files']),'sourceCommit':manifest['sourceCommit'],'actualNewNativeRounds':1,'nativeResult':'failed-second-normalStop-recovery-pending','firstCycleStartStatusStopMs':[8202,227,8749],'secondCycleStartStatusStopMs':[14368,1453,10736],'rawProfileFilesVerified':74,'exactNewCoordinatorsRemoved':3,'globalLUnchanged':True,'previousNativeFailureUnchanged':True,'scope':'new hypothesis round retained as failure, no budget change or fixed application acceptance'},indent=2))
