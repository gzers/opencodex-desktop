from pathlib import Path
import hashlib,json,sys
ROOT=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
observations=[]
for run in sys.argv[1:]:
    for line in (ROOT/run/'stdout.log').read_text(encoding='utf-8-sig').splitlines():
        marker='r6-completed-history-native='
        if marker not in line:continue
        row=json.loads(line.split(marker,1)[1]);d=row['durable'];case=row['case'];directory=Path(d['directory'])
        raw=(directory/'current.json').read_bytes();assert len(raw)==d['currentActualBytes'] and sha(raw)==d['currentActualSha256']
        for event in d['immutableRecoveryEvents']+d['immutableLeaseEvents']+d['immutableHistoryEvents']+d['immutableHandleEvents']:
            assert json.loads((directory/f"event-{event['sequence']:06}.json").read_bytes())==event
        assert not row['typedClosed']
        if case in [80,81,83,84,85,87]:
            assert row['completion'] is None and row['freshReadRefused'] and d['poisoned']
            if case==87:
                assert d['originalHandleSummary']['released'] and d['originalNativeHandoff']['handles']['rows']
                assert 'timed out' in row['closeError'].lower()
        else:
            receipt=row['completion'];assert receipt['completedInOriginalWindow'] and row['originalDeadlineExpired']
            assert d['originalHandleSummary']['released']
            assert receipt['currentFileId'].isdigit() and len(receipt['stateDigest'])==64
            if case==0:assert row['freshReadSucceeded'] and row['lateMutationRefused']
            else:assert row['freshReadRefused'] and d['poisoned']
            if case==91:
                history=Path(d['immutableHistoryEvents'][1]['references'][0]['directory']);assert (history/'synthetic-completed-history-foreign').is_dir()
            client=Path(d['originalNativeHandoff']['roots'][0]['root'])/'config.toml'
            assert client.read_bytes()==b'synthetic next action live config after original cutoff'
        if case in [84,95]:assert raw==b'{'
        else:
            current=json.loads(raw);assert current['sequence']==d['sequence'] and current['handles']==d['originalHandleSummary']
        observations.append(dict(run=run,observation=row,offlineDataOnly=True))
assert observations
with (ROOT/('completed-native-traces-'+'-'.join(sys.argv[1:])+'.json')).open('x',encoding='utf-8') as out:json.dump(dict(observations=observations,nativeAuthority=False),out,indent=2)
print(json.dumps(dict(actualNativeCases=len(observations),cases=[r['observation']['case'] for r in observations],typedClosed=False)))
