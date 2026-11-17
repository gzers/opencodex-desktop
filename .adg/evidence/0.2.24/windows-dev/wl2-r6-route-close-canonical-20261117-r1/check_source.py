from pathlib import Path
import hashlib,json,subprocess,sys
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE='b8e06be268182c517a19706dd974961b7e26e57a'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
sha=lambda b:hashlib.sha256(b).hexdigest()
paths=set(git('diff','--name-only',BASE).decode().splitlines());paths.update(git('ls-files','--others','--exclude-standard').decode().splitlines());paths.discard('apps/desktop/tauri/gen/schemas/windows-schema.json')
assert len(paths)==15,paths
assert all(p.startswith('apps/desktop/tauri/src/infrastructure/windows_client_originals024/') or p=='test/windows024-native/build-wl2.ps1' for p in paths)
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
preserved=[]
for p in [prefix+'bounded.rs',prefix+'handle_release.rs',prefix+'v2/routes.rs',prefix+'tree/anchored/capture.rs',prefix+'tree/anchored/observation/retirement/handoff.rs',prefix+'coordination/segments.rs',prefix+'v3/journal/production.rs',prefix+'v3/journal/handles.rs',prefix+'v3/handles.rs',prefix+'v3/qualification.rs',prefix+'v3/recovery.rs',prefix+'v3/read_window.rs',prefix+'coordination.rs','apps/desktop/tauri/src/infrastructure/windows_diagnostic_roles024.rs','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs']:
    old=git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');raw=(CODE/p).read_bytes().replace(b'\r\n',b'\n');assert old==raw,p;preserved.append(dict(path=p,normalizedSha256=sha(raw)))
for p,marker in [(prefix+'content.rs','    pub(super) fn check_completed_history('),(prefix+'v2/routes/close.rs','    pub(in crate::infrastructure::windows_client_originals024) fn retain(')]:
    old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n');new=(CODE/p).read_text(encoding='utf-8-sig');assert old[old.index(marker):]==new[new.index(marker):],p
    preserved.append(dict(path=p,section=marker,normalizedSha256=sha(new[new.index(marker):].encode())))
gate=git('grep','-n','-A','10','fn require_durable_capture','--','apps/desktop/tauri/src/infrastructure').decode();assert 'Err(' in gate
name=sys.argv[1] if len(sys.argv)>1 else 'source-scope.json';assert name in ['source-scope.json','source-scope-final.json','source-scope-verified.json']
with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(dict(base=BASE,changedFiles=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in sorted(paths)],preserved=preserved,publicGateExcerpt=gate,actualProductionExecuted=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(changed=len(paths),preserved=len(preserved),nativePrimitiveCloseAndMutationLanesUnchanged=True)))
