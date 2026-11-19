import argparse
import hashlib
import json
import re
from pathlib import Path

EXPLICIT_SHA = 'f06070f4f474b94289323872db63f73657f7a39b1eccc571d07d80326967648b'
COLLECTOR_SHA = 'ff4cc38475476a60b98fe981c09a40ecfc7447d46bfbeec1a155839b7b5c5e82'
PRIOR_SHA = 'ba74770fa579a673d2a402383552c9f842825a55aff52efde051d5e5fa5dc3e4'
PREFIX_SHA = '4faebbaf1807aa1aeb9ef7524892d681645974172ab3d81ff129bf733aaf7819'
FILE_KEY = '0ebd4e59:00010000001a4e61'
CREATION = '134395302700540299'
DACL_SHA = '2e65b8119cde90e80709f5ccff35de45a725be8e167ed03d241eee1b96044951'
HOST = {'pid': 25144, 'filetime': '134395302283231479', 'session': 7, 'exited': False}
CONTEXT = {'sid': 'S-1-5-21-587408064-856152790-2583328096-1003', 'session': 7,
           'desktop': 'WinSta0/Default', 'elevated': 0, 'elevationType': 1}
STAGES = {'Unknown', 'v2-original-request-intent', 'v2-original-request-result'}
ERROR_KINDS = {'TimedOut', 'WouldBlock', 'PermissionDenied', 'InvalidData', 'BrokenPipe',
               'Other', 'NotFound', 'ConnectionAborted'}


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def exact_keys(data, keys, label):
    require(isinstance(data, dict) and set(data) == set(keys.split()), f'{label} shape differs')


def hex_digest(value):
    return isinstance(value, str) and re.fullmatch(r'[0-9a-f]{64}', value) is not None


def valid_guid(value):
    return isinstance(value, str) and re.fullmatch(r'[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}', value) is not None


