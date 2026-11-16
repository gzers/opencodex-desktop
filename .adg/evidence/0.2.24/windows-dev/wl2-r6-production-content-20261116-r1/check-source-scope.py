from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a]).decode().strip()
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
allowed={prefix+p for p in ['mod.rs','source.rs','content.rs','content_tests.rs','tree.rs','tree/anchored/capture.rs',
 'tree/anchored/observation.rs','tree/anchored/observation/series.rs','tree/anchored/observation/retirement/witness.rs',
 'v3/capture.rs','v3/journal.rs','v3/journal/production.rs','v3/journal/snapshots.rs','v3/journal/recovery.rs',
 'v3/journal/batches.rs','v3/journal/recovery/final_release.rs','v3/journal/history.rs']}
allowed|={'apps/desktop/tauri/src/infrastructure/windows_diagnostic_roles024.rs','apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024/actions.rs','test/windows024-native/build-wl2.ps1'}
base='c259886cf5723409a1c77d41d1f9b42196bd6b61'
changed=set(git('diff','--name-only',base,'--').splitlines());untracked=set(git('ls-files','--others','--exclude-standard').splitlines())
preexisting={'apps/desktop/tauri/gen/schemas/windows-schema.json'}
assert changed|untracked==allowed|preexisting,(changed,untracked)
rows=[]
for path in sorted(allowed):
    raw=(CODE/path).read_bytes();rows.append(dict(path=path,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
with (ROOT/'source-scope.json').open('x',encoding='utf-8') as out:json.dump(dict(baseline=base,changedFiles=rows,
    preexistingUntrackedNotManuallyEditedOrStaged=sorted(preexisting),sharedOfficialDailyConfigurationModified=False),out,indent=2)
print(json.dumps(dict(changedFiles=len(rows),withinWindowsWriteSet=True,preexistingSchemaPreserved=True)))
