"""Isolated data-contract probe; this is not production implementation evidence."""
from dataclasses import dataclass, replace
from datetime import datetime, timezone
from hashlib import sha256
from pathlib import Path
from tempfile import TemporaryDirectory
import json, os

@dataclass(frozen=True)
class Version:
    digest: str
    saved_at: str = ''
    kind: str = 'unknown'

def decision(local, remote):
    if local is None and remote is None: return 'unchanged'
    if remote is None: return 'upload'
    if local is None: return 'download'
    if local.digest == remote.digest: return 'unchanged'
    if any(v.kind != 'saved' or not v.saved_at for v in (local, remote)):
        return 'undetermined'
    try:
        a, b = [datetime.fromisoformat(v.saved_at.replace('Z', '+00:00')) for v in (local, remote)]
        if a.tzinfo is None or b.tzinfo is None: return 'undetermined'
    except ValueError: return 'undetermined'
    return 'upload' if a > b else 'download' if b > a else 'undetermined'

def saved(path, content, at, previous=None):
    path.write_bytes(content)
    digest = sha256(content).hexdigest()
    if previous is not None and previous.digest == digest: return previous
    return Version(digest, at, 'saved')

def run():
    older = Version('old', '2026-10-30T00:54:00+08:00', 'saved')
    newer = Version('new', '2026-11-04T12:00:00+08:00', 'saved')
    cases = [
        ('both_empty', None, None, 'unchanged'),
        ('no_remote', newer, None, 'upload'),
        ('local_empty', None, older, 'download'),
        ('same_content_unknown_time', Version('same'), Version('same'), 'unchanged'),
        ('local_newer_without_baseline', newer, older, 'upload'),
        ('remote_newer_without_baseline', older, newer, 'download'),
        ('both_modified_local_newer', newer, older, 'upload'),
        ('both_modified_remote_newer', older, newer, 'download'),
        ('equal_instants_different_offsets', Version('a', '2026-11-04T12:00:00+08:00', 'saved'), Version('b', '2026-11-04T04:00:00Z', 'saved'), 'undetermined'),
        ('unknown_local', Version('a'), older, 'undetermined'),
        ('unknown_remote', older, Version('b'), 'undetermined'),
        ('legacy_mtime', replace(newer, kind='legacy-mtime'), older, 'undetermined'),
        ('invalid_time', replace(newer, saved_at='invalid'), older, 'undetermined'),
        ('naive_time', replace(newer, saved_at='2026-11-04T12:00:00'), older, 'undetermined'),
        ('known_clock_problem', replace(newer, kind='clock-unreliable'), older, 'undetermined'),
    ]
    results=[]
    for name, a, b, expected in cases:
        actual=decision(a,b);assert actual == expected,(name,actual,expected)
        results.append({'case':name,'result':actual,'passed':True})
    with TemporaryDirectory(prefix='ocx021-version-') as directory:
        root=Path(directory);p=root/'local.json';q=root/'received.json'
        v=saved(p,b'{"channel":"Krill"}','2026-11-04T12:00:00+08:00')
        # Repeated save and mtime-only changes cannot promote content time.
        same=saved(p,p.read_bytes(),'2026-11-05T12:00:00+08:00',v);assert same == v
        os.utime(p,(1900000000,1900000000));assert sha256(p.read_bytes()).hexdigest()==v.digest
        # Receive/copy keeps source version; legacy copying grants no trusted event.
        q.write_bytes(p.read_bytes());received=replace(v);assert received == v
        assert decision(received,v)=='unchanged'
        legacy=Version(sha256(q.read_bytes()).hexdigest());assert legacy.kind=='unknown' and not legacy.saved_at
        # A check/prepare race must invalidate the frozen facts, not rebase a stale direction.
        baseline=(v.digest, older.digest)
        changed_remote=replace(newer,digest='remote-later',saved_at='2026-11-06T04:00:00Z')
        assert baseline != (v.digest,changed_remote.digest)
        assert decision(v,changed_remote)=='download'
        results.extend({'case':name,'passed':True} for name in ['no_op_save_keeps_time','mtime_touch_not_a_save','transfer_inherits','legacy_copy_unknown','confirmation_race_recompare'])
    result={'result':'pass','checks':len(results),'productionImplemented':False,'scope':'temporary files and pure data-contract feasibility only','cases':results}
    (Path(__file__).parent/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(result,ensure_ascii=False))

if __name__ == '__main__':run()
