"""Read original preflight evidence only. Never execute archive members or change accounts."""
import hashlib, json, pathlib, zipfile
ROOT = pathlib.Path(__file__).resolve().parent
SPECS = {
    'preflight-42314822371.zip': (109472, '26a55dcff684bcf63214fc3f5a3b68db983b9ac41906021e78181dc5eddda859'),
    'preflight-42315398563.zip': (112007, 'da4d5bde9c6142e04d740db6b66206c7eab243ee75d0d4a5889bd8bbefc1437b'),
}
def read_zip(name):
    data = (ROOT/name).read_bytes()
    size, digest = SPECS[name]
    assert len(data) == size and hashlib.sha256(data).hexdigest() == digest
    with zipfile.ZipFile(ROOT/name) as archive:
        entries = archive.infolist()
        assert len(entries) < 2000 and sum(x.file_size for x in entries) < 16*1024*1024
        for entry in entries:
            path = pathlib.PurePosixPath(entry.filename)
            assert not path.is_absolute() and '..' not in path.parts
            assert ':' not in entry.filename and chr(92) not in entry.filename
            assert (entry.external_attr >> 16) & 0o170000 != 0o120000
        return {entry.filename: archive.read(entry) for entry in entries if not entry.is_dir()}
def parse(files, name):
    return json.loads(files[name].decode('utf-8-sig'))
failed = read_zip('preflight-42314822371.zip')
succeeded = read_zip('preflight-42315398563.zip')
first = parse(failed, 'native/rust-ordinary.json')
assert first['expectedCommit'] == '1ad1bf5b3534dac74e370b7f626539dd1a9ebf55'
assert first['result'] == 'fail' and first['exitCode'] == 1
assert first['nativeSuiteExecuted'] and not first['allBinariesAttempted']
assert 'native/ordinary/ordinary-suite.json' not in failed
assert b'test result: ok. 2 passed' in failed['native/ordinary/test-1.stdout.log']
parent = parse(succeeded, 'native/rust-ordinary.json')
child = parse(succeeded, 'native/ordinary/ordinary-suite.json')
assert parent['expectedCommit'] == child['expectedCommit'] == '47eb7cf7420ba9127f4d7047e4d83231f826035f'
assert parent['result'] == child['result'] == 'pass-ordinary-rust-binaries'
assert len(child['tests']) == 1
single = child['tests'][0]
for value in [parent['exitCode'], child['exitCode'], single['exitCode']]:
    assert type(value) is int and value == 0, 'null/string/bool is not an authoritative successful exit'
assert single['exitAuthority'] == single['process']['identityAuthority'] == 'retained-CreateProcess-handle'
assert single['identityObservedBeforeResume'] is True
assert child['allBinariesAttempted'] is parent['allBinariesAttempted'] is True
assert child['fullWorkspacePassed'] is False and child['docTestsExecuted'] is False
assert child['accountSid'] == child['tokenUser'] == child['tokenOwner'] == child['freshDirectoryOwner']
assert single['token']['tokenUser'] == single['process']['sid'] == child['accountSid']
assert single['token']['elevated'] is False and not single['cim']['contradictions']
assert child['profile'] == child['home'] and child['temp'].startswith(child['profile'] + chr(92))
for receipt in [first, parent]:
    for key in ['sourceBytesUnchanged', 'sourceOrArtifactAclRestored', 'desktopAclRestored', 'accountAndProfileRemoved']:
        assert receipt[key] is True, key
    assert receipt['remainingNewSidProcesses'] == [] and receipt['aclRestoreErrors'] == []
assert b'test result: ok. 2 passed' in succeeded['native/ordinary/test-1.stdout.log']
assert b'test result: ok. 1 passed' in succeeded['native/ordinary/test-1.stdout.log']
result = {
    'result': 'pass-evidence-audit-only', 'originalZipHashesVerified': True,
    'failedPreflightRun': 42314822371, 'failedPreflightPreserved': True,
    'successfulPreflightRun': 42315398563, 'nativeExitIntegerZeroVerified': True,
    'nativeExitAuthority': single['exitAuthority'], 'ordinaryIdentityVerified': True,
    'accountAndProfileCleanupVerified': True, 'allDaclAndSourceRestorationVerified': True,
    'testBinaryCount': 1, 'stdoutParentTestsPassed': 2, 'stdoutExactChildTestsPassed': 1,
    'fullProductPassed': False, 'twoSidProductPassed': False,
    'installersPassed': False, 'doctestsPassed': False, 'accountsCreatedLocally': False,
    'archiveMemberProgramsExecuted': False,
}
(ROOT/'preflight-audit.json').write_text(json.dumps(result, indent=2)+chr(10), encoding='utf-8')
print(json.dumps(result))
