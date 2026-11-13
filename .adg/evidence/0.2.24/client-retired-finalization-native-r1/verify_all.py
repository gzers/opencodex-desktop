from pathlib import Path
import json,gzip,hashlib,subprocess,sys
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
s=json.loads((base/'compiled-source.json').read_text());assert s['baseCommit']==m['sourceCommit'] and s['cleanSource'] and len(s['files'])==32 and s['binarySha256']=='b22eb8b7e75ede414d6002fbd4486c9ccb55dd90f7f7d4d6372853a2b1929eb7'
assert json.loads((base/'release-build-final.json').read_text())['exitCode']==0
result=json.loads(subprocess.check_output([sys.executable,str(base/'verify_native.py')],text=True));assert result==json.loads((base/'verified-native.json').read_text())
out=base/'native-retired-r1';runner=json.loads((out/'runner-summary.json').read_text());assert runner['exitCode']==0 and runner['diagnosticPassed'] and runner['exactOneTestExecuted'] and runner['retiredCompletionMarker'] and runner['binarySha256']==s['binarySha256']
summary=json.loads((out/'native/summary.json').read_text());assert summary['retiredFinalizeElapsedMs']==2694 and summary['retiredDuplicateElapsedMs']==2691 and summary['retiredPostImageDriftRefused'] is True
assert summary['stop']['result'] is None and summary['stop']['failure']['code']=='client_original_restore_pending' and summary['retiredPendingBeforeFinalize']['revision']==8
c=json.loads((base/'cleanup-final.json').read_text());assert c['actualNativeRounds']==1 and c['directedExplicitFinalizePassed'] and not c['originalNormalStopPass'] and c['rawProfileFiles']==51 and c['archiveComparedBeforeRemoval'] and c['temporaryNativeRootRemoved'] and c['exactNewCoordinatorsRemoved']==3 and c['globalLUnchangedAndReleased'] and c['eachRecoveryBudgetUnchanged']==10000
print(json.dumps({'accepted':False,'sourceCommit':m['sourceCommit'],'actualNativeRounds':1,'directedExplicitFinalizePassed':True,'explicitFinalizeMs':2694,'exactDuplicateMs':2691,'newClosedRevision':9,'sameConfirmationNoAdditionalRevision':True,'trackedSourceByteSnapshotsEqual':7,'postImageCurrentGrokDriftRefusedAndPreserved':True,'originalStopRemainsFailed':True,'rawProfileFilesVerified':51,'allOwnedTemporaryRootsRemoved':True,'priorNormalTwoCycleEvidenceUnchanged':True,'fixedAppAndWholeVersionAccepted':False,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in m['files'])},indent=2))
