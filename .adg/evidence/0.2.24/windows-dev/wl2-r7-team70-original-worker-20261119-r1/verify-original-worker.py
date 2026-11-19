import hashlib
import json
import re
from datetime import datetime
from pathlib import Path

src = Path('C:/Users/Public/ocx-r7-isolated-owner-range-r1/out01')
prior = Path('.adg/evidence/0.2.24/windows-dev/wl2-r7-range-host-20261119-r1')

def read(path):
    return json.loads(path.read_bytes().decode('utf-8-sig'))

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def check(condition, description):
    if not condition:
        raise ValueError(description)

host_sha = '97d40caf5661aec2c16464d32d7ec4cbbff608d4e2d2b0ee757972c3fbac7346'
for name in ('Host-result.json', 'Host-ready.json', 'Host-creation.json', 'Host-intent.json'):
    check(digest(src / name) == digest(prior / name), f'{name} differs from accepted archive')
check(digest(prior / 'Host-result.json') == host_sha, 'Host SHA differs from TEAM70')
host = read(prior / 'Host-result.json')
creation = read(prior / 'Host-creation.json')
host_intent = read(prior / 'Host-intent.json')
ready = read(prior / 'Host-ready.json')
injection = ready['nativeResult']['injection']
live = read(src / 'Host-live-independent.json')
diag = read(src / 'DiagnosticBefore-result.json')
diag_host = read(src / 'DiagnosticBefore-original-host.json')
diag_intent = read(src / 'DiagnosticBefore-intent.json')
diag_creation = read(src / 'DiagnosticBefore-creation.json')
check(digest(src / 'Host-live-independent.json') == '5b38a25e0025d883e304b02f216eb57415337770c3516559e71fc38c95024978', 'Live receipt SHA drift')
check(digest(src / 'DiagnosticBefore-result.json') == '677d8e9f2d1ceec235837c69a1ddffd5d8023c89e5f464ae5f79843d24dd6977', 'Diagnostic receipt SHA drift')
check(live['raw'] == 0 and live['firstError'] is None and live['liveProcessVerified'] is True and live['facts']['exited'] is False and live['originalRangeContinuityVerified'] is False, 'Live observer status differs')
check(live['hostResultSha256'] == host_sha and live['observerSid'] == host_intent['sid'] == diag_intent['sid'] and live['observerSession'] == host_intent['session'] == diag_intent['session'] == 7, 'Observer identity or binding differs')
check(datetime.fromisoformat(live['queryStartedUtc']) <= datetime.fromisoformat(live['queryFinishedUtc']), 'Observer timestamps invalid')
check(live['facts']['pid'] == host['pid'] == ready['pid'] == creation['pid'] == diag_host['pid'] == 25144 and live['facts']['filetime'] == host['creationFiletime'] == ready['creationFiletime'] == creation['creationFiletime'] == diag_host['creationFiletime'] == '134395302283231479', 'Original process generation differs')
check(live['facts']['image'] == creation['image'] == diag_host['image'] == diag_creation['image'] and live['facts']['session'] == diag_host['session'] == 7, 'Original process image or session differs')
check(diag['mode'] == diag_intent['mode'] == 'DiagnosticBefore' and diag_intent['nativeStageReady'] is True and diag_intent['testSha256'] == host_intent['testSha256'] and diag['pid'] == diag_creation['pid'] and diag['creationFiletime'] == diag_creation['creationFiletime'], 'Diagnostic helper run differs')
log = (src / 'DiagnosticBefore.stdout.log').read_bytes().decode('utf-8-sig')
rows = re.findall(r'r7-partial-manager=(\{[^\r\n]+\})', log)
check(len(rows) == 1 and json.loads(rows[0]) == diag['manager'], 'Raw manager result differs')
check('test result: ok. 1 passed; 0 failed;' in log and (src / 'DiagnosticBefore.stderr.log').stat().st_size == 0, 'Rust test or stderr differs')
check(diag['raw'] == 0 and diag['passed'] is True and diag.get('firstError') is None and 0 <= diag['manager']['elapsedMs'] <= 5000, 'Original 5-second diagnostic failed')
transport = diag['manager']['result']
reply = transport['response']
check(transport['operation'] == 'read-worker-diagnostic' and transport['refused'] is False and reply['error'] is None and reply['purpose'] == 'worker-diagnostic', 'Diagnostic transport differs')
for result in (diag, reply):
    check(all(result[key] is False for key in ('publicStartOpen', 'normalStop', 'closed')), 'Unexpected lifecycle authority')
facts = reply['facts']
check(facts['runtimeAuthority'] is False and facts['recoveryRequired'] is True and reply['hostMayExit'] is False, 'Expected retained state differs')
status = facts['sameSampleStatus']
worker = facts['snapshot']
proof = facts['originalRangeProof']
check(status['sameSample'] is True and status['recovery'] == 'Required' and status['payload']['startWorker'] == worker and status['payload'].get('recoveryWorker') is None, 'Same sample is not the original retained worker')
check(worker['phase'] == 'FailedRetained' and worker['stage'] == 'Capture' and worker['rawError'] == proof['startRawError'] == 31 and worker['originalDeadlineExpired'] is True and worker['creatorResumed'] is proof['creatorResumed'] is False and injection['originalCreatorNeverResumed'] is True, 'Original failure state differs')
check(worker['thread'] == proof['workerThread'] == injection['workerThread'] and worker['actionId'] == proof['failedActionId'] == injection['failedActionId'] and worker['sessionId'] == proof['sessionId'] == injection['ownerSessionId'] and worker['scopeDigest'] == proof['scopeDigest'] == injection['scopeDigest'] and proof['hostInstanceId'] == injection['hostInstanceId'], 'Original worker binding differs')
check(proof['originalRanges'] == injection['originalClaimAndNlkVerified'], 'Original claim/N/L/K fingerprint differs')
check(proof['requestId'] == reply['requestId'] and len({proof['requestId'], proof['transportActionId'], proof['failedActionId']}) == 3, 'Request identity differs')
print(json.dumps(dict(passed=True, hostResultSha256=host_sha, liveReceiptSha256=digest(src / 'Host-live-independent.json'), diagnosticResultSha256=digest(src / 'DiagnosticBefore-result.json'), diagnosticElapsedMs=diag['manager']['elapsedMs'], diagnosticBudgetMs=5000, diagnosticRaw=diag['raw'], originalStartRaw=worker['rawError'], hostPid=host['pid'], hostFiletime=host['creationFiletime'], hostSession=7, originalWorker=worker['thread'], originalAction=worker['actionId'], firstError=injection.get('firstError', ready['nativeResult']['firstError']), ranges=proof['originalRanges'], rawManagerEqual=True, rustTestPassed=True, runtimeAuthority=False, originalRangeProofAccepted=True), indent=2, ensure_ascii=True))
