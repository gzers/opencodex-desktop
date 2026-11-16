from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE='502858eb56f1a5401684259e7420bc0712c6cb4f'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
sha=lambda b:hashlib.sha256(b).hexdigest()
paths=git('diff','--name-only',BASE).decode().splitlines()
assert len(paths)==7,paths
assert all(p.startswith('apps/desktop/tauri/src/infrastructure/windows_client_originals024/') or p in ['apps/desktop/tauri/src/infrastructure/windows_official_runtime024/paths.rs','test/windows024-native/build-wl2.ps1'] for p in paths)
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
preserved=[]
for p in [prefix+'v3/journal/production.rs',prefix+'tree/anchored/capture.rs',prefix+'v3/journal/completed.rs',prefix+'v3/qualification.rs',prefix+'v3/recovery.rs',prefix+'v3/handles.rs',prefix+'v3/history.rs',prefix+'content.rs',prefix+'coordination.rs','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs']:
    old=git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');raw=(CODE/p).read_bytes().replace(b'\r\n',b'\n');assert old==raw,p
    preserved.append(dict(path=p,normalizedSha256=sha(raw)))
sections=[(prefix+'handle_release.rs','    pub(super) fn close_all(','    pub(super) fn completed('),
          (prefix+'coordination/segments.rs','    pub(in super::super) fn transfer_original_files(','fn facts_from_files('),
          (prefix+'tree/anchored/observation/retirement/handoff.rs','    pub(in crate::infrastructure::windows_client_originals024) fn detach(','    pub(in crate::infrastructure::windows_client_originals024) fn pending_history_references('),
          ('apps/desktop/tauri/src/infrastructure/windows_official_runtime024/paths.rs','    pub(crate) fn into_owned_files(',None)]
for p,start,stop in sections:
    old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n');new=(CODE/p).read_text()
    def section(s):
        i=s.index(start);return s[i:s.index(stop,i)] if stop else s[i:]
    assert section(old)==section(new),p
    preserved.append(dict(path=p,section=start,normalizedSha256=sha(section(new).encode())))
gate=git('grep','-n','-A','10','fn require_durable_capture','--','apps/desktop/tauri/src/infrastructure').decode();assert 'Err(' in gate
with (ROOT/'source-scope.json').open('x',encoding='utf-8') as f:
    json.dump(dict(base=BASE,changedFiles=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in paths],preserved=preserved,publicGateExcerpt=gate,actualProductionExecuted=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(changed=len(paths),preserved=len(preserved),actualOriginalCloseUnchanged=True)))
