from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(raw):return hashlib.sha256(raw).hexdigest()
results={name:read(ROOT/name/'result.json') for name in ['product01','product02','product03','regression04','regression05','regression06','regression07']}
assert [results[n]['raw'] for n in results]==[101,101,0,0,0,0,0]
logs={name:(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig') for name in results}
assert '1 passed; 1 failed;' in logs['product02']
assert 'native valid:' in logs['product02'] and 'native foreign-released:' in logs['product02']
for n,count in [('product03',1),('regression04',3),('regression05',2),('regression06',1),('regression07',1)]:
    assert f'test result: ok. {count} passed; 0 failed;' in logs[n],n
scenes={}
for name in ['product02','product03','regression04','regression05','regression06']:
    rows=set(re.findall(r'r7-(?:tree-prefix|return-validation-worker|worker|driver-worker)-cleanup-absent=(.+)',logs[name]))
    rows=sorted(p.strip() for p in rows)
    assert all(not Path(p).exists() for p in rows),name
    scenes[name]=rows
assert [len(v) for v in scenes.values()]==[6,4,7,10,2]
prior=read(ROOT.parent/'wl2-r7-v2-reference-20261117-r1/cleanup-readback.json')
old=prior['oldRejectedRootsRetained']+prior['lastDriverFailureRootsRetained']+['C:/Users/15119/AppData/Local/Temp/.tmpJEMT1O']
write('cleanup-readback.json',dict(newOwnRootsAbsent=scenes,totalRoots=29,failedAndRejectedRoots=[dict(path=p,present=Path(p).exists()) for p in old],newFailureArchive='product02/failure-readonly.zip',synthetic27960='readonly native identity evidence separately pushed a752014ab; no disposition',historicalCleanupComplete=False,oldDeletionRetried=False,currentR2Touched=False))
sources=read(ROOT/'product03/source.json')['sources']
before=read(ROOT/'product02/source.json')['sources']
changes=[r['path'] for r in sources if r['sha256']!=next(x['sha256'] for x in before if x['path']==r['path'])]
assert changes==['apps/desktop/tauri/src/infrastructure/windows_client_originals024/recovery_worker/tests/return_validation_tests.rs'],changes
for r in sources:
    raw=(CODE/r['path']).read_bytes()
    assert len(raw)==r['bytes'] and sha(raw)==r['sha256'] and raw==(ROOT/'product03/executed-source'/r['path']).read_bytes(),r['path']
write('source-manifest.json',dict(codeBefore=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip(),mainInput=subprocess.check_output(['git','rev-parse','origin/main'],cwd=CODE).decode().strip(),docsInput=subprocess.check_output(['git','rev-parse','origin/docs/governance-main'],cwd=CODE).decode().strip(),files=sources,coreNativeProduct02Reused=True,onlyChangeAfterProduct02=changes))
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe'
raw=exe.read_bytes();artifact=sha(raw)
assert all(results[n]['artifactSha256']==artifact for n in ['regression04','regression05','regression06','regression07'])
write('artifact.json',dict(path=str(exe),bytes=len(raw),sha256=artifact,kind='actual full Tauri MSVC library TEST exe, not fixed release candidate or real original productOwner qualification'))
for mode in ['success','failure','expired','panic','disconnect','release-partial','release-expired']:
    child=ROOT/'regression04/subprocess'/mode
    assert all((child/p).exists() for p in ['entered.json','observed.json','cleanup.json','child.stdout.log','child.stderr.log'])
    raw=read(child/'cleanup.json')['ownFixtureRaw'];assert raw==(0 if mode=='success' else 242)
write('first-error-and-fixes.json',dict(product01=dict(raw=101,error='E0624 test called private validate_returned twice, E0382 test tried drop of already consumed old_lease',nativeExecuted=False,fix='cfg(test) wrapper only; keep production generic callback private, remove duplicate consumed fixture drop'),product02=dict(raw=101,error='new worker fixture attempted private source file directly in unprotected temp root; native helper correctly PermissionDenied before worker creation',passed='all6fullnativecorevalidation modes',fix='new own private source directory per existing native fixture pattern; preserve4file failure tree+readonlyZIP'),product03=dict(raw=0,passed='all4actualworkerreturnvalidation modes, core source unchanged/reused'),boundariesChanged=False,currentR2Touched=False))
base=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
validation=(base/'recovery_worker/return_validation.rs').read_text(encoding='utf-8')
assert validation.index('retained.original = Some(original)')<validation.index('retained.attempted = true')<validation.index('value.original.binding(&value.window)?')
assert 'ReleaseProbe' not in validation and 'BeforeResumeRecoveryWindow::begin' not in validation
core=(base/'tree/anchored/capture/abort_return/released.rs').read_text(encoding='utf-8').split('fn recheck_abort_return<',1)[1]
assert core.count('restored(trees, end)?')==2 and 'scope.record_verified' not in core and 'observe_released_ranges' not in core
write('source-review.json',dict(method='supplemental actual source-order/private factory check, not independent safety audit',checks=dict(originalReturnedBeforeFallibleValidation=True,originalWindowValueTransported=True,oneShotNoRewindow=True,workerValidatorThreadsSeparate=True,noNewProbeOrRecordInRecheck=True,completeBothTreesReadTwice=True,ordinaryGateStillOccupied=True),nativeOriginalOwnerRuntimeAccepted=False))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:shutil.copyfile(CODE/'apps/desktop/tauri/gen/schemas'/name,schema/name)
(schema/'desktop-schema-original.json').write_bytes(subprocess.check_output(['git','show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'],cwd=CODE))
print(json.dumps(dict(raws={k:v['raw'] for k,v in results.items()},roots=29,sources=len(sources),artifact=artifact)))
