from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
def sha(b):return hashlib.sha256(b).hexdigest()
names=['product01','product07','product08','regression02','regression03','regression04','regression05','regression06'];runs={name:read(ROOT/name/'result.json') for name in names}
assert runs['product01']['raw']==101 and all(row['raw']==0 for name,row in runs.items() if name!='product01')
logs={name:(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig') for name in names}
for name,count in dict(product07=4,product08=1,regression02=5,regression03=3,regression04=3,regression05=1,regression06=1).items():assert f'test result: ok. {count} passed; 0 failed; 0 ignored;' in logs[name],name
assert logs['product07'].count('actualwork fault')==4 and logs['product07'].count('native refuses ')==9
assert 'actualopen raw32' in logs['product08']
roots={}
for name in ['product07','product08','regression02','regression03','regression05']:
    values=sorted(set(p.strip() for p in re.findall(r'r7-(?:tree-membership|tree-prefix|v2-reference|driver-worker)-cleanup-absent=(.+)',logs[name])))
    assert all(not Path(p).exists() for p in values),name;roots[name]=values
assert [len(v) for v in roots.values()]==[15,1,11,3,2]
old=read(ROOT.parent/'wl2-r7-directory-read-20261118-r1/cleanup-readback.json')
write('cleanup-readback.json',dict(newSuccessfulRootReceipts=roots,totalExactNewRoots=32,newUnexpectedNativeFailures=0,oldPartialCaptureRegression='original FixtureDrop only; no new exact-root receipt/full historical cleanup claim',failedAndRejectedRoots=[dict(path=row['path'],present=Path(row['path']).exists()) for row in old['failedAndRejectedRoots']],oldDeletionRetried=False,synthetic27960='previous native readonly identity/unready disposition plan only',ordinaryDesktopOutEmpty=not list(Path('C:/Users/Public/ocx-r7-private-native-20261118-r1/out01').iterdir()),currentR2Touched=False,historicalCleanupComplete=False))
sources=read(ROOT/'product08/source.json')['sources'];prior=read(ROOT/'product07/source.json')['sources'];changes=[row['path'] for row in sources if row['sha256']!=next(x['sha256'] for x in prior if x['path']==row['path'])]
assert changes==['apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/before_resume/membership/tests.rs']
for row in sources:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256'] and raw==(ROOT/'product08/executed-source'/row['path']).read_bytes(),row['path']
write('source-manifest.json',dict(**read(ROOT/'inputs.json'),files=sources,onlyChangeAfterProduct07=changes,product07NativeEvidenceReusedBecauseProductionUnchanged=True,realOriginalProductOwnerQualified=False))
base=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
tree=(base/'tree/anchored.rs').read_text(encoding='utf-8');reader=(base/'tree/anchored/before_resume/membership.rs').read_text(encoding='utf-8')
assert tree.index('*pending_locator = Some(')<tree.index('*pending = Some(open_node(')<tree.index('out.push(Node {')<tree.index('*pending_locator = None')
assert tree.count('pending_capture_locator: None')==1
assert '"original"' in reader and '"work"' in reader and '"pendingWork"' in reader and '"pendingCapture"' in reader
assert reader.count('native_membership_pass(session, role, end)?')==2
assert 'Some(&node.original.bytes)' in reader and 'security_digest: observation' in reader and '"originalBaselineVerified"' in reader
assert not any(text in reader for text in ['.write_all(','.rename(','.remove_file(','.into_owned_files(','.take_complete(','.initialize(','self.poisoned = false'])
assert 'objects.sort_by' in reader and 'role, slot, relative' in reader
write('source-review.json',dict(method='supplemental actual native source ordering/full resource membership/read-only/stable key checks; not independent security audit or real productOwner/Source restoration qualification',checks=dict(pendingIntendedRelativeKindBeforeNativeOpen=True,failedOpenNoHandleRemainsIntentNoAdoption=True,allOriginalWorkingAndPendingCreatorsIndividuallyIncluded=True,sameNativeIdSecurityAndBytesBeforeAfterFullTwoPasses=True,unknownMembersAndLateDriftRefuse=True,missingPendingBaselineStaysNull=True,stableKeysIndependentOfVectorOrderAndWorkAddition=True,poisonAndNormalMutationGatesRetained=True,noWriteMoveDropProofOrOwnerFactory=True),claimOrSourceRecoveryPermissionSupplied=False))
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe';raw=exe.read_bytes();artifact=sha(raw)
assert all(row['artifactSha256']==artifact for name,row in runs.items() if name.startswith('regression'))
write('artifact.json',dict(path=str(exe),bytes=len(raw),sha256=artifact,kind='actual full Tauri MSVC library TEST executable; not desktopinputocxd/release or actual fixedofficialOwner qualification'))
write('first-error-and-fixes.json',dict(product01=dict(raw=101,elapsedMs=runs['product01']['elapsedMs'],error='new tests used assert_eq/unwrap_err requiring Debug for strict Object type (E0277); no native execution',fix='test equality predicate and explicit .err() branch; no Object type or security assertion changed'),product07=dict(raw=0,elapsedMs=runs['product07']['elapsedMs'],passed='4actual native membership tests/15fixtures'),product08=dict(raw=0,elapsedMs=runs['product08']['elapsedMs'],passed='real share0 open conflict raw32/no handle: intent preserved after actual conflict release, never reopened'),productionSourceUnchangedAfterProduct07=True,unexpectedNativeError=None,currentR2Touched=False))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:shutil.copyfile(CODE/'apps/desktop/tauri/gen/schemas'/name,schema/name)
(schema/'desktop-schema-original.json').write_bytes(subprocess.check_output(['git','show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'],cwd=CODE))
print(json.dumps(dict(sources=7,roots=32,artifact=artifact,runs={name:dict(raw=row['raw'],ms=row['elapsedMs']) for name,row in runs.items()})))
