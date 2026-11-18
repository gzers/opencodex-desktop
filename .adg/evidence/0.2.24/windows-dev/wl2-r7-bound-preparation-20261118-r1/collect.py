from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
CODE = Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
PREV = ROOT.parent / 'wl2-r7-tree-membership-20261118-r1'
SCHEMA = 'apps/desktop/tauri/gen/schemas'


def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def write(name, data):
    (ROOT / name).write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def git(*args):
    return subprocess.check_output(['git', *args], cwd=CODE)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def normalize(raw):
    return raw.replace(b'\r\n', b'\n')


def verify_sources(revision):
    paths = read(ROOT / 'sources.json')
    snapshot = ROOT / 'product05/executed-source'
    rows = read(ROOT / 'product05/source.json')['sources']
    assert {r['path'] for r in rows} == set(paths) and len(paths) == 8
    for row in rows:
        name = row['path']
        file = snapshot / name
        assert file.stat().st_size == row['bytes'] and digest(file) == row['sha256'], name
        assert file.read_bytes() == (CODE / name).read_bytes(), name
        assert normalize(file.read_bytes()) == normalize(git('show', f'{revision}:{name}')), name
    return rows


def prepare():
    log = (ROOT / 'product05/stderr.log').read_text(encoding='utf-8')
    binaries = re.findall(r'Running unittests [^\r\n]+? \(([^\r\n]+\.exe)\)', log)
    assert len(binaries) == 1
    exe = CODE / binaries[0]
    write('artifact.json', dict(path=exe.as_posix(), bytes=exe.stat().st_size, sha256=digest(exe), run='product05', kind='MSVC Tauri library test executable'))
    selected = read(ROOT / 'affected-regressions/selection.json')
    assert selected['sha256'] == '20ebb9e84ddb5fa32e115fb594434f258d0dbe083d23be11e156587eb1fdb63f'
    runs = {}
    for folder in sorted(ROOT.glob('product*')):
        if not folder.is_dir():
            continue
        path = folder / 'result.json'
        runs[folder.name] = read(path) if path.exists() else dict(state='interrupted', raw=None, evidence='partial stdout/stderr; process/session absent at follow-up')
    assert [runs[k]['raw'] for k in ('product01','product02','product03','product04','product05')] == [101,0,None,0,0]
    regressions = {}
    for name, tests in selected['selection'].items():
        folder = ROOT / 'affected-regressions' / name
        result = read(folder / 'result.json')
        assert result['raw'] == 0 and f'{len(tests)} passed; 0 failed' in (folder / 'stdout.log').read_text(encoding='utf-8')
        regressions[name] = dict(passed=len(tests), result=result)
    assert sum(item['passed'] for item in regressions.values()) == 37
    receipts = {}
    for path in sorted(ROOT.rglob('stdout.log')):
        lines = path.read_text(encoding='utf-8', errors='replace')
        roots = re.findall(r'(?:^|\n)r7-[\w-]+-cleanup-absent=(.+)', lines)
        if roots:
            receipts[path.parent.relative_to(ROOT).as_posix()] = sorted(set(p.strip() for p in roots))
    assert all(len(receipts[k]) == 2 for k in ('product02','product04','product05'))
    unique = {p for paths in receipts.values() for p in paths}
    assert all(not Path(p).exists() for p in unique)
    failures = read(PREV / 'cleanup-readback.json')['failedAndRejectedRoots']
    write('cleanup-readback.json', dict(exactNewRoots=len(unique), allExactNewRootsAbsent=True, receipts=receipts, previousFailedAndRejectedRoots=[dict(path=row['path'], present=Path(row['path']).exists()) for row in failures], previousDeletionAutoReview='blocked by policy', previousDeletionRetried=False, testsWithoutExactReceipt='No cleanup claim inferred', historicalCleanupComplete=False, ordinaryDesktopOutEmpty=not list(Path('C:/Users/Public/ocx-r7-private-native-20261118-r1/out01').iterdir())))
    write('run-results.json', dict(builds=runs, regressions=regressions, firstBuildError='LLVM out of memory; Cargo raw101, rustc 0xc0000409; no native tests', recovery='Command-local Cargo codegen-units 16 to 4, j1, debug0, incremental off, locked/offline', regressionSelectionFirstError='Windows same-sample filter used source-file name and stopped before any regression executed; corrected to status_source::windows_sample_tests::', noUnexpectedNativeTestFailure=True))
    schema = ROOT / 'generated-schema'
    schema.mkdir(exist_ok=False)
    for name in ('desktop-schema.json', 'windows-schema.json'):
        shutil.copyfile(CODE / SCHEMA / name, schema / name)
    (schema / 'desktop-schema-original.json').write_bytes(git('show', f'HEAD:{SCHEMA}/desktop-schema.json'))
    git('restore', '--worktree', '--', f'{SCHEMA}/desktop-schema.json')
    shutil.move(CODE / SCHEMA / 'windows-schema.json', schema / 'windows-schema-moved.json')
    assert (schema / 'windows-schema.json').read_bytes() == (schema / 'windows-schema-moved.json').read_bytes()
    assert normalize((CODE / SCHEMA / 'desktop-schema.json').read_bytes()) == normalize((schema / 'desktop-schema-original.json').read_bytes())
    paths = read(ROOT / 'sources.json')
    git('add', '--', *paths)
    assert set(git('diff', '--cached', '--name-only').decode().splitlines()) == set(paths)
    git('diff', '--cached', '--check')
    verify_sources('')
    print(json.dumps(dict(artifact=read(ROOT/'artifact.json'), roots=len(unique), staged=len(paths))))


if sys.argv[1] == 'prepare':
    prepare()
else:
    revision = git('rev-parse', 'HEAD').decode().strip()
    write('source-git-verification.json', dict(codeSha=revision, buildParent=git('rev-parse', 'HEAD^').decode().strip(), sources=verify_sources(revision), executedSnapshotMatchesGitWithOnlyCRLFNormalization=True))
    print(revision)

