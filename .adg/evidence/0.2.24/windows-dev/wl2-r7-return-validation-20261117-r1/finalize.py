from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
PREV=ROOT.parent/'wl2-r7-v2-durability-20261117-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def git(*args):return subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip();assert head.startswith('361cdedf0')
assert git('rev-parse','origin/codex/024-windows-native').decode().strip()==head
sources=read(ROOT/'source-manifest.json')
checks=[]
for row in sources['files']:
    raw=(CODE/row['path']).read_bytes();blob=git('show',f"{head}:{row['path']}")
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert raw.replace(b'\r\n',b'\n')==blob.replace(b'\r\n',b'\n')
    checks.append(dict(path=row['path'],normalizedGitBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(codeSha=head,files=checks))
runs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
old=read(PREV/'status.json');matrix=old['matrix']
matrix.extend([
 dict(item='TEAM28/IMP161 latest input',status='main81f52452cb4cfafc64a80ca92d97a3ba5990c2fb merged and finalproduct03 compiled on same graph; docbf135bf249066f7fa306400bdd839893d747e425 consumed/merged; Windowsr7exclusivewrite set unchanged'),
 dict(item='original native returned core independent validation',status='6actual native modes valid/tree-member/journal-member/wrongcutoff/expired/foreignreleased passed; originalfinalimage bound to samecore/end; complete all8/bytes/children/source/canonical/attachment fences+released original objects, twice originaltree reread, no newrange/newcompletionrecord'),
 dict(item='original worker receiving-slot one-shot validation',status='4actual owningworker modes valid/raw5/wrongcutoff/expired passed; exactReturnedBoxOwner+originalwindow+ReleasedLease enrolled in control before check, same transferbinding before/after; failure retains/revokes, no retry/restart/Closed; productionOwner bridge onlycompiled/not actualofficialruntimeaccepted'),
 dict(item='return worker identity projection',status='originalworkerThread preserved; separate returnValidatorThread; ReturnedValidated/ReturnValidationFailedRetained stillrecoveryRequired/no runtimeAuthority, normaldispatch remainscontroloccupied'),
 dict(item='return validation first compile/native failures',status='product01 raw101E0624 testprivateentry/E0382consumedlease; cfgtestwrapper only and testdrop corrected. product02 core6passed but workerfixture protectedDACL refusal beforeworkercreation; private sourcedir corrected,4filefailedscene readonlyZIPverified/source retained'),
 dict(item='return validation affected original regressions',status='worker3parents7actualsubcases raw0/1485ms; release2tests10actualscenes raw0/36859ms; concretepipeline1test2actualmodes raw0/9516ms; Send1raw0/31ms.6ownfailedsubprocessraw242 isolatedfixture only; no currentr2'),
 dict(item='return validation cleanup',status='29new successful fixture roots absent across executed revisions/affectedregressions;1new4fileprivateDACLfailure root+archive retained; oldrejected/driverfailedroots+synthetic27960 retained,no deletionretry'),
 dict(item='returned slot remaining scope',status='internal production originalSlot admission/Owner+proof recheck compiled/linked; actual productOwner runtime, postexpiryfullhistoricalstatus, fulltrait/strictreachableprivatehost/fullpartialV2/absentroot stillincomplete; originalwindowvalidation notpostexpirypermission'),
])
write('status.json',dict(codeSha=head,codeBranch='codex/024-windows-native',mainInput=sources['mainInput'],docsInput=sources['docsInput'],authority='TEAM28/IMP161/r7 unchanged',scope='original-window returnedOwner/native finalproof independent readback; productionOwner path compiled only, componentnative acceptance separately bounded',runs=runs,artifact=read(ROOT/'artifact.json'),sourceFilesVerified=13,firstError=read(ROOT/'first-error-and-fixes.json'),uniqueNewNativeTests=2,uniqueNewNativeModes=10,affectedRegressionParentTests=7,productionAccepted=False,allAccepted=False,publicStartOpen=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof\nregression04/subprocess/*/child.stdout.log whitespace=cr-at-eol,-blank-at-eof,-blank-at-eol\n',encoding='utf-8')
print(json.dumps(dict(codeSha=head,matrix=len(matrix),sources=13)))
