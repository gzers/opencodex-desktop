from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code');BASE='c870bd81fcbd65310ff79d262c9c49fc5a2c6f62'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a]);load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest()
assert load(ROOT/'library01/result.json')['exitCode']==0
log=(ROOT/'library01/stdout.log').read_text(encoding='utf-8-sig');assert '9 passed; 0 failed; 0 ignored' in log
assert 'old prefix source returns raw2' in log and 'five original diagnostic role callsite source hashes' in log
assert not git('diff','--cached','--name-only').strip()
paths=git('diff','--name-only').decode().splitlines()
expected=['apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/content.rs','apps/desktop/tauri/src/infrastructure/windows_diagnostic_roles024.rs','test/windows024-native/build-wl2.ps1','test/windows024-native/launch-r6-official-existing.ps1']
assert sorted(paths)==sorted(expected),paths
for row in load(ROOT/'library01/source-manifest.json'):
    raw=(ROOT/'library01/executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
    if row['path'] in paths:assert raw==(CODE/row['path']).read_bytes()
preserved=[]
for p in ['apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs','apps/desktop/tauri/src/infrastructure/official_runtime024_transport.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/source.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/adapter.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/capture.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/journal/closed_stop.rs','apps/desktop/tauri/src/infrastructure/windows_supervisor024.rs']:
    raw=(CODE/p).read_bytes().replace(b'\r\n',b'\n');assert raw==git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');preserved.append(dict(path=p,normalizedSha256=sha(raw)))
p=expected[0];old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n');new=(CODE/p).read_text(encoding='utf-8-sig');marker='pub(crate) fn recovery_state_at('
assert old[old.index(marker):]==new[new.index(marker):]
assert 'super::windows_client_originals024::require_durable_capture()' in new[new.index('async fn execute_async('):new.index('async fn execute_admitted(')]
preserved.append(dict(path=p,section='complete recovery_state_at/StatusReceipt tail',normalizedSha256=sha(new[new.index(marker):].encode())))
# Fixed source-digest literals are unchanged; only paths are corrected.
for p in expected[1:3]:
    a=git('show',f'{BASE}:{p}').decode();b=(CODE/p).read_text(encoding='utf-8-sig')
    before=re.findall(r'"([0-9a-f]{64})"',a);after=re.findall(r'"([0-9a-f]{64})"',b)
    assert before==after,(p,before,after)
rows=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in paths]
git('add','--',*paths);git('diff','--cached','--check');assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths)
out=git('commit','-m','fix(windows): 按实际包目录绑定审计与诊断源码');(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:json.dump(dict(head=head,tree=tree,files=rows,preserved=preserved,fixedSourceSHALiteralsUnchanged=True,actualNativeSourcePathTestsPassed=9,failedOriginalSource=BASE,actualProductionExecuted=False,startOpen=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(head=head,tree=tree,files=len(paths),passed=9)))
