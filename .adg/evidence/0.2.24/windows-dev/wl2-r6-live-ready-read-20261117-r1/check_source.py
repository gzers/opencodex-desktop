from pathlib import Path
import hashlib,json,subprocess,sys
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code');BASE='7c0bc016b4daedada6fad3f2a0884ae40e7b7fc3'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a]);sha=lambda b:hashlib.sha256(b).hexdigest()
paths=set(git('diff','--name-only',BASE).decode().splitlines());paths.update(git('ls-files','--others','--exclude-standard').decode().splitlines());paths.discard('apps/desktop/tauri/gen/schemas/windows-schema.json')
assert len(paths)==9,paths
assert all(p.startswith('apps/desktop/tauri/src/infrastructure/windows_client_originals024/') or p=='test/windows024-native/build-wl2.ps1' for p in paths)
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/';preserved=[]
for p in [prefix+'v3/journal/closed_stop.rs',prefix+'v3/journal/completed.rs',prefix+'v3/journal/handles.rs',prefix+'handle_release.rs',prefix+'v2/routes.rs',prefix+'coordination/segments.rs',prefix+'coordination.rs',prefix+'content.rs',prefix+'v3/journal/production.rs','apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024/actions.rs','apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs','apps/desktop/tauri/src/infrastructure/official_runtime024_transport.rs','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs']:
    old=git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');new=(CODE/p).read_bytes().replace(b'\r\n',b'\n');assert old==new,p;preserved.append(dict(path=p,normalizedSha256=sha(new)))
p=prefix+'tree/anchored/observation.rs';old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n');new=(CODE/p).read_text(encoding='utf-8-sig');assert old==new.replace('mod ready_read;\n','');preserved.append(dict(path=p,section='all old observation/custody/claim checks unchanged; only mod registration',normalizedSha256=sha(old.encode())))
gate=git('grep','-n','-A','10','fn require_durable_capture','--','apps/desktop/tauri/src/infrastructure').decode();assert 'Err(' in gate
name=sys.argv[1] if len(sys.argv)>1 else 'source-scope.json';assert name in ['source-scope.json','source-scope-final.json','source-scope-verified.json']
with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(dict(base=BASE,changedFiles=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in sorted(paths)],preserved=preserved,publicGateExcerpt=gate,actualTypedReadyReadFactoryExecuted=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(changed=len(paths),preserved=len(preserved),oldMutationAndCompleteStopUntouched=True)))
