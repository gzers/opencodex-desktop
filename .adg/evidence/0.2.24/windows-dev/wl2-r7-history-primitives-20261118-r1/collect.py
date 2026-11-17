from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(b):return hashlib.sha256(b).hexdigest()
runs={n:read(ROOT/n/'result.json') for n in ['product01','backend02','product03','product04','native05','regression06','regression07','regression08','regression09']}
assert [r['raw'] for r in runs.values()]==[101,0,101,0,0,0,0,0,0]
logs={n:(ROOT/n/'stdout.log').read_text(encoding='utf-8-sig') for n in runs}
assert '2 passed; 2 failed;' in logs['product03']
for n,count in [('product04',1),('native05',1),('regression06',5),('regression07',6),('regression08',1),('regression09',1)]:assert f'test result: ok. {count} passed; 0 failed;' in logs[n],n
rows={}
for name in ['product03','product04','native05','regression06','regression07','regression08','regression09']:
    paths=sorted(set(p.strip() for p in re.findall(r'r7-(?:v2-attachment|attachment|tree-prefix|driver-worker)-cleanup-absent=(.+)',logs[name])))
    assert all(not Path(p).exists() for p in paths),name
    rows[name]=paths
assert [len(v) for v in rows.values()]==[12,1,1,20,26,3,2]
prior=read(ROOT.parent/'wl2-r7-return-validation-20261117-r1/cleanup-readback.json')
old=[r['path'] for r in prior['failedAndRejectedRoots']]+['C:/Users/15119/AppData/Local/Temp/.tmpErP935','C:/Users/15119/AppData/Local/Temp/.tmpvhhXRI']
write('cleanup-readback.json',dict(newSuccessfulRootsAbsent=rows,totalRoots=65,failedAndRejectedRoots=[dict(path=p,present=Path(p).exists()) for p in old],newFailureArchives='product03/failure-readonly-{0,1}.zip, 22files originals kept',oldDeletionRetried=False,currentR2Touched=False,synthetic27960='readonly observed in separately pushed a752014ab, no disposition',historicalCleanupComplete=False))
sources=read(ROOT/'product04/source.json')['sources'];before=read(ROOT/'product03/source.json')['sources'];backend=read(ROOT/'backend02/source.json')['sources']
changed=[r['path'] for r in sources if r['sha256']!=next(x['sha256'] for x in before if x['path']==r['path'])]
assert changed==['apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/before_resume/quiesce_tests/history_tests.rs'],changed
native='apps/desktop/tauri/src/infrastructure/windows_official_runtime024/before_resume.rs'
assert next(r['sha256'] for r in sources if r['path']==native)==next(r['sha256'] for r in backend if r['path']==native)
for r in sources:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and sha(raw)==r['sha256'] and raw==(ROOT/'product04/executed-source'/r['path']).read_bytes()
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
mainbuilt='81f52452cb4cfafc64a80ca92d97a3ba5990c2fb';mainconsumed=git('rev-parse','origin/main').decode().strip()
names=git('diff','--name-only',mainbuilt,mainconsumed).decode().splitlines()
assert names==['test/smoke/macos024-gui-root-cancel.py'],names
write('source-manifest.json',dict(codeBefore=git('rev-parse','HEAD').decode().strip(),mainBuilt=mainbuilt,mainConsumed=mainconsumed,mainOnlyUncompiledDelta=names,rustProductGraphUnchangedByLastMainIntake=True,docsConsumed=git('rev-parse','origin/docs/governance-main').decode().strip(),authority='TEAM29/IMP164/r7 unchanged',files=sources,attachmentHistoriesProduct03Reused=True,onlyChangeAfterProduct03=changed,standaloneBackendNativeSourceSame=True))
base=CODE/'apps/desktop/tauri/src/infrastructure'
nativecode=(base/'windows_official_runtime024/before_resume.rs').read_text(encoding='utf-8')
assert 'completed_at: Instant::now()' in nativecode and 'proof.completed_at >= proof.deadline' in nativecode
for version in ['before_resume_record','before_resume_record_v2']:
    text=(base/'windows_client_originals024'/version/'storage.rs').read_text(encoding='utf-8')
    method=text.split('fn read_history_data(',1)[1].split('pub(in crate::infrastructure::windows_client_originals024) fn read_data(',1)[0]
    assert 'last_complete_at.ok_or_else(invalid)?' in method and 'read_retained_files(read_end)' in method
    assert not any(call in method for call in ['create_private_journal_file','append_data(','self.end ='])
    append=text.split('fn append_data(',1)[1]
    assert append.index('self.read_inner(end)?')<append.index('self.last_complete_at = Some(completed)')
write('source-review.json',dict(method='supplemental actual source/private field and write-read boundary ordering, not independent securityaudit/full productionstatus acceptance',checks=dict(nativePrivateTimelyCompletionRequired=True,originalStrictQuiescenceStillOldDeadline=True,originalStorageAppendCutoffUnchanged=True,completeNativeAppendBeforeHistoricalFact=True,noHistoryCreateAppendOrWindowReset=True,distinctV1V2ParsersPreserved=True),fullProductHistoryImplemented=False))
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe';raw=exe.read_bytes();artifact=sha(raw)
assert all(runs[n]['artifactSha256']==artifact for n in ['native05','regression06','regression07','regression08','regression09'])
write('artifact.json',dict(path=str(exe),bytes=len(raw),sha256=artifact,kind='actual full Tauri MSVC library TEST exe; no fixed release or complete productOwner history qualification'))
failure_source=ROOT/'failure-source';failure_source.mkdir(exist_ok=False)
shutil.copyfile(base/'windows_official_runtime024/process.rs',failure_source/'process.rs')
write('first-error-and-fixes.json',dict(product01=dict(raw=101,elapsedMs=runs['product01']['elapsedMs'],error='E0583 tests history module under explicit-path parent needed explicit nested path; no native execution',fix='path attribute only'),product03=dict(raw=101,elapsedMs=runs['product03']['elapsedMs'],passed='12actual V1/V2historyscenes',error='2nativeJobfixtures raw2 before Job/process creation, required privateH missing; old pair helper used to supply it',fix='standalone test historical_scene creates exact own privateH; raw22file failures retained+archive'),product04=dict(raw=0,passed='actual original nativeJob timely242/oldexpired/no renewedmutation'),native05=dict(raw=0,passed='actual2differentJob nativeproof substitution refusals, both ownraw242'),boundariesChanged=False))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:shutil.copyfile(CODE/'apps/desktop/tauri/gen/schemas'/name,schema/name)
(schema/'desktop-schema-original.json').write_bytes(git('show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'))
print(json.dumps(dict(roots=65,sources=9,artifact=artifact,raws={k:v['raw'] for k,v in runs.items()})))