def validate_receipt(receipt):
    exact_keys(receipt, 'schema passed startedUtc firstError explicitReceiptSha256 acceptedWorker '
               'acceptedWorkerEvidenceCommit fileName pathResolution context hostBefore handleBefore '
               'protectionBefore pathBefore repeatHandle handleAfter protectionAfter pathAfter range '
               'hostAfter priorReceiptSha256 priorRange snapshotMode events finishedUtc', 'receipt')
    require(receipt['schema'] == 'team72-event-snapshot-r2' and receipt['passed'] is True
            and receipt['firstError'] is None, 'snapshot did not pass')
    require(receipt['explicitReceiptSha256'] == EXPLICIT_SHA
            and receipt['acceptedWorker'] == 'ThreadId(51)'
            and receipt['acceptedWorkerEvidenceCommit'] == 'd88b1839e456751268ae2bf741cca64e665c548e',
            'receipt anchors differ')
    require(receipt['fileName'] == 'before-resume-private-control-v2-events.jsonl'
            and receipt['pathResolution'] == 'fixed source and TEAM72 filename, normalized r7 path',
            'original log anchor differs')
    require(receipt['priorReceiptSha256'] == PRIOR_SHA
            and receipt['priorRange'] == dict(start=0, endExclusive=5622, sha256=PREFIX_SHA)
            and receipt['snapshotMode'] == 'in-memory-projection-only', 'v1 prefix binding differs')
    require(receipt['context'] == CONTEXT and receipt['hostBefore'] == receipt['hostAfter'] == HOST,
            'collector context or live generation differs')
    for name in ('startedUtc', 'finishedUtc'):
        require(re.fullmatch(r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{7}Z', receipt[name]),
                f'{name} is not a UTC observation time')
    require(receipt['startedUtc'] <= receipt['finishedUtc'], 'UTC observations out of order')
    file_names = ('handleBefore', 'pathBefore', 'repeatHandle', 'handleAfter', 'pathAfter')
    members = [receipt[name] for name in file_names]
    for member in members:
        exact_keys(member, 'key creationFiletime lastWriteFiletime length attributes links', 'file')
        require(member['key'] == FILE_KEY and member['creationFiletime'] == CREATION
                and re.fullmatch(r'[0-9]{1,20}', member['lastWriteFiletime'])
                and member['links'] == 1 and (member['attributes'] & 0x410) == 0, 'file identity/type differs')
        require((member['key'], member['creationFiletime']) ==
                (members[0]['key'], members[0]['creationFiletime']), 'path and handle generations differ')
    for name in ('protectionBefore', 'protectionAfter'):
        protection = receipt[name]
        exact_keys(protection, 'protectedDacl ownerIsOriginalSid grantCount ownerAndDaclSha256', 'protection')
        require(protection['protectedDacl'] is True and protection['ownerIsOriginalSid'] is True
                and protection['grantCount'] == 2 and protection['ownerAndDaclSha256'] == DACL_SHA,
                'original protection differs')
    require(receipt['protectionBefore'] == receipt['protectionAfter'], 'protection changed')
    span = receipt['range']
    exact_keys(span, 'start endExclusive sha256 completeFileAtLastObservation', 'range')
    end = span['endExclusive']
    require(span['start'] == 0 and type(end) is int and 5622 <= end <= 16_777_216
            and end == members[0]['length'] and all(item['length'] >= end for item in members)
            and hex_digest(span['sha256']) and (end != 5622 or span['sha256'] == PREFIX_SHA), 'stable range differs')
    require(span['completeFileAtLastObservation'] is (end == members[3]['length'] == members[4]['length']),
            'complete range assertion differs')
    events = receipt['events']
    exact_keys(events, 'completeLines incompleteTailUtf8Bytes matchingIntents latestBoundStage requestId '
               'transportActionId resultCount resultPurpose resultErrorKind resultRaw explicitRecoveryActionId '
               'v2Durability scanStage replyDelivery', 'event projection')
    require(all(type(events[key]) is int and 0 <= events[key] <= end for key in
                ('completeLines', 'incompleteTailUtf8Bytes', 'matchingIntents')),
            'event counts outside observed prefix')
    require(events['matchingIntents'] <= events['completeLines'] and
            events['completeLines'] + events['incompleteTailUtf8Bytes'] <= end, 'event counts inconsistent')
    stage = events['latestBoundStage']
    require(stage in STAGES and all(events[key] == 'Unknown' for key in
            ('v2Durability', 'scanStage', 'replyDelivery')) and events['explicitRecoveryActionId'] is None,
            'projection claims authority beyond the event log')
    if stage != 'Unknown':
        require(events['matchingIntents'] == 1 and valid_guid(events['requestId'])
                and valid_guid(events['transportActionId']) and type(events['resultCount']) is int
                and 0 <= events['resultCount'] < events['completeLines'], 'bound event IDs or count differ')
    else:
        require(all(events[key] is None for key in ('requestId', 'transportActionId', 'resultCount')),
                'unknown stage exports an unbound request')
    if stage == 'v2-original-request-result':
        require(events['resultCount'] == 1 and events['resultPurpose'] in
                {'refused', 'canonical-abort-result'} and
                (events['resultErrorKind'] in ERROR_KINDS or events['resultErrorKind'] is None) and
                (type(events['resultRaw']) is int or events['resultRaw'] is None), 'result projection differs')
    else:
        require(all(events[key] is None for key in ('resultPurpose', 'resultErrorKind', 'resultRaw')),
                'projection exports an unbound result')
    return stage


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('receipt', type=Path)
    parser.add_argument('explicit', type=Path)
    parser.add_argument('collector', type=Path)
    parser.add_argument('prior', type=Path)
    args = parser.parse_args()
    require(digest(args.explicit.read_bytes()) == EXPLICIT_SHA, 'original Explicit receipt bytes differ')
    require(digest(args.collector.read_bytes()) == COLLECTOR_SHA, 'staged collector bytes differ')
    require(digest(args.prior.read_bytes()) == PRIOR_SHA, 'original v1 receipt bytes differ')
    raw = args.receipt.read_bytes()
    stage = validate_receipt(json.loads(raw))
    print(json.dumps({'passed': True, 'explicitReceiptSha256': EXPLICIT_SHA,
                      'projectionSha256': digest(raw), 'projectionBytes': len(raw),
                      'latestBoundStage': stage, 'privateRawIndependentlyRead': False}, indent=2))


if __name__ == '__main__':
    main()
