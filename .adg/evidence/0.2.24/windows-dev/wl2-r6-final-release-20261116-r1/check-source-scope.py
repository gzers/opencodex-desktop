from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a]).decode().strip()
base='139de12617ebfdb2f5d78c27d93a47abd177eae6'
allowed={
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/recovery.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/recovery_data_tests.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/segments.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/journal/leases.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/journal/recovery.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/journal/recovery/final_release.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/observation/retirement.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/observation/retirement/witness.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/observation/retirement_chain_tests.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/observation/retirement_release_tests.rs',
 'test/windows024-native/build-wl2.ps1',
}
changed=set(git('diff','--name-only',base,'--').splitlines())
untracked=set(git('ls-files','--others','--exclude-standard').splitlines())
preexisting={'apps/desktop/tauri/gen/schemas/windows-schema.json'}
assert changed|untracked==allowed|preexisting,(changed,untracked)
rows=[]
for path in sorted(allowed):
    raw=(CODE/path).read_bytes();rows.append(dict(path=path,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
with (ROOT/'source-scope.json').open('x',encoding='utf-8') as out:
    json.dump(dict(baseline=base,changedFiles=rows,preexistingUntrackedNotStaged=sorted(preexisting),sharedOfficialDailyConfigurationModified=False),out,indent=2)
print(json.dumps(dict(changedFiles=len(rows),withinWindowsWriteSet=True,preexistingUntrackedPreserved=True)))
