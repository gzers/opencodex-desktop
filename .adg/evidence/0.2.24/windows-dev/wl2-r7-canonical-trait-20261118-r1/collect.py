from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(raw):return hashlib.sha256(raw).hexdigest()
runs={name:read(ROOT/name/'result.json') for name in ['product01','product02',*[f'regression{i:02}' for i in range(2,7)]]}
assert runs['product01']['raw']==101 and all(v['raw']==0 for k,v in runs.items() if k!='product01')
logs={name:(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig') for name in runs}
assert '2 passed; 1 failed;' in logs['product01']
for name,count in dict(product02=1,regression02=3,regression03=1,regression04=2,regression05=5,regression06=1).items():assert f'test result: ok. {count} passed; 0 failed;' in logs[name],name
roots={}
for name in ['product02','regression02','regression03','regression04']:
    paths=sorted(set(p.strip() for p in re.findall(r'r7-(?:canonical-trait|worker|tree-prefix|driver-worker|return-validation-worker)-cleanup-absent=(.+)',logs[name])))
    assert all(not Path(p).exists() for p in paths),name
    roots[name]=paths
assert [len(rows) for rows in roots.values()]==[1,7,2,10]
old=read(ROOT.parent/'wl2-r7-full-history-20261118-r1/cleanup-readback.json')
retained=[r['path'] for r in old['failedAndRejectedRoots']]+['C:/Users/15119/AppData/Local/Temp/.tmpe2DUqC']
write('cleanup-readback.json',dict(newSuccessfulRootsAbsent=roots,totalRoots=20,failedAndRejectedRoots=[dict(path=p,present=Path(p).exists()) for p in retained],newFailureArchive='product01/failure-readonly.zip exact4filetaskroot retained',oldDeletionRetried=False,currentR2Touched=False,synthetic27960='prior separate readonlyidentity/plan, no disposition',historicalCleanupComplete=False))
sources=read(ROOT/'product02/source.json')['sources'];before=read(ROOT/'product01/source.json')['sources']
changes=[r['path'] for r in sources if r['sha256']!=next(x['sha256'] for x in before if x['path']==r['path'])]
assert changes==['apps/desktop/tauri/src/infrastructure/windows_client_originals024/recovery_worker/canonical_trait/tests.rs']
for r in sources:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and sha(raw)==r['sha256'] and raw==(ROOT/'product02/executed-source'/r['path']).read_bytes()
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
inputs=read(ROOT/'inputs.json');main_consumed='4ea82e8a61fe00c1f76334db5e39654f49f4342e'
delta=git('diff','--name-only',inputs['mainInput'],main_consumed).decode().splitlines()
assert set(delta)=={'.github/scripts/verify_windows_installer_024.py','.github/workflows/ci.yml','.github/workflows/release.yml','test/test_verify_windows_installer_024.py'}
subprocess.run(['git','merge-base','--is-ancestor',main_consumed,'HEAD'],cwd=CODE,check=True)
write('source-manifest.json',dict(codeBefore=git('rev-parse','HEAD').decode().strip(),mainBuilt=inputs['mainInput'],mainConsumed=main_consumed,mainOnlyUncompiledDelta=delta,rustProductGraphUnchangedByMainIntake=True,docsConsumed=git('rev-parse','origin/docs/governance-main').decode().strip(),authority='TEAM31/IMP168/r7 unchanged',files=sources,metadataTypesProduct01Reused=True,onlyChangeAfterProduct01=changes))
base=CODE/'apps/desktop/tauri/src/infrastructure'
trait=(base/'windows_client_originals024/recovery_worker/canonical_trait.rs').read_text(encoding='utf-8')
driver=(base/'windows_official_lifecycle024/actions/recovery/driver.rs').read_text(encoding='utf-8')
prepare=trait.split('fn prepare_abort(',1)[1].split('fn quiesce_launcher(',1)[0]
quiesce=trait.split('fn quiesce_launcher(',1)[1].split('fn restore_originals(',1)[0]
restore=trait.split('fn restore_originals(',1)[1]
assert prepare.index('self.canonical = Some(Box::new(Attempt')<prepare.index('self.acquire(owner, window)')
assert prepare.index('.prefix = Some(prefix)')<prepare.index('initialize_abort_attachment')
assert quiesce.index('owner.with_originals(')<quiesce.index('.prefix\n')<quiesce.index('.take()')
assert quiesce.index('.quiescent = Some(quiescent)')<quiesce.index('claim.check(')
assert restore.index('originals.return_quiescent_originals')<restore.index('self.release(window.deadline())')<restore.index('self.finish_released_stage')
assert driver.index('local.prepare_abort')<driver.index('local.quiesce_launcher')<driver.index('local.restore_originals')<driver.index('restored.check_original')
assert 'BeforeResumeRecoveryWindow::begin' not in trait+driver
assert not any(re.search(r'^\s*unsafe\s+impl.*(?:Send|Sync)',(CODE/r['path']).read_text(encoding='utf-8'),re.M) for r in sources)
write('source-review.json',dict(method='supplemental source/private-stage retention order, not independent safety audit or realproductOwner native acceptance',checks=dict(unchangedSharedTraitImplemented=True,actualWorkerAttemptBeforeFallibleLocks=True,originalPreparedPrefixRetainedBeforeAttachment=True,takePrefixInsideOriginalAdapterLock=True,actualQuiescentTagRetainedBeforeChecks=True,returnBeforeExplicitReleaseAndFullRead=True,productionDriverUsesThreeTraitStages=True,sameOriginalWindowNoNewTimer=True,noNewUnsafeSend=True),fullV2PartialRestoration=False,actualProductOwnerRuntimeQualified=False))
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe';raw=exe.read_bytes();artifact=sha(raw)
assert all(v['artifactSha256']==artifact for k,v in runs.items() if k.startswith('regression'))
write('artifact.json',dict(path=str(exe),bytes=len(raw),sha256=artifact,kind='actual full Tauri MSVC library TEST exe; not fixed release/actualoriginalproductOwnerqualified'))
for mode in ['success','failure','expired','panic','disconnect','release-partial','release-expired']:
    child=ROOT/'regression02/subprocess'/mode
    assert all((child/n).exists() for n in ['entered.json','observed.json','cleanup.json','child.stdout.log','child.stderr.log'])
    assert read(child/'cleanup.json')['ownFixtureRaw']==(0 if mode=='success' else 242)
write('first-error-and-fixes.json',dict(product01=dict(raw=101,elapsedMs=runs['product01']['elapsedMs'],passed='actual4ranges raw33 across3borrowstages/epochkey, explicitrelease/readback; nonClone/nonserde and metadata guards',error='new fixture asserted lease OptionNone but original release intentionally keeps inactive drained lease shell',fix='onlyfixture tests actual4ranges reacquirable+explicitlyunlocked and oldlease rejectsclaim, no primitive/production/limits changed;4filefailure root/ZIPretained'),product02=dict(raw=0,elapsedMs=runs['product02']['elapsedMs'],passed='actualsamelease continuity and release/oldclaimrefusal'),productionSourceUnchangedAfterFirstRun=True,currentR2Touched=False))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:shutil.copyfile(CODE/'apps/desktop/tauri/gen/schemas'/name,schema/name)
(schema/'desktop-schema-original.json').write_bytes(git('show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'))
print(json.dumps(dict(sources=len(sources),roots=20,artifact=artifact,runs={k:dict(raw=v['raw'],ms=v['elapsedMs']) for k,v in runs.items()})))
