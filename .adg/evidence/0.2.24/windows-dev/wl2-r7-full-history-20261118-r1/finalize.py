from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
PREV=ROOT.parent/'wl2-r7-history-primitives-20261118-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip();assert head.startswith('683a69fa2')
assert git('rev-parse','origin/codex/024-windows-native').decode().strip()==head
sources=read(ROOT/'source-manifest.json');checks=[]
for row in sources['files']:
    raw=(CODE/row['path']).read_bytes();blob=git('show',f"{head}:{row['path']}")
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert raw.replace(b'\r\n',b'\n')==blob.replace(b'\r\n',b'\n')
    checks.append(dict(path=row['path'],normalizedGitBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(codeSha=head,files=checks))
old=read(PREV/'status.json');matrix=old['matrix']
for row in matrix:
    if row['item']=='complete original product historicalStatus gap':
        row['status']='internal canonical originalSlot/nativeOwner+Source/core/twoTrees/canonical/route/attachment/finalproof history nowimplemented and actualproductgraphcompiled; componentnative10modes+QPCpassed; real productOwner/official Source runtime and reachableversionedhost remain unqualified/incomplete'
    if row['item']=='returned slot remaining scope':
        row['status']='internal slot returnverification and original-window/freshStatus5 fullhistoricalread bridges implemented+compiled; actual original productOwner qualification/fulltrait/reachableprivatehost/fullpartialV2/absentroot still incomplete'
matrix.extend([
 dict(item='TEAM30/IMP166 input consumption',status='mainfb755b8ea63d8a3fc9146dc4db491931919905b0/docs'+sources['docsConsumed']+' consumed+merged; Macupdater .github/tests only/noRustproductgraph change from built0675; Windowsr7exclusivewrite set/sharedtraits unchanged, no release task'),
 dict(item='original private final readback timely history',status='actualnative completed_at retained in originalcore+privatefinaltag, sameoriginalend required; history full both-tree/retiredmembers/8/bytes/children and originalReleasedLease/history facts beforeafter; no newranges/secondrecord/nativefactory'),
 dict(item='full original Owner canonical history production bridge',status='original registry runtimeArc/root/H/Profile/hostUUID; actual localfailedStart/recoveryaction/oldwindow/QPCfixedmetadata/nativeJob242; originalRoute/Source/current/fullimage/canonicalrawreplay/nativeaction/strictproducerinitial-last-final read beforeafter inONE callerread cutoff<=Status5; actualfullTaurigraphcompiled only, realproductOwner notruntimequalified'),
 dict(item='original receiving history gate',status='requires original returnedBoxOwner/window/releasedobjects plus actual successfulvalidation completed_at<oldend; binding beforeafter, expired/oversizedread reject without actionrenewal; unknowninsidequery revokesandretains; ordinarydispatch/runtimeAuthority/Closed never granted'),
 dict(item='full history actual native component tests',status='2nativecomponenttests10modes plus1nativeQPCtest passed; core validafteroldexpiry/tree-member/wrongoriginal/expiredreader, worker validafterexpiry/unvalidated/bindingchange/raw5/expiredreader/6secondreader; syntheticcomponent Source/Owner is not realofficialproduct acceptance'),
 dict(item='full history first failure',status='product01raw101/184916ms fullcore4+QPCpassed, receiving valid/unvalidated refusal correct but ownfixtureclose raw32 returnedreceiver still retained originals; exact5remainingfiles readonlyZIP retained; onlyfixture explicit joinedworker/dropreceiver/validation/state order fixed, finalproduct02raw0/160322ms'),
 dict(item='full history affected regressions',status='20original tests allraw0: returnvalidation2, worker3parents7subcases, release2, concretepipeline1, origin5, strictV1data4, Send1, QPC2; originalraw242 only own6failedsubprocessdisposal; no currentr2/official/W2'),
 dict(item='full history cleanup',status='40successful ownfixture roots absent; new5filefailure-afterpartialclose ZIP/root retained; all priorfailed/rejected roots/synthetic27960 gaps retained/no delete retry; schemas rawarchived/trackedrestored/untrackedmoved'),
])
write('status.json',dict(codeSha=head,codeBranch='codex/024-windows-native',mainBuilt=sources['mainBuilt'],mainConsumed=sources['mainConsumed'],mainOnlyUncompiledDelta=sources['mainOnlyUncompiledDelta'],docsConsumed=sources['docsConsumed'],authority='TEAM30/IMP166/r7 unchanged',scope='internal canonical full native historical read production wiring + native component tests; real original productOwner/official qualification not executed',runs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')},artifact=read(ROOT/'artifact.json'),sourceFilesVerified=19,firstError=read(ROOT/'first-error-and-fixes.json'),uniqueNewTests=3,uniqueNewNativeModes=10,newNativeQpcBoundary=1,affectedRegressionTests=20,publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof\nregression04/subprocess/*/child.stdout.log whitespace=cr-at-eol,-blank-at-eof,-blank-at-eol\n',encoding='utf-8')
print(json.dumps(dict(codeSha=head,matrix=len(matrix),sources=19)))
