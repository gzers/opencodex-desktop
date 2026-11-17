from pathlib import Path
import hashlib,json,re,subprocess,sys
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE='f5fd8211e5e6aeafc0c025887175c1f5905ab5af'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest()
run=sys.argv[1];assert re.fullmatch(r'library\d\d',run)
assert load(ROOT/run/'result.json')['exitCode']==0
log=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',log);assert m
assert git('rev-parse','HEAD').decode().strip()==BASE and not git('diff','--cached','--name-only').strip()
paths=set(git('diff','--name-only').decode().splitlines());paths.update(git('ls-files','--others','--exclude-standard').decode().splitlines());paths.discard('apps/desktop/tauri/gen/schemas/windows-schema.json')
prefix='apps/desktop/tauri/src/infrastructure/'
assert all(p.startswith(prefix+'windows_client_originals024/') or p.startswith(prefix+'windows_owner_host024/') or p==prefix+'windows_official_lifecycle024.rs' or p.startswith('test/windows024-native/') for p in paths)
assert len(paths)==11,paths
preserved=[]
for p in [prefix+'windows_client_originals024/adapter.rs',prefix+'windows_client_originals024/v3/capture.rs',prefix+'windows_client_originals024/source.rs',prefix+'windows_client_originals024/content.rs',prefix+'windows_client_originals024/v3/journal/closed_stop.rs',prefix+'windows_client_originals024/v3/journal/handles.rs',prefix+'windows_diagnostic_roles024.rs',prefix+'windows_supervisor024.rs',prefix+'official_runtime024_transport.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs','apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs']:
    raw=(CODE/p).read_bytes().replace(b'\r\n',b'\n');assert raw==git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');preserved.append(dict(path=p,normalizedSha256=sha(raw)))
p=prefix+'windows_official_lifecycle024.rs';old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n');new=(CODE/p).read_text(encoding='utf-8-sig');marker='pub(crate) fn recovery_state_at('
assert old[old.index(marker):]==new[new.index(marker):]
assert 'super::windows_client_originals024::require_durable_capture()' in new[new.index('async fn execute_async('):new.index('async fn execute_admitted(')]
preserved.append(dict(path=p,section='complete recovery_state_at/StatusReceipt/same-sample consumer tail',normalizedSha256=sha(new[new.index(marker):].encode())))
exceptions=[]
for row in load(ROOT/run/'source-manifest.json'):
    raw=(ROOT/run/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
    now=(CODE/row['path']).read_bytes()
    if raw!=now:
        assert row['path'].startswith('test/windows024-native/') and row['path'].endswith(('.ps1','.cmd'))
        exceptions.append(dict(path=row['path'],buildSha256=sha(raw),finalSha256=sha(now),reason='Standalone launcher/record exporter only; no native Rust/test predicate changes, actualPS51 syntax separately checked'))
rows=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in sorted(paths)]
git('add','--',*sorted(paths));assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths);git('diff','--cached','--check')
out=git('commit','-m','test(windows): 接入隔离原包existing-root原生闭环验证通道');(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:json.dump(dict(head=head,tree=tree,files=rows,preserved=preserved,directedRun=run,directedPassed=int(m[1]),scriptOnlyChangesAfterNativeBuild=exceptions,publicStartOpen=False,actualProductionExecuted=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(head=head,files=len(paths),passed=int(m[1]),publicStartOpen=False)))
