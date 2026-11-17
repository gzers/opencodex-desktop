from pathlib import Path
import hashlib,json,subprocess,sys
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE='fb9a64ff79319383f015850d54903866bc6756d7'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
sha=lambda b:hashlib.sha256(b).hexdigest()
paths=set(git('diff','--name-only',BASE).decode().splitlines())
paths.update(git('ls-files','--others','--exclude-standard').decode().splitlines())
paths.discard('apps/desktop/tauri/gen/schemas/windows-schema.json')
prefix='apps/desktop/tauri/src/infrastructure/'
assert paths and all(p.startswith(prefix+'windows_client_originals024/') or p in [prefix+'windows_official_lifecycle024.rs',prefix+'windows_official_lifecycle024/actions.rs','test/windows024-native/build-wl2.ps1'] for p in paths),paths
preserved=[]
for p in [prefix+'windows_client_originals024/mod.rs',prefix+'windows_client_originals024/v2/adapter.rs',prefix+'windows_client_originals024/v3/journal/closed_stop.rs',prefix+'windows_client_originals024/v3/journal/handles.rs',prefix+'windows_client_originals024/handle_release.rs',prefix+'windows_client_originals024/tree/anchored/capture.rs',prefix+'windows_client_originals024/coordination.rs',prefix+'windows_client_originals024/content.rs',prefix+'windows_client_originals024/source.rs',prefix+'windows_diagnostic_roles024.rs',prefix+'windows_supervisor024.rs',prefix+'official_runtime024_transport.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs','apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs']:
    old=git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n')
    new=(CODE/p).read_bytes().replace(b'\r\n',b'\n')
    assert old==new,p
    preserved.append(dict(path=p,normalizedSha256=sha(new)))
p=prefix+'windows_official_lifecycle024.rs'
old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n')
new=(CODE/p).read_text(encoding='utf-8-sig')
marker='pub(crate) fn recovery_state_at('
assert old[old.index(marker):]==new[new.index(marker):]
preserved.append(dict(path=p,section='full recovery_state_at and original same-sample Status receiver tail',normalizedSha256=sha(new[new.index(marker):].encode())))
gate=git('grep','-n','-A','10','fn require_durable_capture','--','apps/desktop/tauri/src/infrastructure').decode()
assert 'Err(' in gate
name=sys.argv[1] if len(sys.argv)>1 else 'source-scope.json'
assert name in ['source-scope.json','source-scope-final.json']
with (ROOT/name).open('x',encoding='utf-8') as f:
    json.dump(dict(base=BASE,changedFiles=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in sorted(paths)],preserved=preserved,publicGateExcerpt=gate,actualProductionExecuted=False,startOpen=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(changed=len(paths),preserved=len(preserved),unchangedFullRecoveryAndStatusReceiver=True)))
