from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code');PREV=ROOT.parent/'wl2-r7-private-native-20261118-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip();assert head.startswith('96396f19b') and git('rev-parse','origin/codex/024-windows-native').decode().strip()==head
source=read(ROOT/'source-manifest.json');checks=[]
for row in source['files']:
    raw=(CODE/row['path']).read_bytes();blob=git('show',f'{head}:{row["path"]}')
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert raw.replace(bytes([13,10]),bytes([10]))==blob.replace(bytes([13,10]),bytes([10]))
    checks.append(dict(path=row['path'],normalizedGitBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(codeSha=head,remoteBranchSha=head,files=checks))
matrix=read(PREV/'status.json')['matrix'];matrix.extend([
 dict(item='original JournalPreparation native partial reads',status='same retained CREATE_NEW handles read actualempty/partial/full manifest+current originalexpectedprefix, ID/path/nativeprivate/all8/ADS twice perfile+two completefilepasses/fullmembers/dirSD/pins/peer beforeafter; succeedsafteroldStart cutoff onlyboundedreadDATA, neverinitializes/writes/clearsfailure'),
 dict(item='original journal partial baseline limitation',status='alreadyrecorded SD must equal original retainedbaseline; pendingopenwith nobaseline leaves securityDigestnull even if currentSDread succeeds. Aggregate peer/pins/dir metadata stillPending, full originalSource/permission/allcoverage not supplied; no restoreterminal/Closed fromDATA'),
 dict(item='stable V2 journal actual object mapping',status='manifest/current/aggregate originalDATA slots fixed0/1/2 independentof subset, path+nativeID/content digest currentread, originalbaselineknownonly. actualcomplete retainedJournal CurrentPublished; manifestRecord JournalPublished; rawmanifest/current/pins HandlesRetained; Sourceunknown remainsPending'),
 dict(item='native partial journal four tests',status='final4tests30distinctscenarios passed:10actualinitializerprefixes,9nativeunknown/drift/cutoffrefusals incllatechange,1oldexpiredprefixread/no renewedmutation,10actualCaptureSlot+PartialCaptureAnchor mapping states. ComponentDATA session/host, not real officialproductOwner acceptance'),
 dict(item='preparation read failures and regressions',status='prepare extra brace raw1 source kept; product01raw101E0599SnapshotData noClone, fixedbyoriginalborrow/noAPIwidening; native no unexpectedfailure. supplementalcollectornewlineSyntaxError kept;10affectedoldtests passed:journal3/v2native3/partialcapture3/Send1, no unrelatedreruns'),
 dict(item='preparation read cleanup and original desktop wait',status='55exactnew successfulroot receipts acrossexecutedrevisions/regressions absent; oldpartialregression originalDrop only/no fullhistoricalclaim; alloldfailed/rejectedroots/synthetic27960 retained/no deletionretry. PublicdesktopoutstillEmpty/session7Disconnected; no rerun or oldr2 action; networkTLSfailuresprepushheldthen code96396fpushsucceeded'),
])
runs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
write('status.json',dict(codeSha=head,codeBranch='codex/024-windows-native',mainInput=source['mainInput'],docsInput=source['docsInput'],authority='TEAM33/IMP171/r7 unchanged, latest fetch attempted but TLS failed; no unseen revision consumed',scope='original failed journal nativeprefix read and actual v2 premanifest membership projection; fullSource/nativepermission/partial restoration stillPending',runs=runs,artifact=read(ROOT/'artifact.json'),sourceFilesVerified=6,firstError=read(ROOT/'first-error-and-fixes.json'),collectorError='collector-error.json + original unexecuted collector source',uniqueNewTests=4,uniqueNewNativeScenarios=30,affectedRegressionTests=10,ordinaryDesktop='previous exactPublicinput unchanged, out empty; userordinarydesktop result stillPending',publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof'+chr(10),encoding='utf-8')
print(json.dumps(dict(codeSha=head,matrix=len(matrix),sources=6)))
