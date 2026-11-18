"""Check exported Host evidence before issuing any diagnostic request."""
import argparse
import hashlib
import json
import re
from pathlib import Path, PureWindowsPath


EXPORTS = {
    'Host-fixture-preflight.json',
    'Host-fixture-result.json',
    'Host-image-created-verified.json',
}
HEX = r'[0-9a-f]{64}'
UUID = r'[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unique_fields(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f'duplicate JSON field: {key}')
        result[key] = value
    return result


def decode(raw):
    return json.loads(raw, object_pairs_hook=unique_fields)


def read(root, name):
    with (root / name).open('rb') as source:
        raw = source.read(1048577)
    require(len(raw) <= 1048576, f'oversized receipt: {name}')
    return raw


def verify_scans(reports, elapsed_ms):
    require([report['phase'] for report in reports] == ['start-worker', 'start-entry'],
            'expected one worker report followed by one entry report')
    deadline = reports[1]['deadlineNs']
    require(type(deadline) is int and 0 < deadline <= 30_000_000_000
            and reports[0]['deadlineNs'] == deadline, 'Start deadline differs')
    prior = 0
    operations = (
        ('launch-plan-prepare', 'launch-plan', 'windows-read-complete'),
        ('source-capture',) + ('source-readback',) * 5,
    )
    for report, expected in zip(reversed(reports), operations):
        require([scan['operation'] for scan in report['scans']] == list(expected),
                'full scan sequence differs')
        for ordinal, scan in enumerate(report['scans'], 1):
            started, finished = scan['startedNs'], scan['finishedNs']
            require(scan['ordinal'] == ordinal and scan['completed'] is True
                    and scan['firstError'] is None and 0 <= prior <= started < finished < deadline
                    and (deadline - started) // 1000 == scan['remainingBeforeUs']
                    and (deadline - finished) // 1000 == scan['remainingAfterUs'] > 0
                    and (finished - started) // 1000 == scan['elapsedUs'] > 0,
                    'incomplete, out-of-order or expired full scan')
            prior = finished
    require(prior < (elapsed_ms + 1) * 1_000_000, 'last scan follows the Start receipt')


def verify(root, sid, session, profile, image, image_sha):
    result = decode(read(root, 'Host-result.json'))
    require(result['mode'] == 'Host' and result['passed'] is True,
            f"Host refused: {result.get('firstError')}")
    ready = decode(read(root, 'Host-ready.json'))
    creation = decode(read(root, 'Host-creation.json'))
    intent = decode(read(root, 'Host-intent.json'))
    rows = result['privateEvidence']
    require(len(rows) == len(EXPORTS) and {row['name'] for row in rows} == EXPORTS,
            'private export set differs')
    exports = {}
    for row in rows:
        raw = read(root, row['name'])
        require(len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256'],
                f"private export length or digest differs: {row['name']}")
        exports[row['name']] = decode(raw)
    preflight = exports['Host-fixture-preflight.json']
    record = exports['Host-fixture-result.json']
    require(intent['mode'] == 'Host' and intent['nativeStageReady'] is True
            and intent['sid'] == preflight['sid'] == sid
            and intent['session'] == preflight['session'] == session
            and PureWindowsPath(preflight['profile']) == profile
            and preflight['desktop'] == 'WinSta0/Default' and preflight['elevated'] is False
            and preflight['outerJobAbsent'] is True
            and intent['testSha256'] == preflight['imageSha256'] == image_sha
            and PureWindowsPath(creation['image']) == image, 'Host provenance differs')
    require(all(creation[key] == preflight[key] == ready[key] == result[key]
                for key in ('pid', 'creationFiletime')), 'Host process receipts disagree')
    require(ready['nativeResult'] == record and all(
        result[key] is True and ready[key] is True for key in (
            'reachedOriginalInjection', 'privateEvidenceComplete', 'initialScansVerified')),
        'Host and native receipts disagree')
    require(all(row['publicStartOpen'] is False for row in (result, intent, record))
            and all(row[key] is False for row in (result, record) for key in ('normalStop', 'closed'))
            and record['stage'] == 'initial-start' and record['success'] is False
            and record['budgetMs'] == 30000 and 0 <= record['elapsedMs'] <= 30000
            and record['statusBudgetMs'] == 5000 and 0 <= record['statusElapsedMs'] <= 5000
            and record['firstError'] and record['statusError'] is None, 'native action result differs')
    injection = record['injection']
    status = record['sameSampleStatus']
    worker = status['payload']['startWorker']
    require(status['sameSample'] is True and status['recovery'] == 'Required'
            and injection['stage'] == 'captured-both-trees-before-first-journal-intent'
            and injection['injected'] is True and injection['nativeAuthority'] is False
            and all(injection[key] is True for key in ('manifestIntentAndFileAbsent',
                    'currentIntentAndFileAbsent', 'originalCreatorNeverResumed'))
            and injection['raw'] == worker['rawError'] == 31
            and worker['phase'] == 'FailedRetained' and worker['creatorResumed'] is False,
            'native raw31/no-Resume evidence differs')
    require(all(worker[key] == injection[original] and re.fullmatch(pattern, worker[key])
                for key, original, pattern in (('thread', 'workerThread', r'ThreadId\([1-9][0-9]*\)'),
                    ('actionId', 'failedActionId', UUID), ('sessionId', 'ownerSessionId', UUID),
                    ('scopeDigest', 'scopeDigest', HEX))), 'original worker tuple differs')
    ranges = injection['originalClaimAndNlkVerified']
    require(set(ranges) == {'epoch', 'claimKey', 'objectsDigest'}
            and re.fullmatch(UUID, ranges['epoch'])
            and all(re.fullmatch(HEX, ranges[key]) for key in ('claimKey', 'objectsDigest')),
            'original claim/N/L/K receipt is missing or malformed')

    stdout = read(root, 'Host.stdout.log').split(b'\n')[:-1]
    prefix = b'test infrastructure::windows_owner_host024::partial_capture_tests::isolated_r7_partial_capture_host ... '
    lines = [line.removeprefix(prefix) for line in stdout]
    reports = [decode(line.split(b'=', 1)[1]) for line in lines
               if line.startswith(b'r7-initial-image-scans=')]
    native = [decode(line.split(b'=', 1)[1]) for line in lines if line.startswith(b'r7-partial-capture=')]
    require(native == [record] and reports == result['initialScans'], 'raw stdout and exports differ')
    verify_scans(reports, record['elapsedMs'])
    return {'hostReceiptsVerified': True, 'pid': creation['pid'],
            'creationFiletime': creation['creationFiletime'], 'initialFullScans': 9,
            'raw': 31, 'originalWorker': worker['thread'], 'originalRanges': ranges,
            'liveProcessVerified': False, 'laterRangeContinuityVerified': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output_directory', type=Path)
    parser.add_argument('--sid', required=True)
    parser.add_argument('--session', type=int, required=True)
    parser.add_argument('--profile', type=PureWindowsPath, required=True)
    parser.add_argument('--image', type=PureWindowsPath, required=True)
    parser.add_argument('--image-sha256', required=True)
    args = parser.parse_args()
    try:
        require(re.fullmatch(HEX, args.image_sha256), 'expected image SHA-256 is malformed')
        print(json.dumps(verify(args.output_directory, args.sid, args.session, args.profile,
                                args.image, args.image_sha256)))
    except (OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        parser.exit(1, f'{error}\n')
