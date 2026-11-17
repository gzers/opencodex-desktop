from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code');PREV=ROOT.parent/'wl2-r7-canonical-trait-20261118-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip();assert head.startswith('58f0d49f2') and git('rev-parse','origin/codex/024-windows-native').decode().strip()==head
source=read(ROOT/'source-manifest.json');checks=[]
for row in source['files']:
    raw=(CODE/row['path']).read_bytes();blob=git('show',f"{head}:{row['path']}");assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert raw.replace(b'\r\n',b'\n')==blob.replace(b'\r\n',b'\n');checks.append(dict(path=row['path'],normalizedGitBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(codeSha=head,files=checks))
matrix=read(PREV/'status.json')['matrix'];matrix.extend([
 dict(item='independent strict private control V2 DATA codec',status='6actual productlibrary strict DATA tests passed/raw0/160830ms; closed command/confirmation variants, allfields/nestedbudget/duplicate/unknown/version/framing/locator/localpipe/identity shape strict, no native authority'),
 dict(item='private V2 original QPC budgets',status='BeginCanonicalAbort<=original10 fromsampledstart, WorkerDiagnostic/CanonicalHistory<=Status5; elapsed/frequency/future/expired/overflow/framingwrong refuse; Instant sampledBEFOREnativeQPC and remainingticksfloor, elapsedceil, no transport budgetreset'),
 dict(item='private V2 replay binding',status='exacthello instance/generation/challenge/scope, request/transportaction/challenge singleuse keys and bounded16384set; allvalidatedbeforekeypublication, no arbitrarytargetPID/path/nativeproof or oldv1command'),
 dict(item='private V2 actual reachable transport gap',status='codec only registered/compiled; originalnative server/client protectedlocator/pipe/noJob/sameimage/token/registrydispatch/presourcemutation installation notyetimplemented, no impliedreachability or actualordinarydesktop acceptance'),
 dict(item='private V2 preparation error',status='rustfmt firstraw1 testremoteUNC literal normalquoted invalidescapes p/o, sourcepreserved; rawstringnegative corrected before build. dependentrunpathabsent/no build/native start onthatpreparation; actualproduct01raw0 no unexpected producterror'),
 dict(item='private V2 cleanup',status='no new native fixtures/Ownercalls/recovery/endpoints created; schemasgeneratedraw archived/restored/moved; all priorfailed/rejected roots/synthetic27960 gaps retained/no deletionretry'),
])
write('status.json',dict(codeSha=head,codeBranch='codex/024-windows-native',mainInput=source['mainInput'],docsInput=source['docsInput'],authority='TEAM31/IMP168/r7 unchanged',scope='independent privateV2 wire/clock/replay/locator DATA only; actual server/client/ordinarydesktop reachable originalOwner path incomplete',runs=dict(product01=read(ROOT/'product01/result.json')),artifact=read(ROOT/'artifact.json'),sourceFilesVerified=4,firstUnexpectedProductError=None,preparationError='preparation-error.json + exact beforefixsource',uniqueNewDataTests=6,nativeRecoveryCalls=0,publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof\n',encoding='utf-8')
print(json.dumps(dict(codeSha=head,matrix=len(matrix),sources=4)))
