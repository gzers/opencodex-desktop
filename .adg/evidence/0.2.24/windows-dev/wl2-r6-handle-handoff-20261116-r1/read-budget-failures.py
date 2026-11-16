from pathlib import Path
import hashlib,json,re,sys
ROOT=Path(__file__).resolve().parent
run=sys.argv[1] if len(sys.argv)>1 else 'library02'
rows=[];failed_tests=0;text=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig')
for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',text,re.S):
    if not body.rstrip().endswith('FAILED'):continue
    failed_tests+=1
    paths=re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',body);data=[Path(p)/'session' for p in paths if p.startswith('D:')];assert data
    for i,directory in enumerate(data):
        events=[json.loads(p.read_bytes()) for p in sorted(directory.glob('event-*.json'))];raw=(directory/'current.json').read_bytes()
        rows.append(dict(test=name,directory=str(directory),lastSceneInFailedGroupedTest=i==len(data)-1,current=json.loads(raw) if raw!=b'{' else None,currentSha256=hashlib.sha256(raw).hexdigest(),lastEvent=events[-1],
            historyEvents=[e for e in events if e['documentKind']=='original-history-event'],handleEvents=[e for e in events if e['documentKind']=='original-handle-release-event']))
assert failed_tests==2
with (ROOT/f'budget-failure-forensics-{run}.json').open('x',encoding='utf-8') as out:json.dump(dict(run=run,rawExit=101,observations=rows,readOnly=True,noCatchupOrMutation=True),out,indent=2)
print(json.dumps(dict(failures=len(rows),handleEvents=[len(r['handleEvents']) for r in rows],historyLinked=[any(e['operation']=='OriginalHistoryLinkedVerified' for e in r['historyEvents']) for r in rows])))
