from pathlib import Path
import hashlib,json,re,sys
ROOT=Path(__file__).resolve().parent
run=sys.argv[1];text=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig');rows=[];failed=0
for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',text,re.S):
    if not body.rstrip().endswith('FAILED'):continue
    failed+=1
    paths=[Path(p)/'session' for p in re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',body) if p.startswith('D:')];assert paths
    for i,directory in enumerate(paths):
        raw=(directory/'current.json').read_bytes();events=[json.loads(p.read_bytes()) for p in sorted(directory.glob('event-*.json'))]
        assert events
        rows.append(dict(test=name,directory=str(directory),lastSceneInFailedGroup=i==len(paths)-1,
            current=json.loads(raw) if raw!=b'{' else None,currentRawBytes=len(raw),currentSha256=hashlib.sha256(raw).hexdigest(),lastImmutableEvent=events[-1],
            phaseCounts={kind:sum(e['documentKind']==kind for e in events) for kind in ['bounded-node-batch-event','original-history-event','original-handle-release-event']},
            nativeReceiptClaimed=False))
assert failed
with (ROOT/f'budget-prefixes-{run}.json').open('x',encoding='utf-8') as out:json.dump(dict(run=run,rawExit=101,failedGroupedTests=failed,
    scenes=rows,readOnlyForensics=True,noMutationOrCatchup=True),out,indent=2)
print(json.dumps(dict(failedGroups=failed,scenes=len(rows),lastOperations=[r['lastImmutableEvent'].get('operation') for r in rows if r['lastSceneInFailedGroup']])))
