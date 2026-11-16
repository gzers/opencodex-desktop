from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
def git(*args):return subprocess.check_output(['git','-C',str(CODE),*args])
assert git('rev-parse','HEAD').decode().strip()=='d20103fee084a3e63f728bf7c6664af65405e456'
assert git('diff','--cached','--name-only')==b''
result=load(ROOT/'library09/result.json');assert result['exitCode']==0
assert '44 passed; 0 failed' in (ROOT/'library09/stdout.log').read_text(encoding='utf-8-sig')
assert len(load(ROOT/'native-traces-library09.json')['observations'])==32
assert len(load(ROOT/'completed-native-traces-library09.json')['observations'])==13
rows=load(ROOT/'source-scope.json')['changedFiles'];paths=[r['path'] for r in rows]
assert len(paths)==31 and len(set(paths))==len(paths)
assert 'apps/desktop/tauri/gen/schemas/windows-schema.json' not in paths
for row in rows:
    raw=(CODE/row['path']).read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
git('add','--',*paths)
assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths)
git('diff','--cached','--check')
for row in rows:
    blob=git('show',f":{row['path']}");raw=(CODE/row['path']).read_bytes()
    assert raw.replace(b'\r\n',b'\n')==blob.replace(b'\r\n',b'\n')
output=git('commit','-m','feat(windows): 批量登记准确节点并验证原窗完成历史')
with (ROOT/'source-commit.stdout.log').open('xb') as out:out.write(output)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as out:json.dump(dict(head=head,tree=tree,exactStagedFiles=paths,
    affectedNative44Pass=True,batchPhysical32Cases=True,completedHistory13Cases=True,gitLineEndingEquivalent=True,
    preexistingSchemaNotStaged=True,pushed=False),out,indent=2)
print(json.dumps(dict(head=head,tree=tree,files=len(paths),preexistingSchemaNotStaged=True)))
