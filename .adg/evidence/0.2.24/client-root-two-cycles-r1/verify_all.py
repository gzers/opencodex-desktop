from pathlib import Path
import json,hashlib,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());expected=set()
for item in manifest['files']:
    p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in expected
    expected.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==expected
source=json.loads((base/'source-final.json').read_text());compiled=json.loads((base/'compiled-source.json').read_text())
assert source['files']==compiled['files'] and len(source['files'])==3 and source['binarySha256']==compiled['binarySha256'] and source['committedTestSource']==manifest['sourceCommit']
for name in ['compile-final','strict-clippy-final','release-build-final','release-command-regression','formatting']:
    assert json.loads((base/(name+'.json')).read_text())['exitCode']==0
assert json.loads((base/'compile-check.json').read_text())['exitCode']==101
assert '4 passed; 0 failed; 1 ignored' in (base/'release-command-regression.log').read_text()
for name in ['native-two-cycles','native-two-cycles-r2']:
    failure=json.loads((base/name/'preflight-failure.json').read_text());assert failure['accepted'] is False and failure['officialServiceStarted'] is False and failure['globalLWritten'] is False
verified=json.loads(subprocess.check_output([sys.executable,str(base/'verify_failed.py')],text=True));assert verified==json.loads((base/'verified-failure.json').read_text()) and verified['directedTwoCyclesPassed'] is False
sandbox=json.loads((base/'sandbox-preflight.json').read_text());assert sandbox['loopbackKernelBindPassed'] and sandbox['externalNetworkDenied'] and sandbox['dailyHomeDenied']
runner=json.loads((base/'native-two-cycles-r3/runner-summary.json').read_text());assert runner['binarySha256']==compiled['binarySha256']
cleanup=json.loads((base/'cleanup-final.json').read_text());assert cleanup['allOwnedTemporaryRootsRemoved'] and cleanup['archivedBeforeRemoval'] and cleanup['unitTmpEmptyBeforeRemoval']
assert cleanup['diagnosticPassed'] is False and cleanup['normalStopBudgetUnchanged'] and cleanup['actualNativeRounds']==1
print(json.dumps({'accepted':False,'manifestFiles':len(expected),'manifestBytes':sum(v['bytes'] for v in manifest['files']),
                  'sourceCommit':manifest['sourceCommit'],'actualNativeRounds':1,'nativeRoundResult':'failed-second-normal-Stop-timeout',
                  'firstCycleStartStatusStopMs':[8199,224,8782],'secondCycleStartStatusStopMs':[20700,2205,12878],
                  'rawFilesVerified':74,'ownedCoordinatorsRemoved':3,'globalLUnchanged':True,'testResidueRemoved':True,
                  'scope':'failure evidence for repeated official lifecycle; no retry, changed budget or fixed app acceptance'},indent=2))
