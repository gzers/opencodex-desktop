from pathlib import Path
import hashlib
import json

root=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding="utf-8-sig"))
sha=lambda raw:hashlib.sha256(raw).hexdigest()
manifest=load(root/'payload-manifest.json')
assert manifest['productionAcceptance'] is False
for row in manifest['files']:
    path=root/row['path']
    assert path.resolve().is_relative_to(root.resolve())
    raw=path.read_bytes()
    assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
status=load(root/'final-status.json')
receipt=load(root/'received/archive-cleanup-result.json')
assert receipt['complete'] is True and receipt['removedSuccessfulProfileChild'] is True
assert receipt['targets'][0]['sourceRetained'] is False and receipt['targets'][1]['sourceRetained'] is True
assert receipt['oldRejectedPathsRetried'] is False
assert sha((root/'received/archive-cleanup-result.json').read_bytes())==status['cleanup']['receiptSha256']
assert status['controllerPublicVerification']==dict(readable=12,unreadable=8,complete=False,deleted=False,aclChanged=False)
assert status['sameSidCleanupProcessRawExit'] is None and receipt['brokerRawExit'] is None
assert status['allAccepted'] is False and status['productionExecuted'] is False and status['startOpen'] is False
assert status['cleanup']['overallCleanupComplete'] is False
assert load(root/'path-observation01-invalid.json')['usableAsAbsenceEvidence'] is False
print(json.dumps(dict(payloads=len(manifest['files']),integrity='verified',productionAcceptance=False)))
