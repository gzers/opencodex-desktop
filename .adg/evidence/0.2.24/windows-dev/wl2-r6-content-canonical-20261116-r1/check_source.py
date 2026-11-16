from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE='ed89291540e1d02c886c66020ebbbdc49c2cebeb'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
sha=lambda b:hashlib.sha256(b).hexdigest()
changed=set(git('diff','--name-only',BASE).decode().splitlines())
changed.update(git('ls-files','--others','--exclude-standard').decode().splitlines())
changed.discard('apps/desktop/tauri/gen/schemas/windows-schema.json')
assert all(p.startswith('apps/desktop/tauri/src/infrastructure/windows_client_originals024/') or p=='test/windows024-native/build-wl2.ps1' for p in changed),changed
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
preserved=[]
for p in [prefix+'content.rs',prefix+'coordination.rs',prefix+'v3/capture.rs',prefix+'v3/journal/handles.rs',
          prefix+'v3/journal/completed.rs',prefix+'v3/recovery.rs',prefix+'v3/history.rs',
          'apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024/actions.rs',
          'apps/desktop/tauri/src/infrastructure/windows_diagnostic_roles024.rs',
          'apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs']:
    old=git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');new=(CODE/p).read_bytes().replace(b'\r\n',b'\n')
    assert old==new,p
    preserved.append(dict(path=p,normalizedSha256=sha(new)))
p=prefix+'tree/anchored/observation/series.rs'
old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n');new=(CODE/p).read_text()
marker='pub(in crate::infrastructure::windows_client_originals024) fn assess_tree_series('
assert old[old.index(marker):]==new[new.index(marker):]
preserved.append(dict(path=p,section='original assess_tree_series and resume_trees through EOF',normalizedSha256=sha(new[new.index(marker):].encode())))
raw=git('grep','-n','-A','10','fn require_durable_capture','--','apps/desktop/tauri/src/infrastructure').decode()
assert 'Err(' in raw
with (ROOT/'source-scope-final.json').open('x',encoding='utf-8') as out:
    json.dump(dict(base=BASE,changedFiles=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in sorted(changed)],
        preserved=preserved,publicGateExcerpt=raw,sharedAndOfficialUnchanged=True,newNativeProductionFactoryExecuted=False),out,ensure_ascii=False,indent=2)
print(json.dumps(dict(changed=len(changed),preserved=len(preserved),publicStartOpen=False)))
