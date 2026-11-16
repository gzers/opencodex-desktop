from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
text=(ROOT/'library02/stdout.log').read_text(encoding='utf-8-sig');rows=[];failed_tests=0
for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',text,re.S):
    if not body.rstrip().endswith('FAILED'):continue
    failed_tests+=1
    paths=[Path(p)/'session' for p in re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',body) if p.startswith('D:')];assert paths
    for i,path in enumerate(paths):
        events=[json.loads(p.read_bytes()) for p in sorted(path.glob('event-*.json'))];raw=(path/'current.json').read_bytes()
        rows.append(dict(test=name,directory=str(path),lastSceneOfFailedGroup=i==len(paths)-1,current=json.loads(raw),
            currentSha256=hashlib.sha256(raw).hexdigest(),lastImmutableEvent=events[-1],
            historyEvents=[e for e in events if e['documentKind']=='original-history-event'],
            handleEvents=[e for e in events if e['documentKind']=='original-handle-release-event']))
assert failed_tests==3
with (ROOT/'failure-prefixes-library02.json').open('x',encoding='utf-8') as out:json.dump(dict(rawExit=101,failedGroupedTests=failed_tests,
    scenes=rows,readOnlyAfterOriginalDeadline=True,noMutationOrCatchup=True),out,indent=2)
print(json.dumps(dict(failedGroupedTests=failed_tests,physicalScenes=len(rows),lastFailureKinds=[r['lastImmutableEvent']['operation'] for r in rows if r['lastSceneOfFailedGroup']])))
