from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a]).decode().strip()
base='d20103fee084a3e63f728bf7c6664af65405e456'
prefix='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
allowed={prefix+p for p in ['tree.rs','tree/anchored.rs','tree/anchored/batches.rs','tree/anchored/capture.rs',
 'tree/anchored/observation/retirement.rs','tree/anchored/observation/retirement/witness.rs',
 'tree/anchored/observation/retirement_batch_tests.rs','tree/anchored/observation/retirement_chain_tests.rs',
 'tree/anchored/observation/retirement_completed_tests.rs',
 'v3.rs','v3/actions.rs','v3/batches.rs','v3/batch_data_tests.rs','v3/capture.rs','v3/handles.rs','v3/history.rs',
 'v3/observations.rs','v3/recovery.rs','v3/recovery_data_tests.rs','v3/segments.rs','v3/snapshots.rs',
 'v3/journal.rs','v3/journal_tests.rs','v3/journal/batches.rs','v3/journal/completed.rs','v3/journal/driver.rs','v3/journal/handles.rs','v3/journal/history.rs',
 'v3/journal/recovery.rs','v3/journal/recovery/final_release.rs']}
allowed.add('test/windows024-native/build-wl2.ps1')
changed=set(git('diff','--name-only',base,'--').splitlines());untracked=set(git('ls-files','--others','--exclude-standard').splitlines())
preexisting={'apps/desktop/tauri/gen/schemas/windows-schema.json'}
assert changed|untracked==allowed|preexisting,(changed,untracked)
rows=[]
for path in sorted(allowed):
    raw=(CODE/path).read_bytes();rows.append(dict(path=path,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
with (ROOT/'source-scope.json').open('x',encoding='utf-8') as out:json.dump(dict(baseline=base,changedFiles=rows,
    preexistingUntrackedNotManuallyEditedOrStaged=sorted(preexisting),sharedOfficialDailyConfigurationModified=False),out,indent=2)
print(json.dumps(dict(changedFiles=len(rows),withinWindowsWriteSet=True,preexistingSchemaPreserved=True)))
