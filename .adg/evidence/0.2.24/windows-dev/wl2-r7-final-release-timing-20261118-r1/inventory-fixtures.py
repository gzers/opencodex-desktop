from pathlib import Path
import hashlib
import json
import os
import re
import stat
import sys

batch = Path(sys.argv[1])
allowed = {
    Path(os.environ['LOCALAPPDATA']) / 'Temp': r'.tmp[A-Za-z0-9]{6}',
    Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target'): r'r6-driver-[0-9a-f-]{36}',
}
success_tests = {
    'wl2_r6_retirement_current_files_and_original_absent_artifacts_retire_same_handles_full_tree_restore',
    'wl2_r6_retirement_existing_artifacts_changed_and_atomically_replaced_work_restore_original_objects',
}
record = {}
for log in sorted(batch.rglob('stdout.log')):
    test = None
    for line in log.read_text(encoding='utf-8-sig').splitlines():
        match = re.search(r'^test .*::(wl2_[A-Za-z0-9_]+) ...', line)
        if match:
            test = match[1]
        match = re.search(r'(?:r4-retained-synthetic-fixture|r7-final-release-fixture-cleanup-absent)=(.*)$', line)
        if not match:
            continue
        path = Path(match[1].strip())
        assert any(path.parent.resolve() == parent.resolve() and re.fullmatch(pattern, path.name) for parent, pattern in allowed.items()), path
        assert str(path) not in record, path
        record[str(path)] = {'test': test, 'log': log.relative_to(batch).as_posix(), 'present': path.exists()}

plan = []
for name, row in record.items():
    if row['log'] != 'combined-scan/retirement/stdout.log' or row['test'] not in success_tests:
        continue
    root = Path(name)
    assert row['present'] and root.resolve(strict=True) == root and root.is_dir(), root
    entries = []
    for child in [root, *sorted(root.rglob('*'))]:
        info = child.lstat()
        assert not info.st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT, child
        entry = {'relative': child.relative_to(root).as_posix(), 'fileId': info.st_ino, 'volume': info.st_dev, 'directory': child.is_dir()}
        if not entry['directory']:
            entry.update(size=info.st_size, sha256=hashlib.sha256(child.read_bytes()).hexdigest())
        entries.append(entry)
    plan.append({'root': name, 'entries': entries})
assert len(plan) == 4, plan
with (batch/'fixture-inventory.json').open('x', encoding='utf-8') as output:
    json.dump({'roots': record, 'eligibleSuccessfulRoots': plan}, output, indent=2, ensure_ascii=False)
print(json.dumps({'roots': len(record), 'present': sum(item['present'] for item in record.values()), 'eligible': [item['root'] for item in plan]}))
