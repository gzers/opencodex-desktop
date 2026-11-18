from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
def sha(raw):return hashlib.sha256(raw).hexdigest()
names=['product01','backend02','product03','product08','regression04','regression05','regression06','regression07'];runs={name:read(ROOT/name/'result.json') for name in names}
assert [runs[name]['raw'] for name in names]==[101,0,101,0,0,0,0,0]
logs={name:(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig') for name in names}
for name,count in dict(product08=5,regression04=4,regression05=3,regression06=1,regression07=1).items():assert f'test result: ok. {count} passed; 0 failed; 0 ignored;' in logs[name],name
assert logs['product08'].count('actualCreate fault')==5 and logs['product08'].count('native refuses ')==5
roots={}
for name in ['product08','regression04','regression05','regression06']:
    values=sorted(set(p.strip() for p in re.findall(r'r7-(?:directory|v2-reference|journal)-cleanup-absent=(.+)',logs[name])))
    assert all(not Path(p).exists() for p in values),name;roots[name]=values
assert [len(v) for v in roots.values()]==[13,8,3,10]
old=read(ROOT.parent/'wl2-r7-preparation-read-20261118-r1/cleanup-readback.json')
write('cleanup-readback.json',dict(newSuccessfulRootReceipts=roots,totalExactNewRoots=34,newUnexpectedNativeFailures=0,failedAndRejectedRoots=[dict(path=row['path'],present=Path(row['path']).exists()) for row in old['failedAndRejectedRoots']],oldDeletionRetried=False,synthetic27960='previous exact native readonlyidentity/unready plan,no disposition',ordinaryDesktopOutEmpty=not list(Path('C:/Users/Public/ocx-r7-private-native-20261118-r1/out01').iterdir()),currentR2Touched=False,historicalCleanupComplete=False))
sources=read(ROOT/'product08/source.json')['sources'];backend=read(ROOT/'backend02/source.json')['sources']
native='apps/desktop/tauri/src/infrastructure/windows_official_runtime024/paths/preparation.rs'
assert next(row['sha256'] for row in sources if row['path']==native)==next(row['sha256'] for row in backend if row['path']==native)
for row in sources:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256'] and raw==(ROOT/'product08/executed-source'/row['path']).read_bytes(),row['path']
write('source-manifest.json',dict(**read(ROOT/'inputs.json'),files=sources,nativeBackendFinalSourceMatches=True,affectedWritesOnly=True,actualOriginalProductOwnerQualification=False))
base=CODE/'apps/desktop/tauri/src/infrastructure'
native_text=(base/'windows_official_runtime024/paths/preparation.rs').read_text(encoding='utf-8')
borrow=native_text.split('fn retained_native_read(',1)[1].split('/// Bounded DATA',1)[0]
assert all(call not in borrow for call in ['OpenOptions','create_private','serde','unsafe','try_clone'])
assert native_text.index('self.transferred = true')>native_text.index('self.complete.take()') if 'self.complete.take()' in native_text else native_text.index('self.transferred = true')>native_text.index('.take()')
reader=(base/'windows_client_originals024/v3/capture/preparation/directory_read.rs').read_text(encoding='utf-8')
assert reader.index('if let Some(terminal) = original.terminal')<reader.index('let fresh = OpenOptions')
assert reader.count('members(path)?')==2 and reader.count('native_stamp(terminal, end)')==2
assert 'first != second' in reader and 'if !first.is_empty()' in reader
assert all(call not in reader for call in ['.write_all(','.create_new(','.remove_dir(','.set_permissions(','.take_complete(','.into_owned_files('])
mapping=(base/'windows_client_originals024/v3/capture/preparation/v2.rs').read_text(encoding='utf-8')
assert '16 + ordinal' in mapping and '16 + i' in mapping and 'filter_map(|(ordinal, pin)|' in mapping
assert '"nativeDirectoryReads"' in mapping and 'super::directory_read::read(p,end)?' in mapping
write('source-review.json',dict(method='supplemental exact source borrow/path/no mutation/stable ordinal boundary check, not independent security audit/nativeOwner recovery acceptance',checks=dict(originalNativeBorrowNoNewHandleOrSerializeDependency=True,targetObservationOnlyInsideOriginalTerminalBranch=True,twoFullMemberReadsAndAll8IdentityBeforeAfter=True,untransferredDirectoryMustRemainEmpty=True,noInitialSecurityBaselineInferred=True,createdUnopenedAndTransferredNeverReopenTarget=True,finiteStableSourceCustodyPrefixSlots=True,noWriteDeleteRepairOrRecoveryFactory=True),realOriginalOwnerOrInitialSourceCoverage=False))
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe';raw=exe.read_bytes();artifact=sha(raw)
assert all(runs[name]['artifactSha256']==artifact for name in names if name.startswith('regression'))
write('artifact.json',dict(path=str(exe),bytes=len(raw),sha256=artifact,kind='actual full Tauri MSVC library TEST executable; not original desktopinputocxd/release or actual originalproductOwner restoration qualification'))
write('first-error-and-fixes.json',dict(product01=dict(raw=101,elapsedMs=runs['product01']['elapsedMs'],error='E0433 new directory_read module missing explicit native alias import at5sites',fix='explicit module import only, no native tests executed'),product03=dict(raw=101,elapsedMs=runs['product03']['elapsedMs'],error='E0277 passing unsized Path directly to existing Sized projection_digest',fix='hash path reference at callsite, no existing signature widening/no native execution'),backend02=dict(raw=0,elapsedMs=runs['backend02']['elapsedMs'],sameNativeSourceAsFinal=True),product08=dict(raw=0,elapsedMs=runs['product08']['elapsedMs'],passed='5actual native directoryread tests/13fixtures'),unexpectedNativeError=None,toolPreparation='one functions cell had unclosed JS string after patch before tool execution; reissued valid exact cell/no file or native mutation in failedcell',currentR2Touched=False))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:shutil.copyfile(CODE/'apps/desktop/tauri/gen/schemas'/name,schema/name)
(schema/'desktop-schema-original.json').write_bytes(subprocess.check_output(['git','show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'],cwd=CODE))
print(json.dumps(dict(sources=6,roots=34,artifact=artifact,raws={name:dict(raw=row['raw'],ms=row['elapsedMs']) for name,row in runs.items()})))
