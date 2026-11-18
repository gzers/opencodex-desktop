from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
def sha(b):return hashlib.sha256(b).hexdigest()
names=['product01','product02','product03','product08','regression04','regression05','regression06','regression07']
runs={name:read(ROOT/name/'result.json') for name in names}
assert runs['product01']['raw']==101 and all(row['raw']==0 for name,row in runs.items() if name!='product01')
logs={name:(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig') for name in names}
for name,count in dict(product02=3,product03=1,product08=1,regression04=3,regression05=3,regression06=3,regression07=1).items():assert f'test result: ok. {count} passed; 0 failed;' in logs[name],name
assert logs['product02'].count('actualjournal fault')==10
assert logs['product02'].count('actualjournal refuses ')==9
assert 'expired originalStart:' in logs['product02']
assert logs['product08'].count('actualslot fault')==10
roots={}
for name in ['product02','product03','product08','regression04','regression05']:
    values=sorted(set(p.strip() for p in re.findall(r'r7-(?:journal|v2-reference)-cleanup-absent=(.+)',logs[name])))
    assert all(not Path(p).exists() for p in values),name
    roots[name]=values
assert [len(roots[name]) for name in ['product02','product03','product08']]==[20,10,10]
previous=read(ROOT.parent/'wl2-r7-private-native-20261118-r1/cleanup-readback.json')
write('cleanup-readback.json',dict(successfulOwnRootAbsence=roots,totalExactNewRootReceipts=sum(len(v) for v in roots.values()),oldPartialCaptureRegression='original FixtureDrop only, no exact per-root receipt asserted; no full historical cleanup claim',newUnexpectedNativeFailures=0,failedAndRejectedRoots=[dict(path=row['path'],present=Path(row['path']).exists()) for row in previous['failedAndRejectedRoots']],oldDeletionRetried=False,synthetic27960='previous separate native readonly identity and unready plan only; no disposition',currentR2Touched=False,ordinaryDesktopOutEmpty=not list(Path('C:/Users/Public/ocx-r7-private-native-20261118-r1/out01').iterdir()),historicalCleanupComplete=False))
sources=read(ROOT/'product08/source.json')['sources'];old=read(ROOT/'product03/source.json')['sources']
changes=[row['path'] for row in sources if row['path'] not in {x['path'] for x in old} or row['sha256']!=next(x['sha256'] for x in old if x['path']==row['path'])]
assert set(changes)=={'apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/capture/preparation.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/capture/preparation/v2.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/journal/preparation/read_tests.rs'}
for row in sources:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256'] and raw==(ROOT/'product08/executed-source'/row['path']).read_bytes(),row['path']
write('source-manifest.json',dict(**read(ROOT/'inputs.json'),files=sources,postProduct03Changes=changes,nativeReadImplementationUnchangedAfterProduct03=True,product02RefusalAndExpiredReadReused='only later file_row output fixed finite slot label and stable DATA ordinal, no native-read checks/rights/limits changed; native read old negative assertions unchanged',notRealProductOwnerQualification=True))
base=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
text=(base/'v3/journal/preparation/read.rs').read_text(encoding='utf-8')
assert 'record.security.clone' not in text and 'let baseline = &record.security' in text
assert text.count('partial_rows(')==3 and text.count('after_rows(')==3
assert 'self.poison();' in text and 'self.ready = true' not in text and '.create(' not in text
assert not any(call in text for call in ['.write_all(','.set_len(','.rename(','.take_complete(','.initialize('])
assert 'expected.starts_with(&first)' in text and 'if objects != again' in text
assert 'originalSecurityBaselineVerified' in text and ('Role::Journal,'+chr(10)+'            2,') in text
mapping=(base/'v3/capture/preparation/v2.rs').read_text(encoding='utf-8')
assert 'journal_prefix.v2_native_objects(session, end)?' in mapping
assert 'journal_members["completeOriginalJournal"] == true' in mapping and 'journal_members["manifestOriginalHandle"] == true' in mapping
write('source-review.json',dict(method='supplemental actual source ordering/native-read/no-mutation/stable-slot check, not independent security audit or productionOwner restoration acceptance',checks=dict(originalSecurityBaselineBorrowedNoClone=True,allOriginalFileRowsReadTwice=True,fullMembersPinsPeerDirectoryBeforeAfter=True,partialBytesMustMatchOriginalExpectedPrefix=True,pendingMissingSecurityBaselineNeverFilledByObservation=True,noCreateWriteInitializeClearFailureOrProofFactory=True,fixedManifest0Current1Aggregate2ObjectLabels=True,actualSlotPendingAndStageMappingTested=True),fullOriginalSourceAdmission=False,fullPartialRecoveryOrLockReleaseAccepted=False))
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe'
raw=exe.read_bytes();artifact=sha(raw)
assert all(row['artifactSha256']==artifact for name,row in runs.items() if name.startswith('regression'))
write('artifact.json',dict(path=str(exe),bytes=len(raw),sha256=artifact,kind='actual full Tauri MSVC library TEST executable; not desktopinputocxd/release/realoriginalproductOwner recovery qualification'))
write('first-error-and-fixes.json',dict(preparation='rustfmt raw1 extra replaced-branch brace; exact v2-before-brace-fix.rs kept; no product/native started',product01=dict(raw=101,elapsedMs=runs['product01']['elapsedMs'],error='E0599 immutable native SnapshotData has no Clone; native not executed',fix='borrow original stored raw/path/security fields; no native type/API widening'),product02=dict(raw=0,elapsedMs=runs['product02']['elapsedMs'],passed='3actualnative tests20scenes'),product03=dict(raw=0,elapsedMs=runs['product03']['elapsedMs'],passed='10initializerstage nativefixtures stablemanifest/current/aggregate slots'),product08=dict(raw=0,elapsedMs=runs['product08']['elapsedMs'],passed='actual original CaptureSlot/PartialCaptureAnchor receives sameJournalPreparation in10faultstates; Sourceunknown meansPending'),unexpectedNativeError=None,currentR2Touched=False))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:shutil.copyfile(CODE/'apps/desktop/tauri/gen/schemas'/name,schema/name)
(schema/'desktop-schema-original.json').write_bytes(subprocess.check_output(['git','show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'],cwd=CODE))
print(json.dumps(dict(sources=len(sources),newRootReceipts=sum(len(v) for v in roots.values()),rootsByRun={name:len(v) for name,v in roots.items()},artifact=artifact,runs={name:dict(raw=row['raw'],ms=row['elapsedMs']) for name,row in runs.items()})))
