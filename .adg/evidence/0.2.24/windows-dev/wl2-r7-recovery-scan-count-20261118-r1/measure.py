import collections
import hashlib
import json
import re
import sys
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED

run = Path(sys.argv[1])
results = json.loads((run / 'tests.result.json').read_text(encoding='utf-8'))
result = {'traces': {}, 'fixtures': []}

def digest(data):
    return hashlib.sha256(data).hexdigest()

def current_inventory(root):
    rows = []
    for path in [root, *sorted(root.rglob('*'))]:
        info = path.lstat()
        if info.st_file_attributes & 1024:
            raise ValueError(f'unexpected fixture reparse point: {path}')
        directory = path.is_dir()
        rows.append({'relative': path.relative_to(root).as_posix(), 'fileId': info.st_ino,
                     'volume': info.st_dev, 'directory': directory,
                     'sha256': None if directory else digest(path.read_bytes())})
    return rows

for test in results['tests']:
    name = test['name']
    stdout = (run / f'{name}.stdout.log').read_text(encoding='utf-8', errors='strict')
    stderr = (run / f'{name}.stderr.log').read_text(encoding='utf-8', errors='strict')
    [trace] = [json.loads(line.split('=', 1)[1]) for line in stdout.splitlines()
               if line.startswith('r7-recovery-timing=')]
    assert [row['ordinal'] for row in trace] == list(range(1, 1 + len(trace)))
    stages = collections.defaultdict(lambda: {'calls': 0, 'completed': 0, 'elapsedUs': 0})
    for row in trace:
        stage = stages[(row['phase'], row['operation'])]
        stage['calls'] += 1
        stage['completed'] += row['completed']
        stage['elapsedUs'] += row['elapsedUs']
    intervals = sorted((row['startedUs'], row['startedUs'] + row['elapsedUs']) for row in trace)
    union = end = 0
    for start, finish in intervals:
        union += max(0, finish - max(start, end))
        end = max(end, finish)
    result['traces'][name] = {'raw': test['raw'], 'elapsedMs': test['elapsedMs'],
        'totalCalls': len(trace), 'measuredUnionUs': union,
        'stages': [dict(phase=k[0], operation=k[1], **v) for k,v in stages.items()],
        'firstIncompleteReadback': next((row for row in trace if not row['completed']), None),
        'originalDeadlineReported': 'bounded-original-deadline' in (stderr + stdout),
        'fixedBunBusyRaw73': bool(re.search(r'raw.?73|raw=Some\(73\)', stdout))}
    recorded = set()
    for line in stdout.splitlines():
        marker = re.match(r'(?:r4-retained-synthetic-fixture|r7-final-release-fixture-cleanup-absent)=(.+)$', line)
        if not marker:
            continue
        root = Path(marker[1]).absolute()
        if root in recorded:
            continue
        recorded.add(root)
        data = root.parent == Path(results['dataRoot']).absolute() and re.fullmatch('r6-driver-[0-9a-f-]{36}', root.name)
        temp = root.parent == Path(results['tempRoot']).absolute() and re.fullmatch(r'\.tmp[A-Za-z0-9]+', root.name)
        assert data or temp, root
        candidates = test['newDataRoots'] if data else test['newTempRoots']
        assert root.name in candidates or not root.exists(), root
        roots = {'test': name, 'path': str(root), 'present': root.exists(), 'archive': None, 'rows': []}
        result['fixtures'].append(roots)
        if not roots['present']:
            continue
        inventory = current_inventory(root)
        roots['rows'] = inventory
        if test['raw'] != 0:
            continue
        assert root.resolve(strict=True) == root, root
        archive = run / f'{name}-{root.name}.zip'
        files = [row for row in inventory if not row['directory']]
        with ZipFile(archive, 'x', ZIP_DEFLATED) as zip:
            for row in inventory:
                if row['directory']:
                    if row['relative'] != '.':
                        zip.writestr(row['relative'] + '/', b'')
                else:
                    zip.write(root / row['relative'], row['relative'])
        with ZipFile(archive) as zip:
            assert zip.testzip() is None
            assert {n for n in zip.namelist() if not n.endswith('/')} == {r['relative'] for r in files}
            for row in files:
                assert digest(zip.read(row['relative'])) == row['sha256']
        assert inventory == current_inventory(root)
        roots['archive'] = {'path': str(archive.absolute()), 'bytes': archive.stat().st_size,
                            'sha256': digest(archive.read_bytes())}
(run / 'measurement.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')

