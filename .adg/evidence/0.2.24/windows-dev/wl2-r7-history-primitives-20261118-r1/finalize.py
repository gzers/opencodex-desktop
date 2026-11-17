from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
PREV=ROOT.parent/'wl2-r7-return-validation-20261117-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip();assert head.startswith('f8feb3825')
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
 dict(item='TEAM29/IMP164 input consumption',status='main0675a105017ae7fc82a4bd58dd43fe72031470df merged, onlyMacsmoketestdelta from built81f/noRustgraph change; docsb4a54df778b3322e46e895af9a5836f604c86346 merged; r7exclusivewrite set/sharedtraits unchanged; statusbusyErr staysUnknown/Required'),
 dict(item='native original quiescence historical read',status='2actual product-library native tests passed; same original privateproof completed_at<olddeadline and actual original Job total1active0/root242; old strict API stillTimedOut, expiredreader/foreignactualproof reject; read only no prepare/Resume/Stop/Closed'),
 dict(item='native V1/V2 attachment historical DATA read',status='2actual native tests12scenes passed; original retained prefix completedtimely withinoriginalwriteend then fresh bounded nativefile/fullchain/all8/ADS/membership read; partial/missingcompletion/memberSD/expiredread refuse, originalappend remains expired/rewindow reject; independentdecoders/noadoption'),
 dict(item='history primitives standalone backend',status='backend02cargo check raw0/3046ms; finalnative source exactlysame aschecked; no newschema/serialization dependency'),
 dict(item='history primitives first failures',status='product01raw101E0583explicitpathparentregistration,no native. product03raw101 storage12passed,2Jobfixtures raw2missingHbeforeCreateJob/Process; onlytestpath/Hsetupfixed,22fileoriginalfailuretrees+2ZIP retained'),
 dict(item='history primitives affected regressions',status='V1native5/raw0/343ms; V2native6/raw0/1032ms; originalquiescejoin1/3actualprefix/raw0/2765ms; originalconcretepipeline1/2actualmodes/raw0/9797ms; no unaffected/W2/full official rerun'),
 dict(item='history primitives cleanup',status='65successful ownroots absent acrossexecuted revisions/regressions;2new failedroots22files readonlyarchived/source retained; priorfailed/rejectedroots+synthetic27960 gap retained, no deletionretry'),
 dict(item='complete original product historicalStatus gap',status='these nativeJob/attachmentprimitives only; full originalSlot validatedcompletion/nativeOriginalSet/core/Source/canonical/route/finalproof bound toONE Status5 entry notimplemented yet; metadataobserve nevercountsfullhistory or successorpermission'),
])
runs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
write('status.json',dict(codeSha=head,codeBranch='codex/024-windows-native',mainBuilt=sources['mainBuilt'],mainConsumed=sources['mainConsumed'],mainOnlyUncompiledDelta=sources['mainOnlyUncompiledDelta'],docsConsumed=sources['docsConsumed'],authority='TEAM29/IMP164/r7 unchanged',scope='foundational native historical read only; no complete originalproductOwner status or restoration acceptance',runs=runs,artifact=read(ROOT/'artifact.json'),sourceFilesVerified=9,firstError=read(ROOT/'first-error-and-fixes.json'),uniqueNewNativeTests=4,uniqueNewNativeScenes=14,affectedRegressionTests=13,publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof\n',encoding='utf-8')
print(json.dumps(dict(codeSha=head,matrix=len(matrix),sources=9)))
