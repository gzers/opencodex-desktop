from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code');PREV=ROOT.parent/'wl2-r7-directory-read-20261118-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip();assert head.startswith('a22f6ebe8');remote=git('rev-parse','origin/codex/024-windows-native').decode().strip()
source=read(ROOT/'source-manifest.json');checks=[]
for row in source['files']:
    raw=(CODE/row['path']).read_bytes();blob=git('show',f'{head}:{row["path"]}')
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert raw.replace(bytes([13,10]),bytes([10]))==blob.replace(bytes([13,10]),bytes([10]))
    checks.append(dict(path=row['path'],normalizedGitBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(codeSha=head,observedRemoteBranch=remote,pushConfirmedAtSeal=head==remote,files=checks))
matrix=read(PREV/'status.json')['matrix'];matrix.extend([
 dict(item='pending original capture locator retention',status='relative/kind intent retained before actual originalnode nativeopen; samependingfile kept acrosscapturefailure, clearedonly after actualnode enrolled; nofile afterraw32 remainsIntent/no reopening evenconflictgone. old complete/mutation/abort-final gates now also reject pendinglocator'),
 dict(item='V2 actual tree full resource membership',status='every originalnode/nativework/pendingworkcreator and pendingcapture gets independent scopedobject; parent/custody separatelyretained. originalID/all8/bytes match originalsnapshot, actualpendingbytes mustbeoriginalprefix, missinginitialSDnull. Two complete nativefile/member passes and sourceparent/custody fences; late/foreign/SD/ADS/bytes/cutoffreject, no proof/Owner/claim factory'),
 dict(item='stable original tree V2 object keys',status='session+role+original/work/parent/custody+relative key, sortedobjectset; enumeration/nodeorder/pendingtoknown/laterworkdoesnotcompactoriginalIDs. actualoriginal/workIDdistinct; original logical locator stable aftercustodymove. DATA not persistedoldrecordmigration'),
 dict(item='tree membership actual new tests',status='5newnative tests16ownfixtures passed: actualpartialinvalidcoordinator,4working/empty/partial/fullpendingcreators,sort/move/workstablekeys,9nativeunknown/drift/cutoffrefusals,realnativeopenraw32 intentnoreopenafterconflictclose. Componentnativeonly/no realofficialOwnerSource qualification'),
 dict(item='tree membership first failure and regressions',status='product01raw101/55361ms testsrequiredDebugObjectE0277, no native; equalpredicate/expliciterrbranch fixed,noObjectAPIchange. product07raw0/161763ms4tests15scenes;product08raw0/160560msnewactualfailedopen1, productioncodeunchanged. oldtree5/v2native3/partialcapture3/pipeline1/Send1 =13tests allraw0, no W2'),
 dict(item='tree membership cleanup and delivery',status='32newsuccessfulroot receipts absent; oldpartialcapture originalDrop/noexactnewreceipt; no unexpectednativefailure or newfailedruntime. allpriorfailed/rejectedroots/synthetic27960 retained/nodeletionretry. Publicoutempty/session7disconnected; startfetch and firstpushTLSfailed, accurate delivery laterreceipt required'),
])
write('status.json',dict(codeSha=head,codeBranch='codex/024-windows-native',remoteCodeAtSeal=remote,codePushConfirmedAtSeal=head==remote,mainInput=source['mainInput'],docsInput=source['docsInput'],authority='TEAM33/IMP171/r7 lastconsumed,latestfetchTLSfailed/no unreadrevisionclaimed',scope='complete retained tree/work/pending V2 member readback, no full originalSource/partialrestore permission or actual fixedofficialOwner qualification',runs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')},artifact=read(ROOT/'artifact.json'),sourceFilesVerified=7,firstError=read(ROOT/'first-error-and-fixes.json'),uniqueNewTests=5,newActualNativeFixtureRoots=16,affectedRegressionTests=13,ordinaryDesktop='exact prior preparedPublicinput unchanged,out empty/session7disconnected; await original user entry',publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof'+chr(10),encoding='utf-8')
print(json.dumps(dict(codeSha=head,remote=remote,pushConfirmed=head==remote,matrix=len(matrix),sources=7)))
