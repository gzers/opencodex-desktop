from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
PREV=ROOT.parent/'wl2-r7-full-history-20261118-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip();assert head.startswith('1f940af6c')
assert git('rev-parse','origin/codex/024-windows-native').decode().strip()==head
sources=read(ROOT/'source-manifest.json');checks=[]
for row in sources['files']:
    raw=(CODE/row['path']).read_bytes();blob=git('show',f"{head}:{row['path']}")
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert raw.replace(b'\r\n',b'\n')==blob.replace(b'\r\n',b'\n')
    checks.append(dict(path=row['path'],normalizedGitBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(codeSha=head,files=checks))
old=read(PREV/'status.json');matrix=old['matrix']
matrix.extend([
 dict(item='TEAM31/IMP168 input consumption',status='main4ea82e8a61fe00c1f76334db5e39654f49f4342e/docs'+sources['docsConsumed']+' merged; Windows embedded signedinstaller verifier/workflows/tests only, noRustgraph/installer/nativeOwnerexclusivechange; no buildrelease or W2 action'),
 dict(item='canonical originalworker three-stage shared trait',status='WorkerLocal implements unchanged BeforeFirstResumeRecovery withoriginalOwner/private nonClone/nonserde Prepared/Quiescent/Restored locators; originalactualprefix/quiescentresources retainedinBoxattempt/Owner/adapter beforefalliblechecks, sameworkerthread/action/end, failedone-shotrevoke/park; canonicalcompletepath only, premanifestV2partial stillRequired'),
 dict(item='production recovery driver trait consumption',status='original callback now prepare_abort -> quiesce_launcher -> restore_originals -> actualretainednativeproofcheck, no sharedABI changes/newtimer; explicitlease release/nativefinalread remains originalWorkerLocal; fullproductgraphcompiled, realoriginalproductOwner/officialSource runtime notqualified'),
 dict(item='canonical actual claim borrow continuity',status='newactualnativefixture3borrowstages sameepoch/key and all4originalclaimNLKranges competitorraw33, no underlyingrelease between; explicitrelease then actual4exclusiveprobes+unlock pass, originalinactiveleaseclaim refused; no Ownerfabricated'),
 dict(item='canonical type and metadata boundary tests',status='2tests passed/reused: unchangedtrait associatedtypes/nonClone/nonserde andWorkerLocal!Send; crosssession/action/scope/end/thread/expiredmetadata refuse; these DATA guards not actualOwnerexecution'),
 dict(item='canonical trait first failure',status='product01raw101/168410ms newfixture asserted leaseOptionNone though originalreleasekeepsinactive drainedshell; originalnativebusy/epoch/release plusmetadata/types alreadypassed; onlytest changedto actualfree4ranges/oldclaimrefusal,4filefailuretree+readonlyZIP retained; product02raw0/167330ms'),
 dict(item='canonical trait affected regressions',status='12originaltestsraw0:worker3parents7actualsubcase/1735ms,concretepipeline1twoModes/11187ms,returned2/33594ms,origin5/63ms,Send1/46ms; not realofficialOwner or W2'),
 dict(item='canonical trait cleanup',status='20new successful ownroots absent;1new4filefailuretree/ZIP and allpriorfailed/rejected/synthetic27960 gaps retained; schemas rawarchived/trackedrestored/untrackedmoved,no deletionretry'),
])
write('status.json',dict(codeSha=head,codeBranch='codex/024-windows-native',mainBuilt=sources['mainBuilt'],mainConsumed=sources['mainConsumed'],mainOnlyUncompiledDelta=sources['mainOnlyUncompiledDelta'],docsConsumed=sources['docsConsumed'],authority='TEAM31/IMP168/r7 unchanged',scope='canonical originalworker sharedtrait adapter and productiondriver compile + actuallease/type/metadata tests; partialV2/fulltraitscope and realofficialOwner stillincomplete',runs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')},artifact=read(ROOT/'artifact.json'),sourceFilesVerified=5,firstError=read(ROOT/'first-error-and-fixes.json'),uniqueNewTests=3,newActualLeaseFixture=1,affectedRegressionTests=12,publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof\nregression02/subprocess/*/child.stdout.log whitespace=cr-at-eol,-blank-at-eof,-blank-at-eol\nreceived-raw-whitespace-check.log whitespace=cr-at-eol,-blank-at-eof,-blank-at-eol\n',encoding='utf-8')
print(json.dumps(dict(codeSha=head,matrix=len(matrix),sources=5)))
