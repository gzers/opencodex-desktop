import copy
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

verifier, observed, evidence = map(Path, sys.argv[1:])
runtime_bytes = (observed / 'Host-image-created-verified.json').read_bytes()
runtime = json.loads(runtime_bytes)
preflight = json.loads((observed / 'Host-fixture-preflight.json').read_bytes())
creation = json.loads((observed / 'Host-creation.json').read_bytes())
intent = json.loads((observed / 'Host-intent.json').read_bytes())

# Only the proof envelope and timings are synthetic. The setup report comes from nine-r1.
creation['image'] = creation['image'].replace('nine-r1', 'nine-r2')
new_hash = 'ab282752940b875b98e5e957a102ee8f4b1d32829f7524eb2dbe45959defc030'
preflight['imageSha256'] = intent['testSha256'] = new_hash
args = ['--sid', preflight['sid'], '--session', str(preflight['session']),
        '--profile', preflight['profile'], '--image', creation['image'], '--image-sha256', new_hash]
uid = '12345678-0000-4000-8000-123456789abc'
digest = 'a' * 64
worker = dict(thread='ThreadId(1)', actionId=uid, sessionId=uid, scopeDigest=digest,
              rawError=31, phase='FailedRetained', creatorResumed=False)
injection = dict(stage='captured-both-trees-before-first-journal-intent', injected=True,
                 nativeAuthority=False, raw=31, manifestIntentAndFileAbsent=True,
                 currentIntentAndFileAbsent=True, originalCreatorNeverResumed=True,
                 workerThread=worker['thread'], failedActionId=uid, ownerSessionId=uid,
                 scopeDigest=digest, originalClaimAndNlkVerified=dict(epoch=uid, claimKey=digest, objectsDigest=digest))
refusal = dict(publicStartOpen=False, normalStop=False, closed=False)
record = dict(**refusal, stage='initial-start', success=False, budgetMs=30000, elapsedMs=100,
              statusBudgetMs=5000, statusElapsedMs=1, firstError='synthetic raw31', statusError=None,
              injection=injection, sameSampleStatus=dict(sameSample=True, recovery='Required',
              payload=dict(startWorker=worker)))
common = dict(pid=creation['pid'], creationFiletime=creation['creationFiletime'],
              reachedOriginalInjection=True, privateEvidenceComplete=True, initialScansVerified=True)
deadline = 30_000_000_000
clock = 0
reports = []
for phase, operations in [('start-entry', ['launch-plan-prepare', 'launch-plan', 'windows-read-complete']),
                          ('start-worker', ['source-capture'] + ['source-readback'] * 5)]:
    scans = []
    for ordinal, operation in enumerate(operations, 1):
        end = clock + 1_000_000
        scans.append(dict(ordinal=ordinal, operation=operation, completed=True, firstError=None,
                          startedNs=clock, finishedNs=end, elapsedUs=1000,
                          remainingBeforeUs=(deadline-clock)//1000, remainingAfterUs=(deadline-end)//1000))
        clock = end
    reports.insert(0, dict(phase=phase, deadlineNs=deadline, scans=scans))


def encode(value):
    return json.dumps(value).encode()


def run(name, candidate, expected, reason=None, executable=verifier):
    path = candidate if isinstance(candidate, Path) else root / name
    if not isinstance(candidate, Path):
        path.mkdir()
        exports = {'Host-fixture-preflight.json': encode(preflight),
                   'Host-fixture-result.json': encode(record),
                   'Host-image-created-verified.json': encode(candidate)}
        rows = [dict(name=key, bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())
                for key, raw in exports.items()]
        files = dict(exports, **{
            'Host-creation.json': encode(creation), 'Host-intent.json': encode(intent),
            'Host-ready.json': encode(dict(**common, nativeResult=record)),
            'Host-result.json': encode(dict(**common, **refusal, mode='Host', passed=True,
                                       privateEvidence=rows, initialScans=reports)),
            'Host.stdout.log': b''.join(b'r7-initial-image-scans=' + encode(row) + b'\n' for row in reports)
                               + b'r7-partial-capture=' + encode(record) + b'\n'})
        for filename, raw in files.items():
            (path / filename).write_bytes(raw)
    result = subprocess.run([sys.executable, '-B', str(executable), str(path), *args],
                            capture_output=True, text=True, encoding='utf-8', env=env)
    print(json.dumps(dict(case=name, raw=result.returncode, stdout=result.stdout, stderr=result.stderr)))
    assert result.returncode == expected and (reason is None or reason in result.stderr), name


env = dict(os.environ, PYTHONUTF8='1')
with tempfile.TemporaryDirectory(prefix='ocx-r7-runtime-report-') as tmp:
    root = Path(tmp)
    prior = root / 'prior-verifier.py'
    prior.write_bytes(subprocess.check_output(['git', 'show', 'HEAD:test/windows024-native/verify-r7-host-receipts.py']))
    run('synthetic-envelope-original-runtime-report', runtime, 0)
    for name, field, value in [
        ('wrong-bundle', ('bundleSha256',), digest),
        ('missing-readback', ('sharedFullImageReadback',), None),
        ('false-readback', ('sharedFullImageReadback',), False),
        ('wrong-current', ('current', 'imageSha256'), digest),
        ('altered-node-digest', ('image', 'nodeSha256'), digest),
        ('extra-image-field', ('image', 'extra'), True),
    ]:
        candidate = copy.deepcopy(runtime)
        obj = candidate if len(field) == 1 else candidate[field[0]]
        if value is None:
            del obj[field[-1]]
        else:
            obj[field[-1]] = value
        reason = 'manifest fields differ' if name == 'extra-image-field' else (
            'sharedFullImageReadback' if value is None else 'full readback differs')
        run(name, candidate, 1, reason)
    run('prior-verifier-reproduces-gap', root / 'altered-node-digest', 0, executable=prior)
    run('real-nine-r1-still-refused', observed, 1, 'os error 32')
assert not root.exists()
print(json.dumps(dict(temporaryRoot=str(root), absent=True)))