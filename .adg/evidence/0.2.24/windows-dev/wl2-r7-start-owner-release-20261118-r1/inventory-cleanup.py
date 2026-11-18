from pathlib import Path
import hashlib, json, os, re, stat, sys

run = Path(sys.argv[1])
allowed = {
    Path(os.environ['LOCALAPPDATA']) / 'Temp': r'\.tmp[A-Za-z0-9]{6}',
    Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target'): r'r6-driver-[0-9a-f-]{36}',
}
logs = [run/'baseline/stdout.log', run/'release-native/stdout.log', run/'final-release-native/stdout.log', run/'final-release-native/segments.stdout.log']
rows = {}
for log in logs:
    for line in log.read_text(encoding='utf-8-sig').splitlines():
        match = re.search(r'(?:r4-retained-synthetic-fixture|r7-release-fixture-cleanup-absent)=(.*)$', line)
        if not match:
            continue
        p = Path(match[1].strip())
        assert any(p.parent.resolve() == parent.resolve() and re.fullmatch(pattern, p.name) for parent, pattern in allowed.items()), p
        rows.setdefault(str(p), {'logs': [], 'present': p.exists()})['logs'].append(log.relative_to(run).as_posix())

successes = {'.tmpc7cwep', 'r6-driver-7e1d64e7-db88-44e8-9764-6a03293bd8f4'}
planned = []
for value, row in sorted(rows.items()):
    p = Path(value)
    if p.name not in successes:
        continue
    assert row['logs'] == ['final-release-native/segments.stdout.log'] and row['present']
    assert p.resolve(strict=True) == p and p.is_dir()
    entries = []
    for child in [p, *sorted(p.rglob('*'))]:
        info = child.lstat()
        assert not info.st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT, child
        entry = {'relative': child.relative_to(p).as_posix(), 'fileId': info.st_ino, 'volume': info.st_dev, 'directory': child.is_dir()}
        if not entry['directory']:
            entry.update(size=info.st_size, sha256=hashlib.sha256(child.read_bytes()).hexdigest())
        entries.append(entry)
    planned.append({'root': str(p), 'entries': entries})
assert len(planned) == 2
with (run/'fixture-cleanup-plan.json').open('x', encoding='utf-8') as f:
    json.dump({'roots': rows, 'eligibleSuccessfulRoots': planned}, f, indent=2, ensure_ascii=False)
print(json.dumps({'roots': len(rows), 'present': sum(i['present'] for i in rows.values()), 'eligible': len(planned)}))
