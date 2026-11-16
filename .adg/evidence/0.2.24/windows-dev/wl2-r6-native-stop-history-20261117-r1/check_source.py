from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE='18a079fa4cf1c628bab1ad9c3e67513e0a227402'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
sha=lambda b:hashlib.sha256(b).hexdigest()
paths=set(git('diff','--name-only',BASE).decode().splitlines());paths.update(git('ls-files','--others','--exclude-standard').decode().splitlines())
paths.discard('apps/desktop/tauri/gen/schemas/windows-schema.json')
assert len(paths)==14,paths
assert all(p.startswith('apps/desktop/tauri/src/infrastructure/windows_client_originals024/') or p in ['apps/desktop/tauri/src/infrastructure/windows_diagnostic_roles024.rs','test/windows024-native/build-wl2.ps1'] for p in paths)
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
preserved=[]
for p in [prefix+'v3/journal/handles.rs',prefix+'handle_release.rs',prefix+'tree/anchored/capture.rs',prefix+'tree/anchored/observation/retirement/handoff.rs',prefix+'coordination/segments.rs',prefix+'v3/journal/production.rs',prefix+'v3/qualification.rs',prefix+'v3/recovery.rs',prefix+'v3/handle_rows.rs',prefix+'v3/history.rs',prefix+'coordination.rs','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs','apps/desktop/tauri/src/infrastructure/windows_official_runtime024/paths.rs','apps/desktop/tauri/src/infrastructure/windows_supervisor024.rs']:
    old=git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');raw=(CODE/p).read_bytes().replace(b'\r\n',b'\n');assert old==raw,p
    preserved.append(dict(path=p,normalizedSha256=sha(raw)))
for p,start,stop in [(prefix+'content.rs','    pub(super) fn capture(',None),
    ('apps/desktop/tauri/src/infrastructure/windows_diagnostic_roles024.rs','impl NormalExit {',None),
    (prefix+'v3/journal/completed.rs','    fn completed_history_at(','    #[cfg(test)]\n    pub(in crate::infrastructure::windows_client_originals024) fn component_completed_history_at_for_test(')]:
    old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n');new=(CODE/p).read_text(encoding='utf-8-sig')
    def section(s):
        i=s.index(start);return s[i:s.index(stop,i)] if stop else s[i:]
    assert section(old)==section(new),p
    preserved.append(dict(path=p,section=start,normalizedSha256=sha(section(new).encode())))
gate=git('grep','-n','-A','10','fn require_durable_capture','--','apps/desktop/tauri/src/infrastructure').decode();assert 'Err(' in gate
with (ROOT/'source-scope-final.json').open('x',encoding='utf-8') as f:json.dump(dict(base=BASE,changedFiles=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in sorted(paths)],preserved=preserved,publicGateExcerpt=gate,actualProductionExecuted=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(changed=len(paths),preserved=len(preserved),oldNormalExitAndNativeRestoreCloseUnchanged=True)))
