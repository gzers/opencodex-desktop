from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code');PREV=ROOT.parent/'wl2-r7-private-wire-20261118-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip();code='b6db6d582cdabd4f87329c6548526fc40db6bc30'
subprocess.run(['git','merge-base','--is-ancestor',code,'HEAD'],cwd=CODE,check=True)
sources=read(ROOT/'source-manifest.json');checks=[]
for row in sources['files']:
    raw=(CODE/row['path']).read_bytes();blob=git('show',f'{head}:{row["path"]}')
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert raw.replace(bytes([13,10]),bytes([10]))==blob.replace(bytes([13,10]),bytes([10]))
    checks.append(dict(path=row['path'],normalizedGitBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(featureCodeSha=code,integratedCodeSha=head,files=checks))
matrix=read(PREV/'status.json')['matrix']
for row in matrix:
    if row['item']=='private V2 actual reachable transport gap':row['status']='original nativeV2server/client/locator/peer gate and internalocxd dispatch nowimplemented+fullMSVCactualocxdbuilt; endpointinstalled before firstqualificationcapture, originalRuntimeArc boundonce; realordinarydesktop endtoend Pending, preparedexactemptyfixture; actualofficialOwner restore stillunqualified'
matrix.extend([
 dict(item='TEAM33/IMP171 intake',status='main88b8fc33d/docs4a09aa011 consumedandmerged;18Mac/UI/smokepaths,noWindowsnativeoverlap;10WindowsSourceBytesunchanged. InitialTLS failures+resolutionpreserved; buildartifact remainsoriginalpremergeb6dbsource/snapshots notnewcandidate'),
 dict(item='original native V2 protected endpoint/client',status='NEWschema2locator/firstinstanceSIDpipe before ExistingQualification::prepare; originalruntimeArc registeredoncebeforeStart; eachrequest native ordinarytoken/session/exactimage/noOuterJob/locator/pins prepost fences, freshstrictV2hello/replay, no oldloaded357/V1cleanupadoption'),
 dict(item='native V2 canonical dispatch',status='Begin uses ONE originalQPCmax10 window -> originalregistryOwner canonicaltraitworker -> sameend returnedowner validation; diagnostics metadata separatefrom fullhistoricalStatus5; timeout/unknown/logerror/disconnect keep originalhost/worker/resources,no renewal/automaticcleanup'),
 dict(item='native V2 exact ocxd dispatch',status='debugWindowsinternalclient exactoperation,beginrequiresconfirm/readsrejectextraargs, inheritedprivatebootstrap inoriginal10/5; realocxdbinary7invalidargv combinations raw2 beforebootstrap/nativeowner'),
 dict(item='native V2 strict replies/ack',status='2newstrictDATA tests plus6existingV2wire allraw0/162460ms;replyrequest/scope/version/purpose/error consistency, requireallfalse releaseflags andexplicitnullableerror;raw5preserved. terminalACK expecteddisconnect corrected while originalnativepeer/locator/pins/actionprepostfencesremain'),
 dict(item='native V2 original product compile/artifacts',status='actualMSVCocxdbinary02raw0/138552ms,finalbinary04raw0/150315ms afternativepeerDATAlogging/terminalACK correction; librarytestsproduct03raw0, final binarysamestrictwiretypebytes;posttesttransportdelta notclaimedactualdesktopruntimeaccepted'),
 dict(item='native V2 first compile failure',status='product01raw101/62768ms outerbootstrap exposedprivateCommandV2 at3sites; native tests notexecuted; enumkeptprivate/stringconversionmovedinside recovery_control; toolconstructionerrors retainedin first-error-and-fixes,no desktopprocessstarted'),
 dict(item='native V2 affected old protocol regressions',status='oldfailurecontrol2/raw0/47ms,strictv1frame1/32ms,originalQPC1/31ms,bootstrap1/31ms;code unchanged/no oldr2pipeexecution'),
 dict(item='native V2 ordinary desktop prepared',status='marenbensession7 currentlyDisconnected; exactnewPublicinput4files verified/ACLscoped/outempty; PS51helper5actualchecks passed/rootabsent. Userordinarydesktop Win+R required, no Runtime/Owner/officialrun; successonly exactownemptyhostcreationhandle242separatefixturedisposition,failureretainsfiles/processes'),
 dict(item='native V2 cleanup',status='1ownPS51helperrootabsent,noendpointordinaryexecutionyet; preparedPublicinput/outputsheld; allpriorfailed/rejectedroots/synthetic27960 gaps unchanged,no delete retry'),
])
write('status.json',dict(codeSha=code,integratedCodeSha=head,codeBranch='codex/024-windows-native',mainBuilt=sources['mainInput'],mainConsumed='88b8fc33d5ec21b54c494cdb529fd68026b3f8b9',docsConsumed='4a09aa011c18d2880ed63bfb211694d49ed84669',authority='TEAM33/IMP171/r7 unchanged',scope='nativeV2original protectedserver/client/internaldispatch build+strictnegative tests; ordinarydesktop endtoend and realofficialOwner recovery stillPending',runs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')},artifacts=read(ROOT/'artifacts.json'),sourceFilesVerified=10,firstError=read(ROOT/'first-error-and-fixes.json'),newReplyDataTests=2,totalV2DataTests=8,oldProtocolRegressions=5,actualBinaryArgvNegativeCases=7,desktop='Pending explicit ordinary desktop run ofprepared exactemptyfixture',publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof'+chr(10),encoding='utf-8')
print(json.dumps(dict(featureCode=code,integratedCode=head,matrix=len(matrix),sources=10)))
