from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(b):return hashlib.sha256(b).hexdigest()
runs={name:read(ROOT/name/'result.json') for name in ['product01','product02',*[f'regression{i:02}' for i in range(3,11)]]}
assert runs['product01']['raw']==101 and all(v['raw']==0 for k,v in runs.items() if k!='product01')
logs={name:(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig') for name in runs}
assert '2 passed; 1 failed;' in logs['product01']
assert all(f'actualnative {mode}:' in logs['product01'] for mode in ['valid-after-expiry','tree-member','wrong-original','expired-reader'])
counts=dict(product02=1,regression03=2,regression04=3,regression05=2,regression06=1,regression07=5,regression08=4,regression09=1,regression10=2)
for name,count in counts.items():assert f'test result: ok. {count} passed; 0 failed;' in logs[name],name
scenes={}
for name in ['product01','product02','regression03','regression04','regression05','regression06']:
    paths=sorted(set(p.strip() for p in re.findall(r'r7-(?:return-validation-worker|tree-prefix|worker|driver-worker)-cleanup-absent=(.+)',logs[name])))
    assert all(not Path(p).exists() for p in paths),name
    scenes[name]=paths
assert [len(v) for v in scenes.values()]==[5,6,10,7,10,2]
old=read(ROOT.parent/'wl2-r7-history-primitives-20261118-r1/cleanup-readback.json')
retained=[r['path'] for r in old['failedAndRejectedRoots']]+['C:/Users/15119/AppData/Local/Temp/.tmpQ94pON']
write('cleanup-readback.json',dict(newSuccessfulRootsAbsent=scenes,totalRoots=40,failedAndRejectedRoots=[dict(path=p,present=Path(p).exists()) for p in retained],newFailureArchive='product01/failure-readonly.zip exact5remaining files AFTER attemptedfixtureclose; not full originalscene',synthetic27960='previous separate native readonly observation/plan, no disposition',oldDeletionRetried=False,currentR2Touched=False,historicalCleanupComplete=False))
sources=read(ROOT/'product02/source.json')['sources'];before=read(ROOT/'product01/source.json')['sources']
changes=[r['path'] for r in sources if r['sha256']!=next(x['sha256'] for x in before if x['path']==r['path'])]
assert changes==['apps/desktop/tauri/src/infrastructure/windows_client_originals024/recovery_worker/tests/return_validation_tests/history_tests.rs'],changes
for r in sources:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and sha(raw)==r['sha256'] and raw==(ROOT/'product02/executed-source'/r['path']).read_bytes(),r['path']
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
main_built=read(ROOT/'inputs.json')['mainInput'];main_consumed='fb755b8ea63d8a3fc9146dc4db491931919905b0'
delta=git('diff','--name-only',main_built,main_consumed).decode().splitlines()
assert set(delta)=={'.github/scripts/verify_macos_updater_024.py','.github/workflows/ci.yml','.github/workflows/release.yml','test/test_verify_macos_updater_024.py'}
subprocess.run(['git','merge-base','--is-ancestor',main_consumed,'HEAD'],cwd=CODE,check=True)
write('source-manifest.json',dict(codeBefore=git('rev-parse','HEAD').decode().strip(),mainBuilt=main_built,mainConsumed=main_consumed,mainOnlyUncompiledDelta=delta,rustProductGraphUnchangedByMainIntake=True,docsConsumed=git('rev-parse','origin/docs/governance-main').decode().strip(),authority='TEAM30/IMP166/r7 unchanged',files=sources,coreAndQpcProduct01Reused=True,onlyChangeAfterProduct01=changes))
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe';raw=exe.read_bytes();artifact=sha(raw)
assert all(v['artifactSha256']==artifact for k,v in runs.items() if k.startswith('regression'))
write('artifact.json',dict(path=str(exe),bytes=len(raw),sha256=artifact,kind='actual full Tauri MSVC library TEST exe; no fixed release/actual original productOwner qualified'))
for mode in ['success','failure','expired','panic','disconnect','release-partial','release-expired']:
    child=ROOT/'regression04/subprocess'/mode
    assert all((child/n).exists() for n in ['entered.json','observed.json','cleanup.json','child.stdout.log','child.stderr.log'])
    assert read(child/'cleanup.json')['ownFixtureRaw']==(0 if mode=='success' else 242)
base=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
core=(base/'tree/anchored/capture/abort_return/released.rs').read_text(encoding='utf-8').split('fn read_abort_history',1)[1]
assert core.count('restored(trees, read_end)?')==2 and core.count('scope.check(')==2
assert 'observe_released_ranges' not in core and 'record_verified(' not in core
worker=(base/'recovery_worker/return_validation/history.rs').read_text(encoding='utf-8')
assert 'Duration::from_secs(5)' in worker and 'retained.completed_at.ok_or_else(invalid)?' in worker
assert 'BeforeResumeRecoveryWindow::begin' not in worker and 'Phase::ReturnedValidated' in worker and 'HistoryReadFailedRetained' in worker
journal=(base/'v3/journal/before_resume.rs').read_text(encoding='utf-8').split('fn read_before_resume_released_history',1)[1].split('fn read_before_resume_released(',1)[0]
assert journal.count('source.check(')==2 and journal.count('check_retained_owner_for_read')==2 and journal.count('check_before_resume_history_owner')==2
assert not any(re.search(r'^\s*unsafe\s+impl.*(?:Send|Sync)',(CODE/r['path']).read_text(encoding='utf-8'),re.M) for r in sources)
write('source-review.json',dict(method='supplemental actual source/private factories and before-after ordering; not independent security audit/real productOwner acceptance',checks=dict(actualTimelyOriginalValidationRequired=True,originalNativeWindowRetained=True,oneStatus5CallerCutoffNoNewWindow=True,fullBothTreeReadTwice=True,sourceCanonicalOriginalNativeActionReleaseReadBeforeAfter=True,typedProducerInitialLastAndFinalStrictRecordMatch=True,noNewRangeOrRecordInHistory=True,noNewUnsafeSend=True),actualProductOwnerRuntimeQualified=False,ordinaryDispatchStillBlocked=True))
write('first-error-and-fixes.json',dict(product01=dict(raw=101,elapsedMs=runs['product01']['elapsedMs'],passed='full4nativecore modes+nativeQPC; receiving valid/unvalidated admission correct',error='own fixture TempDir close raw32 unvalidated originals still held in return receiver; exact5remainingfiles afterpartialclose archived',fix='test explicit joinedworker then drop receiver+validation+state before scene; no production logic/limits changed'),product02=dict(raw=0,elapsedMs=runs['product02']['elapsedMs'],passed='all6receivingcontrol history modes'),coreAndClockSourceUnchanged=True,currentR2Touched=False))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:shutil.copyfile(CODE/'apps/desktop/tauri/gen/schemas'/name,schema/name)
(schema/'desktop-schema-original.json').write_bytes(git('show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'))
print(json.dumps(dict(sourceCount=len(sources),newRootsAbsent=40,artifact=artifact,runs={k:dict(raw=v['raw'],ms=v['elapsedMs']) for k,v in runs.items()})))
