from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
assert git('rev-parse','HEAD').decode().strip()=='502858eb56f1a5401684259e7420bc0712c6cb4f'
assert git('diff','--cached','--name-only')==b''
results={}
for name,profile in [('library01','debug'),('library02','release')]:
    r=load(ROOT/name/'result.json');assert r['exitCode']==0 and r['profile']==profile
    log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
    m=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',log);assert m
    results[name]=int(m[1])
scope=load(ROOT/'source-scope.json')['changedFiles'];paths=[r['path'] for r in scope];assert len(paths)==7
for r in scope:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['sha256']
git('add','--',*paths)
assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths)
git('diff','--cached','--check')
for p in paths:assert (CODE/p).read_bytes().replace(b'\r\n',b'\n')==git('show',f':{p}').replace(b'\r\n',b'\n')
out=git('commit','-m','feat(windows): 独立核对全部原件关闭句柄清单')
(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:
    json.dump(dict(head=head,tree=tree,files=paths,directedPassed=results,ordinaryProductionExecuted=False,publicStartOpen=False),f,indent=2)
print(json.dumps(dict(head=head,tree=tree,files=len(paths),directedPassed=results)))
