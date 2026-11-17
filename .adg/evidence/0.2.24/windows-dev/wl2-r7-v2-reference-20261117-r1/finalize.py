from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent
BASE=ROOT.parent/'wl2-r7-concrete-driver-20261117-r1'
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p): return json.loads(p.read_text(encoding='utf-8-sig'))
def write(name,data): (ROOT/name).write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def git(*args): return subprocess.check_output(['git',*args],cwd=CODE)
head=git('rev-parse','HEAD').decode().strip()
assert head=='dd42d5e3871b71f860a5149bd22ea0166fd717b1'
assert git('rev-parse','origin/codex/024-windows-native').decode().strip()==head
sources=read(ROOT/'source-manifest.json')
sourcechecks=[]
for row in sources['files']:
    raw=(CODE/row['path']).read_bytes()
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'],row['path']
    blob=git('show',f"{head}:{row['path']}")
    assert blob.replace(b'\r\n',b'\n')==raw.replace(b'\r\n',b'\n'),row['path']
    sourcechecks.append(dict(path=row['path'],gitNormalizedBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(codeSha=head,files=sourcechecks))
artifact=read(ROOT/'artifact.json'); raw=Path(artifact['path']).read_bytes()
assert len(raw)==artifact['bytes'] and hashlib.sha256(raw).hexdigest()==artifact['sha256']
runs={d.name:read(d/'result.json') for d in ROOT.iterdir() if d.is_dir() and (d/'result.json').exists()}
assert len(runs)==7 and all(r['raw']==0 for r in runs.values())
for key,count in [('product01',11),('product03',12),('native04',3),('regression05',4),('regression06',3),('regression07',1)]:
    log=(ROOT/key/'stdout.log').read_text(encoding='utf-8-sig')
    assert re.search(rf'test result: ok\. {count} passed; 0 failed;',log),key
cleanup=read(ROOT/'cleanup-readback.json')
assert sum(map(len,cleanup['newV2FixtureRootsAbsent'].values()))==8
assert all(not Path(p).exists() for rows in cleanup['newV2FixtureRootsAbsent'].values() for p in rows)
old=read(BASE/'status.json')
matrix=old['matrix']
for row in matrix:
    if row['item']=='pre-manifest capture attachment':
        row['status']='TEAM24A independent strictV2 DATA+actual preparation membership implemented/tested; native persistence/full proof and recovery still incomplete'
    if row['item']=='future canonical abort recovery record/strict schema':
        row['status']='strictv1 native canonical component evidence reused; independent strictv2 DATA/chain now9tests passed, partialnative mapping3tests; nativev2producer/fullpartial recovery incomplete'
matrix.extend([
 dict(item='TEAM28/IMP159 main intake',status='mainfcf04caec/docs0612d4139 merged; Mac-only incremental change, no affected Windows native path; no unaffected reruns'),
 dict(item='independent strict V2 reference/record/chain DATA',status='9 meaningful strict DATA tests passed in actual product library; explicit null/closed variants/required roles/byte linkage/coverage/replay/terminal pending refusal; no v1/v3 fallback or native authority'),
 dict(item='original native preparation V2 membership',status='3 actual native tests passed; original session+process host UUID retained before source capture; actual creator/pins/tree/pending resources mapped; nativeID failures raw/kind retained; unverified SD/content null=>Pending'),
 dict(item='native preparation stages',status='actual create-success alone NativeCreated, failed/expired create IntentPublished, actual original retainedtree/pins HandleRetained; fixed within same deadline'),
 dict(item='standalone native backend compatibility',status='actual original backend cargo check raw0; tuple getter only/no newserde dependency'),
 dict(item='affected abort/partialcapture/Send regressions',status='4+3+1 passed raw0; no currentr2/official run'),
 dict(item='V2 new fixture cleanup',status='8 own roots absent across executed revisions; oldpartial regression only originalDrop, no full historicalcleanup claim'),
 dict(item='V2 supplemental review collectors',status='two collection errors retained (format-sensitive lookup, broad substring); corrected source review passes, not independent security audit'),
 dict(item='V2 remaining native scope',status='journal aggregated actual membership only; full nativeV2 record producer/persistence/partial prefix restore/8+content+membership verification absent; no DATA terminal grants restoration'),
])
write('status.json',dict(codeSha=head,codeCommits=[git('rev-parse','HEAD~1').decode().strip(),head],codeBranch='codex/024-windows-native',mainInput=sources['mainConsumed'],docsInput=sources['docsConsumed'],authority='TEAM28/IMP159/r7 unchanged',scope='independent strictV2 DATA/chain + conservative original native preparation membership; no complete native V2 restoration',runs=runs,uniqueFinalTests=dict(strictData=9,nativePreparation=3,affectedRegressions=8),artifact=artifact,sourceFilesVerified=len(sourcechecks),firstUnexpectedProductError=None,supplementalCollectionErrors=['source-review01-error.json','source-review02-error.json'],publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=cleanup,matrix=matrix))
for name in ['verify_evidence.py','verify_git_payload.py']: shutil.copyfile(BASE/name,ROOT/name)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof\n',encoding='utf-8')
print(json.dumps(dict(codeSha=head,runs={k:dict(raw=v['raw'],elapsedMs=v['elapsedMs']) for k,v in runs.items()},matrix=len(matrix),verifiedSources=len(sourcechecks)),ensure_ascii=False))
