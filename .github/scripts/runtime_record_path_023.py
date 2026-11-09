"""Exact maintained-source fix for Desktop's canonical local Windows HOME."""
import hashlib

MEMBER = 'src/lib/windows-runtime-private-record.ts'
BEFORE_SHA256 = '7ccc32253a043b79b0d87ddb73fc9a5792ee3e5d068857e2b3852e1d3230fb71'
OLD = 'path.startsWith(String.fromCharCode(92,92,63,92))'
# Only an extended local drive root is admitted. Keep the supplied path intact
# for original ancestor pins, CREATE_NEW, ACL and file-identity checks.
NEW = '(' + OLD + r'&&!/^[A-Za-z]:\\/.test(path.slice(4)))'


def apply(source: bytes) -> bytes:
    if hashlib.sha256(source).hexdigest() != BEFORE_SHA256:
        raise ValueError('Private record source differs from reviewed lifecycle.2')
    text = source.decode('utf-8')
    if text.count(OLD) != 1:
        raise ValueError('Expected one private-record namespace guard')
    return text.replace(OLD, NEW, 1).encode('utf-8')
