from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
paths=['apps/desktop/tauri/src/infrastructure/windows_client_originals024/'+p for p in ['v3/capture.rs','coordination.rs','recovery_worker.rs']]+['apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs']
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip();rows=[]
for name in paths:
    raw=(CODE/name).read_bytes();blob=subprocess.check_output(['git','show',f'{head}:{name}'],cwd=CODE)
    assert raw.replace(bytes([13,10]),bytes([10]))==blob.replace(bytes([13,10]),bytes([10]))
    target=ROOT/'original-source'/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
    rows.append(dict(path=name,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),gitNormalizedBytesEqual=True))
capture=(CODE/paths[0]).read_text(encoding='utf-8');native=(CODE/paths[1]).read_text(encoding='utf-8');worker=(CODE/paths[2]).read_text(encoding='utf-8')
assert 'let _ = lease.release(end);' in capture and 'if let Some(held) = lease.take()' in capture and 'let _ = held.release(end);' in capture
assert 'impl Drop for NativeLease' in native and 'let _ = held.release();' in native
assert 'window: BeforeResumeRecoveryWindow' in worker and 'original.revoke();' in worker and 'park(original, local, worker_state, None)' in worker
result=dict(codeSha=head,sources=rows,method='exact current originalsource inspection, not actual initialStart runtime reproduction',facts=['CaptureStore::capture_held has a local active NativeLease; preparation error consumes release(end) and ignores its result','capture_existing_with_predecessors keeps lease on stack through working move/create/open; error consumes held.release(end) and ignores result','NativeLease::release checks cutoff but still attempts unlock and Drop attempts unlock again; no normal Stop/Closed result is created, yet original range retention is lost','Current dedicated WorkerControl is admitted only from an actual failed Start prepared recovery window and therefore does not own that initial Start lease'],nativeOrOfficialExecuted=False,currentR2Touched=False,scope='only original initialStart failure custody/range ownership dependency paused; independent V2 binding work continues')
(ROOT/'source-findings.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
print(json.dumps(dict(codeSha=head,sources=len(rows),nativeExecuted=False)))
