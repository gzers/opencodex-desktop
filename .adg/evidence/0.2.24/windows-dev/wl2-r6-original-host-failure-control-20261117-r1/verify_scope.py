from pathlib import Path
import hashlib,json,subprocess,sys
root=Path(sys.argv[1]).resolve()
baseline='c0d42f8dcc72d56b84fb5bbea050efe2d66ee65f'
def git(*args):return subprocess.check_output(['git','-C',str(root),*args])
allowed={
 'apps/desktop/tauri/src/bin/ocxd.rs',
 'apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs',
 'apps/desktop/tauri/src/infrastructure/windows_owner_host024/mod.rs',
 'apps/desktop/tauri/src/infrastructure/windows_owner_host024/empty_host.rs',
 'apps/desktop/tauri/src/infrastructure/windows_owner_host024/existing_qualification.rs',
 'apps/desktop/tauri/src/infrastructure/windows_owner_host024/failure_control.rs',
 'test/windows024-native/build-wl2.ps1',
}
changed=set(git('diff','--name-only',baseline).decode().splitlines())
changed.update(git('ls-files','--others','--exclude-standard').decode().splitlines())
assert changed and changed<=allowed,changed-allowed
relative='apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs'
old=git('show',baseline+':'+relative).decode().replace(chr(13)+chr(10),chr(10))
new=(root/relative).read_text(encoding='utf-8')
start=new.index('#[cfg(debug_assertions)]'+chr(10)+'pub(crate) struct QualificationFailureCleanup')
end=new.index('#[cfg(debug_assertions)]'+chr(10)+'pub(crate) fn qualification_trace',start)
assert new[:start]+new[end:]==old,'existing production lifecycle changed'
protected=[
 'apps/desktop/tauri/src/infrastructure/windows_official_runtime024/process.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/adapter.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination.rs',
 'apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs',
 'apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs',
 'apps/desktop/ui/package-lock.json',
]
facts=[]
for name in protected:
    expected=git('show',baseline+':'+name).replace(bytes([13,10]),bytes([10]))
    actual=(root/name).read_bytes().replace(bytes([13,10]),bytes([10]))
    assert actual==expected,name
    facts.append(dict(path=name,normalizedSha256=hashlib.sha256(actual).hexdigest()))
print(json.dumps(dict(baseline=baseline,changed=sorted(changed),protected=facts,
 existingLifecycleIncludingFullRecoveryStateAndStatusUnchanged=True,
 nativeAuthority=False),ensure_ascii=False,indent=2))
