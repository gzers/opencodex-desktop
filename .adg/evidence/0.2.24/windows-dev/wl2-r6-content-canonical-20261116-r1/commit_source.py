from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
assert git('rev-parse','HEAD').decode().strip()=='ed89291540e1d02c886c66020ebbbdc49c2cebeb'
assert git('diff','--cached','--name-only')==b''
assert load(ROOT/'library02/result.json')['exitCode']==0
assert '48 passed; 0 failed; 0 ignored' in (ROOT/'library02/stdout.log').read_text(encoding='utf-8-sig')
scope=load(ROOT/'source-scope-final.json')['changedFiles'];paths=[r['path'] for r in scope]
assert len(paths)==12 and not any('schemas' in p for p in paths)
for r in scope:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['sha256']
git('add','--',*paths)
assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths)
git('diff','--cached','--check')
for p in paths:assert (CODE/p).read_bytes().replace(b'\r\n',b'\n')==git('show',f':{p}').replace(b'\r\n',b'\n')
out=git('commit','-m','feat(windows): 将原内容资格写入同一canonical链')
(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:
    json.dump(dict(head=head,tree=tree,files=paths,directed48Pass=True,ordinaryProductionExecuted=False,publicStartOpen=False),f,indent=2)
print(json.dumps(dict(head=head,tree=tree,files=len(paths))))
