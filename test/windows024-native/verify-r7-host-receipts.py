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
RUNTIME_BUNDLE_SHA = '14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7'
OFFICIAL_ARCHIVE_SHA = 'b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'
RUNTIME_IMAGE_SHA = 'ab9314d3df4475481736245580b24a4d1a6fa52e8030512d9afc9340b6d6c3b6'
# The setup serializes ImageManifest in this order before hashing its bytes.
IMAGE_MANIFEST_FIELDS = (
    'schemaVersion', 'package', 'version', 'platform', 'officialArchiveSha256', 'treeSha256',
    'bin', 'packageSha256', 'binSha256', 'nodeSha256', 'npmSha256', 'bunSha256', 'bunLauncherSha256',
)


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


def verify_runtime_image(report):
    current = report['current']
    image = report['image']
    require(set(image) == set(IMAGE_MANIFEST_FIELDS), 'runtime image manifest fields differ')
    image_manifest = {field: image[field] for field in IMAGE_MANIFEST_FIELDS}
    manifest_sha = hashlib.sha256(json.dumps(image_manifest, indent=2).encode('ascii')).hexdigest()
    require(report['setupOnly'] is True and report['sharedFullImageReadback'] is True
            and report['bundleSha256'] == RUNTIME_BUNDLE_SHA
            and report['officialArchiveSha256'] == image['officialArchiveSha256'] == OFFICIAL_ARCHIVE_SHA
            and current['imageSha256'] == manifest_sha == RUNTIME_IMAGE_SHA
            and current['version'] == image['version'] == '2.99.0'
            and current['platform'] == image['platform'] == 'win32-x64'
            and type(current['schemaVersion']) is int and type(image['schemaVersion']) is int
            and current['schemaVersion'] == image['schemaVersion'] == 1
            and image['package'] == '@bitkyc08/opencodex' and image['bin'] == 'bin/ocx.mjs',
            'fixed runtime image or full readback differs')


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
    verify_runtime_image(exports['Host-image-created-verified.json'])
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
