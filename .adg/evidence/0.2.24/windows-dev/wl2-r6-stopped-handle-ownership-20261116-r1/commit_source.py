from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
assert git('rev-parse','HEAD').decode().strip()=='d29dbc36d567d4344407a07d652311f2bfa140d9'
assert git('diff','--cached','--name-only')==b''
assert load(ROOT/'library02/result.json')['exitCode']==0
log=(ROOT/'library02/stdout.log').read_text(encoding='utf-8-sig')
m=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',log);assert m
assert 'wl2_r6_capture_owned_tree_duplicate_transfer_never_replaces_or_drops_either_original_set ...' in log
assert 'wl2_r6_capture_owned_tree_transfer_retains_native_files_before_fallible_close_and_refuses_mutation ...' in log
scope=load(ROOT/'source-scope-final.json')['changedFiles'];paths=[r['path'] for r in scope];assert len(paths)==7
for r in scope:
    raw=(CODE/r['path']).read_bytes();assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['sha256']
git('add','--',*paths)
assert sorted(git('diff','--cached','--name-only').decode().splitlines())==sorted(paths)
git('diff','--cached','--check')
for p in paths:assert (CODE/p).read_bytes().replace(b'\r\n',b'\n')==git('show',f':{p}').replace(b'\r\n',b'\n')
out=git('commit','-m','feat(windows): 将原件所有权接入受限Stop关闭')
(ROOT/'source-commit.stdout.log').write_bytes(out)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
with (ROOT/'source-commit.json').open('x',encoding='utf-8') as f:
    json.dump(dict(head=head,tree=tree,files=paths,directedPassed=int(m[1]),actualProductionExecuted=False,publicStartOpen=False),f,indent=2)
print(json.dumps(dict(head=head,tree=tree,files=len(paths),directedPassed=int(m[1]))))
