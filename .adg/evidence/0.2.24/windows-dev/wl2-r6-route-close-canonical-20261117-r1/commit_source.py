from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
assert git('rev-parse','HEAD').decode().strip()=='b8e06be268182c517a19706dd974961b7e26e57a'
assert git('diff','--cached','--name-only')==b''
results={}
for name,profile in [('library01','debug'),('library02','debug'),('library03','release')]:
    r=load(ROOT/name/'result.json');assert r['exitCode']==0 and r['profile']==profile
    log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',log);assert m;results[name]=int(m[1])
assert results['library02']==56
assert load(ROOT/'actual-route-observations.json')['canonicalActualFactoryExecuted'] is False
scope=load(ROOT/'source-scope-final.json')['changedFiles'];paths=[r['path'] for r in scope];assert len(paths)==15
for r in scope:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['sha256']
git('add','--',*paths);assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths);git('diff','--cached','--check')
for p in paths:assert (CODE/p).read_bytes().replace(b'\r\n',b'\n')==git('show',f':{p}').replace(b'\r\n',b'\n')
out=git('commit','-m','feat(windows): 将原路由句柄关闭接入最终同链证明');(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:json.dump(dict(head=head,tree=tree,files=paths,directedPassed=results,actualProductionExecuted=False,typedClosedQualified=False,startOpen=False),f,indent=2)
print(json.dumps(dict(head=head,tree=tree,files=len(paths),directedPassed=results)))
