from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a]).decode().strip()
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
allowed={prefix+p for p in ['mod.rs','coordination.rs','coordination/segments.rs','handle_release.rs','tree.rs',
    'tree/anchored/observation.rs','tree/anchored/observation/retirement.rs','tree/anchored/observation/retirement/witness.rs','tree/anchored/observation/retirement/handoff.rs',
    'tree/anchored/observation/retirement_handoff_tests.rs','v3.rs','v3/history.rs','v3/history_data_tests.rs',
    'v3/handles.rs','v3/journal.rs','v3/journal/handles.rs','v3/journal/recovery.rs','v3/segments.rs']}
allowed|={'apps/desktop/tauri/src/infrastructure/windows_official_runtime024/paths.rs','test/windows024-native/build-wl2.ps1'}
base='b30cc913201f972d62b2d2b07a990fd5f5cded44'
changed=set(git('diff','--name-only',base,'--').splitlines());untracked=set(git('ls-files','--others','--exclude-standard').splitlines())
preexisting={'apps/desktop/tauri/gen/schemas/windows-schema.json'}
assert changed|untracked==allowed|preexisting,(changed,untracked)
rows=[]
for path in sorted(allowed):
    raw=(CODE/path).read_bytes();rows.append(dict(path=path,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
with (ROOT/'source-scope.json').open('x',encoding='utf-8') as out:
    json.dump(dict(baseline=base,changedFiles=rows,preexistingUntrackedNotStaged=sorted(preexisting),sharedOfficialDailyConfigurationModified=False),out,indent=2)
print(json.dumps(dict(changedFiles=len(rows),withinWindowsWriteSet=True,preexistingUntrackedPreserved=True)))
