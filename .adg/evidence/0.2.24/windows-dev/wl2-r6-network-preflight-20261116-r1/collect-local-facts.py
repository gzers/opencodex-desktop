from pathlib import Path
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor
import subprocess,json,hashlib
ROOT=Path(__file__).resolve().parent
DOCS=ROOT.parents[4]
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
sha=lambda b:hashlib.sha256(b).hexdigest()
jobs=[('source-head',CODE,['rev-parse','HEAD']),('source-tree',CODE,['rev-parse','HEAD^{tree}']),
      ('source-status',CODE,['status','--porcelain=v1','--untracked-files=all']),
      ('source-main-cached',CODE,['rev-parse','origin/main']),('governance-main-cached',CODE,['rev-parse','origin/docs/governance-main']),
      ('docs-head-before-local-evidence',DOCS,['rev-parse','HEAD']),('docs-tree-before-local-evidence',DOCS,['rev-parse','HEAD^{tree}']),
      ('docs-status-before-local-evidence',DOCS,['status','--porcelain=v1','--untracked-files=all'])]
def run(job):
    name,repo,args=job;result=subprocess.run(['git','-C',str(repo),*args],capture_output=True)
    for stream,raw in [('stdout',result.stdout),('stderr',result.stderr)]:
        with (ROOT/f'{name}.{stream}.log').open('xb') as out:out.write(raw)
    assert result.returncode==0,(name,result.returncode,result.stderr)
    return name,dict(command=['git','-C',str(repo),*args],rawExit=result.returncode,stdoutSha256=sha(result.stdout),stderrSha256=sha(result.stderr),stdout=result.stdout.decode('utf-8').strip())
with ThreadPoolExecutor(max_workers=4) as pool:observations=dict(pool.map(run,jobs))
assert observations['source-head']['stdout']=='d20103fee084a3e63f728bf7c6664af65405e456'
assert observations['source-status']['stdout']=='?? apps/desktop/tauri/gen/schemas/windows-schema.json'
assert observations['docs-head-before-local-evidence']['stdout']=='7d0d567e19b7b3cbc799dcd85ec3a43ad4fca0f4'
assert observations['docs-status-before-local-evidence']['stdout']==''
artifacts=json.loads((ROOT.parent/'wl2-r6-handle-handoff-20261116-r1/product01-artifacts.json').read_text(encoding='utf-8-sig'))
for row in artifacts:
    path=Path(row['path']);raw=path.read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
contracts=[]
for rel in ['docs/03-开发实施/TEAM-0.2.24.md','docs/03-开发实施/0.2.24-contracts/windows-claim-anchored-custody-r6.md']:
    raw=(DOCS/rel).read_bytes();contracts.append(dict(path=rel,bytes=len(raw),sha256=sha(raw),localOnly=True))
with (ROOT/'local-facts.json').open('x',encoding='utf-8') as out:
    json.dump(dict(time=datetime.now().astimezone().isoformat(),observations=observations,artifacts=artifacts,localContractFiles=contracts,
        remoteFreshnessConfirmed=False,sourceModified=False,testsRerun=False,oldCleanupAttempted=False,privateNativeReceiptOrClosedMinted=False),out,ensure_ascii=False,indent=2)
print(json.dumps(dict(localQueries=len(observations),productArtifactsStillMatch=len(artifacts),sourceCleanExceptPreexistingSchema=True,remoteFreshnessConfirmed=False)))
