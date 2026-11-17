from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
assert git('rev-parse','HEAD').decode().strip()=='fcc7a688da39c6e289dd6478b919bbabfed2da45'
assert git('diff','--cached','--name-only')==b''
for run,rc,text in [('library01',101,'47 passed; 1 failed'),('library02',0,'48 passed; 0 failed'),('library03',0,'49 passed; 0 failed')]:
    assert load(ROOT/run/'result.json')['exitCode']==rc
    assert text in (ROOT/run/'stdout.log').read_text(encoding='utf-8-sig')
assert load(ROOT/'actual-observations.json')['productionRouteCloseIntegrated'] is False
scope=load(ROOT/'source-scope-verified.json')['changedFiles'];paths=[r['path'] for r in scope];assert len(paths)==6
for r in scope:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['sha256']
git('add','--',*paths)
assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths)
git('diff','--cached','--check')
for p in paths:assert (CODE/p).read_bytes().replace(b'\r\n',b'\n')==git('show',f':{p}').replace(b'\r\n',b'\n')
out=git('commit','-m','feat(windows): 保管并显式关闭原路由pin句柄')
(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:json.dump(dict(head=head,tree=tree,files=paths,finalDirectedPassed=49,actualProductionExecuted=False,canonicalRouteCloseIntegrated=False,startOpen=False),f,indent=2)
print(json.dumps(dict(head=head,tree=tree,files=len(paths),finalDirectedPassed=49)))
