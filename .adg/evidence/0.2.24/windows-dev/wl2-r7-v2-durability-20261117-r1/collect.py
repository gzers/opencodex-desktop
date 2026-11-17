from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def write(name,value): (ROOT/name).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def read(path): return json.loads(path.read_text(encoding='utf-8-sig'))
result=read(ROOT/'product01/result.json'); assert result['raw']==0
log=(ROOT/'product01/stdout.log').read_text(encoding='utf-8-sig')
assert 'test result: ok. 6 passed; 0 failed; 0 ignored;' in log
roots=re.findall(r'r7-v2-attachment-cleanup-absent=(.+)',log)
roots=[p.strip() for p in roots];assert len(roots)==26 and len(set(roots))==26
assert all(not Path(p).exists() for p in roots)
old=read(ROOT.parent/'wl2-r7-v2-reference-20261117-r1/cleanup-readback.json')
retained=old['oldRejectedRootsRetained']+old['lastDriverFailureRootsRetained']
write('cleanup-readback.json',dict(newRoots=[dict(path=p,absent=True) for p in roots],oldRetained=[dict(path=p,present=Path(p).exists()) for p in retained],oldDeletionRetried=False,currentR2Touched=False,scope='only this new own DATA storage fixtures; original fixture tests close their own resources, never currentr2 or old rejected roots'))
sources=read(ROOT/'product01/source.json')['sources']
for row in sources:
    raw=(CODE/row['path']).read_bytes();snapshot=(ROOT/'product01/executed-source'/row['path']).read_bytes()
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and raw==snapshot
write('source-manifest.json',dict(codeBefore=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip(),files=sources))
base=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
text=(base/'before_resume_record_v2/storage.rs').read_text(encoding='utf-8')
assert 'before_resume_record::' not in text and 'Record::decode' not in text and 'validate_chain(' not in text
initialize=text.split('fn initialize(',1)[1]
append=text.split('fn append_data(',1)[1]
assert initialize.index('self.directory = Some(directory)')<initialize.index('self.cutoff(end)?')
assert append.index('validate_chain_v2(&chain)?')<append.index('self.pending_raw = Some(raw.to_vec())')<append.index('self.pending_file = Some(native::create_private_journal_file(')<append.index('self.checkpoint(2, end)?')
assert len(re.findall(r'fn wl2_r7_v2_storage_native_', (base/'before_resume_record_v2/storage/tests.rs').read_text(encoding='utf-8')))==6
write('source-review.json',dict(method='supplemental source order and independent decoder/namespace check; not independent security audit/native recovery acceptance',checks=dict(independentV2Decoder=True,strictPreviousAndCandidateBeforeCreate=True,directoryRetainedBeforeChecks=True,fileRetainedBeforePostchecks=True,pendingFailureUnretryable=True,sameDeadlineOnly=True),nativeAuthorityFromData=False,currentR2Touched=False))
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe'
raw=exe.read_bytes()
write('artifact.json',dict(path=str(exe),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),kind='actual full Tauri MSVC library TEST executable, not release or productOwner restoration qualification'))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:
    src=CODE/'apps/desktop/tauri/gen/schemas'/name
    shutil.copyfile(src,schema/name)
original=subprocess.check_output(['git','show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'],cwd=CODE)
(schema/'desktop-schema-original.json').write_bytes(original)
print(json.dumps(dict(raw=result['raw'],passed=6,newFixtureRootsAbsent=len(roots),sourcesVerified=len(sources),artifactSha256=hashlib.sha256(raw).hexdigest())))
