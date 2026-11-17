from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(b):return hashlib.sha256(b).hexdigest()
result=read(ROOT/'product01/result.json');assert result['raw']==0
assert 'test result: ok. 6 passed; 0 failed; 0 ignored;' in (ROOT/'product01/stdout.log').read_text(encoding='utf-8-sig')
sources=read(ROOT/'product01/source.json')['sources']
for row in sources:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256'] and raw==(ROOT/'product01/executed-source'/row['path']).read_bytes()
inputs=read(ROOT/'inputs.json');write('source-manifest.json',dict(**inputs,files=sources))
wire=(CODE/'apps/desktop/tauri/src/infrastructure/windows_owner_host024/recovery_control/wire.rs').read_text(encoding='utf-8')
assert wire.index('let sampled = Instant::now()')<wire.index('let (counter, frequency) = qpc()?')
assert 'let seconds = if self.command == CommandV2::BeginCanonicalAbort' in wire
assert wire.index('used.extend(keys)')>wire.index('frequency.checked_mul') if 'frequency.checked_mul' in wire else wire.index('used.extend(keys)')>wire.index('checked_mul(seconds)')
assert 'windows_official_lifecycle024' not in wire and 'BeforeResumeRecoveryWindow::begin' not in wire and 'force_owned_cleanup' not in wire
write('source-review.json',dict(method='supplemental strictcodec/QPCsample/replay/source check, not independent safety audit/native peer or Owner test',checks=dict(independentVersion2Schema=True,noOldDecoderFallback=True,finiteCommandConfirmation=True,sampledInstantBeforeNativeQpc=True,elapsed10Or5NoRenewal=True,replayKeysEnrolledOnlyAfterChecks=True,noNativeOwnerDispatchOrProofFactory=True),nativeServerClientImplemented=False))
prior=read(ROOT.parent/'wl2-r7-canonical-trait-20261118-r1/cleanup-readback.json')
write('cleanup-readback.json',dict(newNativeFixturesCreated=0,nativeRecoveryCalls=0,failedAndRejectedRoots=[dict(path=r['path'],present=Path(r['path']).exists()) for r in prior['failedAndRejectedRoots']],oldDeletionRetried=False,currentR2Touched=False,synthetic27960='priorreadonlyexactidentity/plan,no disposition',historicalCleanupComplete=False))
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe';raw=exe.read_bytes();write('artifact.json',dict(path=str(exe),bytes=len(raw),sha256=sha(raw),kind='actual full Tauri MSVC library TEST exe; no release/nativehost/actualOwnerqualification'))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:shutil.copyfile(CODE/'apps/desktop/tauri/gen/schemas'/name,schema/name)
(schema/'desktop-schema-original.json').write_bytes(subprocess.check_output(['git','show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'],cwd=CODE))
print(json.dumps(dict(passed=6,raw=0,ms=result['elapsedMs'],sources=4,artifact=sha(raw))))
