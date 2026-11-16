from pathlib import Path
import hashlib,json,subprocess,os,uuid
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
rows=load(ROOT/'source-scope.json')['changedFiles']
target=(CODE/'test/windows024-native/target').resolve();index=target/f'completed-history-review-index-{uuid.uuid4()}.bin'
assert index.parent==target and not index.exists()
env=dict(os.environ,GIT_INDEX_FILE=str(index))
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a],env=env)
head=git('rev-parse','HEAD').decode().strip();assert head=='d20103fee084a3e63f728bf7c6664af65405e456'
try:
    git('read-tree','HEAD');git('add','--',*[r['path'] for r in rows]);tree=git('write-tree').decode().strip()
    changed=git('diff-tree','--no-commit-id','--name-only','-r','HEAD^{tree}',tree).decode().splitlines()
    assert sorted(changed)==sorted(r['path'] for r in rows)
    for row in rows:
        raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        blob=git('show',f"{tree}:{row['path']}");assert raw.replace(b'\r\n',b'\n')==blob.replace(b'\r\n',b'\n')
    patch=git('diff','--binary','HEAD^{tree}',tree)
    with (ROOT/'candidate.patch').open('xb') as out:out.write(patch)
finally:
    if index.exists():index.unlink()
with (ROOT/'candidate-tree.json').open('x',encoding='utf-8') as out:
    json.dump(dict(baseCommit=head,localCandidateTree=tree,isSourceCommit=False,sourceBranchUncommitted=True,sourceCandidatePushed=False,
        sourceScope=rows,patch='candidate.patch',patchSha256=sha(patch),normalIndexUnmodified=True,temporaryIndexRemoved=not index.exists(),
        budgetRegressionPassed=False,productionStartOpen=False),out,indent=2)
print(json.dumps(dict(baseCommit=head,localCandidateTree=tree,files=len(rows),sourceCommitCreated=False,normalIndexUnmodified=True)))
