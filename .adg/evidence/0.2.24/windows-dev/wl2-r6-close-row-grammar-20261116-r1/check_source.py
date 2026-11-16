from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE='4e26406402ae20dd3ad8b2c6102c541c2da2d183'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
sha=lambda b:hashlib.sha256(b).hexdigest()
paths=set(git('diff','--name-only',BASE).decode().splitlines())
paths.update(git('ls-files','--others','--exclude-standard').decode().splitlines());paths.discard('apps/desktop/tauri/gen/schemas/windows-schema.json')
assert len(paths)==6,paths
assert all(p.startswith('apps/desktop/tauri/src/infrastructure/windows_client_originals024/') or p=='test/windows024-native/build-wl2.ps1' for p in paths)
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
preserved=[]
for p in [prefix+'v3/journal/handles.rs',prefix+'handle_release.rs',prefix+'tree/anchored/observation/retirement/handoff.rs',prefix+'coordination/segments.rs',prefix+'v3/journal/production.rs',prefix+'tree/anchored/capture.rs',prefix+'v3/journal/completed.rs',prefix+'v3/qualification.rs',prefix+'v3/history.rs',prefix+'content.rs',prefix+'coordination.rs','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs','apps/desktop/tauri/src/infrastructure/windows_official_runtime024/paths.rs']:
    old=git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');raw=(CODE/p).read_bytes().replace(b'\r\n',b'\n');assert old==raw,p
    preserved.append(dict(path=p,normalizedSha256=sha(raw)))
gate=git('grep','-n','-A','10','fn require_durable_capture','--','apps/desktop/tauri/src/infrastructure').decode();assert 'Err(' in gate
with (ROOT/'source-scope.json').open('x',encoding='utf-8') as f:
    json.dump(dict(base=BASE,changedFiles=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in sorted(paths)],preserved=preserved,publicGateExcerpt=gate,actualProductionExecuted=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(changed=len(paths),preserved=len(preserved),nativeCloseAndCompleteInventoryUnchanged=True)))
