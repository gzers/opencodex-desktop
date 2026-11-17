from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
sha=lambda b:hashlib.sha256(b).hexdigest()
assert git('rev-parse','HEAD').decode().strip()=='fb9a64ff79319383f015850d54903866bc6756d7'
assert git('diff','--cached','--name-only')==b''
assert load(ROOT/'library03/result.json')['exitCode']==0
log=(ROOT/'library03/stdout.log').read_text(encoding='utf-8-sig')
m=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',log);assert m
assert 'r6 Ready observation action windows retain actual' in log
scope=load(ROOT/'source-scope-final.json')['changedFiles']
paths=[r['path'] for r in scope];assert len(paths)==13
for r in scope:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and sha(raw)==r['sha256']
for r in load(ROOT/'library03/source-manifest.json'):
    raw=(ROOT/'library03/executed-source'/r['path']).read_bytes()
    assert len(raw)==r['bytes'] and sha(raw)==r['sha256'] and raw==(CODE/r['path']).read_bytes()
git('add','--',*paths)
assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths)
git('diff','--cached','--check')
for p in paths:assert (CODE/p).read_bytes().replace(b'\r\n',b'\n')==git('show',f':{p}').replace(b'\r\n',b'\n')
out=git('commit','-m','feat(windows): 保留v3完整前驱链并接入一次性后继捕获')
(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:
    json.dump(dict(head=head,tree=tree,files=paths,directedPassed=int(m[1]),actualProductionExecuted=False,actualSuccessorQualified=False,startOpen=False),f,indent=2)
print(json.dumps(dict(head=head,tree=tree,files=len(paths),directedPassed=int(m[1]))))
