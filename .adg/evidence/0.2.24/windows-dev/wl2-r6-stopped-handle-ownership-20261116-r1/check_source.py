from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE='d29dbc36d567d4344407a07d652311f2bfa140d9'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
sha=lambda b:hashlib.sha256(b).hexdigest()
paths=git('diff','--name-only',BASE).decode().splitlines()
assert len(paths)==7 and all(p.startswith('apps/desktop/tauri/src/infrastructure/windows_client_originals024/') or p=='test/windows024-native/build-wl2.ps1' for p in paths),paths
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
rows=[]
for p in [prefix+'handle_release.rs',prefix+'tree/anchored/observation/retirement/handoff.rs',prefix+'v3/journal/completed.rs',prefix+'content.rs',prefix+'v3/qualification.rs',prefix+'v3/snapshots.rs',prefix+'v3/journal/recovery.rs',prefix+'v3/journal/history.rs',prefix+'coordination/segments.rs','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs']:
    old=git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');raw=(CODE/p).read_bytes().replace(b'\r\n',b'\n')
    assert old==raw,p
    rows.append(dict(path=p,normalizedSha256=sha(raw)))
p=prefix+'v3/journal/handles.rs'
old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n');new=(CODE/p).read_text()
def body(s):
    start=s.index('fn close_original_handles(');return s[start:s.index('    #[cfg(test)]\n    pub(in crate::infrastructure::windows_client_originals024) fn component_close_original_handles_for_test(',start)]
assert body(old)==body(new)
rows.append(dict(path=p,section='original close_original_handles body, visibility only widened to journal parent',normalizedSha256=sha(body(new).encode())))
gate=git('grep','-n','-A','10','fn require_durable_capture','--','apps/desktop/tauri/src/infrastructure').decode();assert 'Err(' in gate
with (ROOT/'source-scope-final.json').open('x',encoding='utf-8') as f:
    json.dump(dict(base=BASE,changedFiles=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in paths],preserved=rows,publicGateExcerpt=gate,productionPositiveExecuted=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(changed=len(paths),preserved=len(rows),originalCloseBodyUnchanged=True,publicStartOpen=False)))
