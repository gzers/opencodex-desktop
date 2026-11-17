from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
assert git('rev-parse','HEAD').decode().strip()=='4452d583d16f33865a27d8a8518e3a330bd57d2a'
assert load(ROOT/'library01/result.json')['exitCode']==0
log=(ROOT/'library01/stdout.log').read_text(encoding='utf-8-sig');assert '1 passed; 0 failed; 1 ignored' in log
paths=set(git('diff','--name-only').decode().splitlines())
paths.update(git('ls-files','--others','--exclude-standard').decode().splitlines())
paths.discard('apps/desktop/tauri/gen/schemas/windows-schema.json')
assert len(paths)==6,paths
assert all(p.startswith('test/windows024-native/') or p in ['apps/desktop/tauri/src/infrastructure/windows_client_originals024/mod.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/desktop_preflight_tests.rs'] for p in paths)
assert not git('diff','--cached','--name-only').strip()
exceptions=[]
for row in load(ROOT/'library01/source-manifest.json'):
    raw=(ROOT/'library01/executed-source'/row['path']).read_bytes()
    assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
    now=(CODE/row['path']).read_bytes()
    if raw!=now:
        assert row['path']=='test/windows024-native/launch-r6-profile-preflight.ps1'
        exceptions.append(dict(path=row['path'],buildSnapshotSha256=sha(raw),finalSha256=sha(now),reason='Added exact same-SID evidence export invocation after terminal native Rust build; independent PowerShell helper does not affect compiled Rust tests. Parsed in actual WindowsPowerShell5.1.'))
for p in ['test/windows024-native/export-r6-profile-preflight.ps1']:
    assert p in paths
    exceptions.append(dict(path=p,finalSha256=sha((CODE/p).read_bytes()),reason='Standalone same-SID task-only log exporter added after terminal Rust build, no Rust/test assertion changed. Actual PowerShell5.1 parse checked.'))
rows=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in sorted(paths)]
git('add','--',*sorted(paths));assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths)
git('diff','--cached','--check')
out=git('commit','-m','test(windows): 准备普通桌面原件基线并保留缺席事实')
(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:json.dump(dict(head=head,tree=tree,files=rows,scriptOnlyAfterBuildChanges=exceptions,nativeRustTestPassed=1,ordinaryDesktopTestIgnoredPendingUser=True,actualProductionExecuted=False,startOpen=False),f,indent=2)
print(json.dumps(dict(head=head,files=len(paths),ordinaryDesktopExecutionPending=True)))
