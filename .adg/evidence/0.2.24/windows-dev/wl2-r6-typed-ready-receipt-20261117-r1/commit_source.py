from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
assert git('rev-parse','HEAD').decode().strip()=='df0c8c24fdb1e5394290cecc28a4bf68f815dbc4';assert git('diff','--cached','--name-only')==b''
r=load(ROOT/'library01/result.json');assert r['exitCode']==0
log=(ROOT/'library01/stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',log);assert m
assert 'r6 retained actual-admitted Origin metadata' in log
scope=load(ROOT/'source-scope.json')['changedFiles'];paths=[r['path'] for r in scope];assert len(paths)==3
for r in scope:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['sha256']
git('add','--',*paths);assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths);git('diff','--cached','--check')
for p in paths:assert (CODE/p).read_bytes().replace(b'\r\n',b'\n')==git('show',f':{p}').replace(b'\r\n',b'\n')
out=git('commit','-m','feat(windows): 保留原typed Ready观察与只读来源身份');(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:json.dump(dict(head=head,tree=tree,files=paths,directedPassed=int(m[1]),actualTypedObservationFactoryExecuted=False,startOpen=False),f,indent=2)
print(json.dumps(dict(head=head,tree=tree,files=len(paths),directedPassed=int(m[1]))))
