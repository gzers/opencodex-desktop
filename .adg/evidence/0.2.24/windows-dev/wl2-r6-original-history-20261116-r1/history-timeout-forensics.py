from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
text=(ROOT/'library02/stdout.log').read_text(encoding='utf-8-sig')
rows=[]
for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',text,re.S):
    if not body.rstrip().endswith('FAILED'):continue
    paths=re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',body)
    directories=[Path(p)/'session' for p in paths if p.startswith('D:')];assert len(directories)==1
    directory=directories[0];events=[json.loads(p.read_bytes()) for p in sorted(directory.glob('event-*.json'))]
    current=json.loads((directory/'current.json').read_bytes());history=[e for e in events if e.get('documentKind')=='original-history-event']
    recovery=[e for e in events if e.get('documentKind')=='stopped-recovery-event']
    assert recovery and history
    roots=recovery[0]['plan']['roots'];archives=[]
    for i,root in enumerate(roots):
        profile=Path(root['originalTree']['root']).parent;leaf='codex' if i==0 else 'grok'
        path=profile/f".ocxd024-history-{history[0]['context']['sessionId']}-{leaf}"
        archives.append(dict(path=str(path),exists=path.exists(),metadataPresent=(path/'snapshot.json').exists(),
            members=[dict(path=p.name,bytes=p.stat().st_size,sha256=sha(p.read_bytes())) for p in sorted(path.iterdir()) if p.is_file()] if path.exists() else []))
    rows.append(dict(test=name,directory=str(directory),current=current,historyEvents=history,lastEvent=events[-1],archives=archives,
        timings=re.findall(r'r6 recovery native canonical component phase=([^\r\n]+)',body)))
assert len(rows)==2
with (ROOT/'history-timeout-forensics.json').open('x',encoding='utf-8') as out:
    json.dump(dict(rawExit=101,passed=4,failed=2,observations=rows,readOnlyForensics=True,originalDeadlineUnchanged=True,noMutationOrCatchup=True),out,indent=2)
print(json.dumps(dict(failures=len(rows),historyLinked=[any(e['operation']=='OriginalHistoryLinkedVerified' for e in r['historyEvents']) for r in rows],mutated=False)))
