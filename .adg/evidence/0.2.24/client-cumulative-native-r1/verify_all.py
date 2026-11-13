from pathlib import Path
import json,hashlib,gzip,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
source=json.loads((base/'compiled-source.json').read_text());assert source['cleanSource'] and len(source['files'])==21 and source['baseCommit']==manifest['sourceCommit']
assert source['binarySha256']=='1076a7ac010f1c66ef418d1539609049462677e1b7359cf4449f84025e0da95d'
assert json.loads((base/'release-build-final.json').read_text())['exitCode']==0
for script,record in [('verify_failed.py','verified-failure.json'),('verify_progress.py','verified-progress.json')]:
 result=json.loads(subprocess.check_output([sys.executable,str(base/script)],text=True));assert result==json.loads((base/record).read_text())
cleanup=json.loads((base/'cleanup-final.json').read_text());assert cleanup['archiveComparedBeforeRemoval'] and cleanup['temporaryNativeRootRemoved'] and cleanup['rawProfileFiles']==77 and cleanup['exactNewCoordinatorsRemoved']==3 and cleanup['globalLUnchangedAndReleased'] and cleanup['normalStopBudgetUnchanged'] and cleanup['actualNativeRounds']==1
progress=json.loads((base/'verified-progress.json').read_text());assert progress['productInternalStopMs']==10202 and progress['originalRootMovementNotStarted'] and progress['revision']==17
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'actualNativeRounds':1,'firstCycleStartStatusStopMs':[8405,233,8638],'secondCycleStartStatusStopMs':[13253,1164,10888],'retainedState':'RootRetirementPrepared17','rootMoved':False,'rawProfileFilesVerified':77,'allOwnedTemporaryRootsRemoved':True,'globalLUnchangedAndReleased':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
