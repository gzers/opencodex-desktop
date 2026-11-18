from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code');PREV=ROOT.parent/'wl2-r7-preparation-read-20261118-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip();assert head.startswith('b1ef52972')
remote=git('rev-parse','origin/codex/024-windows-native').decode().strip()
source=read(ROOT/'source-manifest.json');checks=[]
for row in source['files']:
    raw=(CODE/row['path']).read_bytes();blob=git('show',f'{head}:{row["path"]}')
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert raw.replace(bytes([13,10]),bytes([10]))==blob.replace(bytes([13,10]),bytes([10]))
    checks.append(dict(path=row['path'],normalizedGitBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(codeSha=head,observedRemoteBranch=remote,pushConfirmedAtSeal=head==remote,files=checks))
matrix=read(PREV/'status.json')['matrix'];matrix.extend([
 dict(item='actual native directory preparation borrow',status='original PrivateDirectoryPreparation exposes readonly borrowed originalpath/parent/terminal/complete/transferred facts under caller end; no newnativeopen/clone/rights/serde dependency inbackend. complete transfer trackedonly afteractualownedPin take, productioncreation sequence/budgets unchanged'),
 dict(item='native directory preparation readback',status='originalterminal-held branch only compares nativeprivate/directoryID and querypathsameID/all8/ADS/twofullmemberreads beforeafter, untransferred directory muststillEmpty; no terminal branch never reopens/enumerates target or infersAbsent/identity. OriginalmissinginitialSD remainssecurityDigestnull/Pending; transferred resources consumedelsewhere notreadopted'),
 dict(item='stable partial Source/custody members',status='Source/contextroles reserve0..3,pins16+;custodycodex0/grok1/directoryprefix16+ independentofsubset; actualgrokslot/prefixIDstayedstablewhencodexarrived. directoryoriginalcreated/retained identity/content DATA projected withallmissingbaselinePending, no restorefactory'),
 dict(item='directory native new tests',status='5newactualnative tests13ownfixtures passed:5creationfaultprefixes;created-unopenedpathreplacement nonadoption;transferred/existing/expiredbranch;5member/security/ADS/latechange negatives;stablecustodymapping. No originalproductOwner/Source or fullpartialrestore acceptance'),
 dict(item='directory native build first errors',status='product01raw101/59954ms missingnativealiasE0433,product03raw101/58659ms unsizedPathE0277;compileonly originalsretained. Callsite import/reference fixed,noAPIwidening. backend02raw0/751ms finalnativesourceidentical; product08fullTauriMSVCraw0/154344ms5tests, no unexpectednativefailure'),
 dict(item='directory affected regressions and cleanup',status='oldnativeDirectory4raw0/31ms,V2native3/94ms,actualjournalSlot1tenstates/1250ms,Send1/16ms =9affectedtests;34successful ownroots absent,no new failednativefixtures. Allpreviousfailed/rejectedroots/synthetic27960 retained/no deletionretry;schema rawarchived/restored/moved;desktopoutstillEmpty/session7disconnected'),
])
write('status.json',dict(codeSha=head,codeBranch='codex/024-windows-native',remoteCodeAtSeal=remote,codePushConfirmedAtSeal=head==remote,mainInput=source['mainInput'],docsInput=source['docsInput'],authority='TEAM33/IMP171/r7 lastconsumed,latestfetchTLSfailed/no unreadrevisionclaimed',scope='read actual original directorypreparation resources and fixed stableDATA slots; missing initialSD/Source/completepartialrestoration staysPending',runs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')},artifact=read(ROOT/'artifact.json'),sourceFilesVerified=6,firstError=read(ROOT/'first-error-and-fixes.json'),uniqueNewTests=5,newActualNativeFixtureRoots=13,affectedRegressionTests=9,ordinaryDesktop='previous exactPublicinput unchanged,out empty; userordinarydesktop result Pending',publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof'+chr(10),encoding='utf-8')
print(json.dumps(dict(codeSha=head,remote=remote,pushConfirmed=head==remote,matrix=len(matrix),sources=6)))
