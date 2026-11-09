"""Core iteration admission/sealing and falsifiable-hypothesis comparison.
Build preparation is separate. No product assertions or budgets are relaxed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import time

from summary import summarize

REPO = Path(__file__).resolve().parents[3]
PLAN = Path(__file__).with_name('core-plan.json')


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def digest(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def write(path, value):
    with Path(path).open('x', encoding='utf-8') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)


def source():
    return subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()


def validate_hypothesis(h, head, build_run):
    assert h.get('schemaVersion') == 1
    assert re.fullmatch(r'[A-Za-z0-9_-]{1,80}', h.get('id', ''))
    assert h.get('source') == head and re.fullmatch(r'[0-9a-f]{40}', head)
    assert str(h.get('buildRun')) == str(build_run) and re.fullmatch(r'[0-9]+', str(build_run))
    for key in ('claim', 'changeUnderTest', 'refutation', 'rerunReason'):
        assert isinstance(h.get(key), str) and 8 <= len(h[key]) <= 2000 and 'REPLACE' not in h[key], key
    checks = {r['step'] for r in read(PLAN)['steps']} - {'01'}
    predictions = h.get('predictions')
    assert isinstance(predictions, list) and predictions
    seen = set()
    for p in predictions:
        assert p['step'] in checks and p['step'] not in seen
        assert p['status'] in ('pass', 'fail') and len(p.get('observation', '')) >= 8
        seen.add(p['step'])
    # Closed, unaffected gates are references, not re-executed acceptance claims.
    assert isinstance(h.get('unchangedPassingEvidence'), list)
    for p in h['unchangedPassingEvidence']:
        assert all(isinstance(p.get(k), str) and p[k] for k in ('gate', 'source', 'evidence', 'whyUnaffected'))
    return h


def inventory(program):
    files = []
    for p in sorted(program.rglob('*')):
        assert not p.is_symlink(), 'No links in sealed program'
        if p.is_file() and p != program / 'core-build.json':
            files.append(dict(path=p.relative_to(program).as_posix(), bytes=p.stat().st_size, sha256=digest(p)))
    assert files
    return files


def seal(program):
    for name in ('opencodex-desktop.exe', 'ocxd.exe', 'sweep-state.exe', 'sweep-filesystem.exe',
                 'runtime-bundle/runtime-bundle.manifest.json'):
        assert (program / name).is_file(), name
    write(program / 'core-build.json', dict(schemaVersion=1, source=source(),
          appsTree=subprocess.check_output(['git', 'rev-parse', 'HEAD:apps'], cwd=REPO, text=True).strip(),
          policySha256=digest(REPO / '.github/runtime-platforms-023.json'),
          run=os.environ['GITHUB_RUN_ID'], profile='debug custom-protocol MSVC diagnostic-only',
          accepted=False, diagnosticOnly=True, files=inventory(program)))


def admit(root, head, build_run):
    h = validate_hypothesis(json.loads(os.environ['CORE_HYPOTHESIS']), head, build_run)
    assert source() == head, 'Exact checked out candidate source required'
    write(root / 'sweep/hypothesis.json', dict(**h, diagnosticOnly=True, accepted=False, startedAt=time.time()))


def verify(program, head, build_run):
    manifest = read(program / 'core-build.json')
    assert manifest['source'] == source() == head and str(manifest['run']) == str(build_run)
    assert manifest['accepted'] is False and manifest['diagnosticOnly'] is True
    assert manifest['policySha256'] == digest(REPO / '.github/runtime-platforms-023.json')
    seen = set()
    for row in manifest['files']:
        name = row['path']
        path = PurePosixPath(name)
        assert name and not path.is_absolute() and not any(part in ('', '.', '..') for part in path.parts)
        assert '\\' not in name and ':' not in name and name == path.as_posix()
        assert name.casefold() not in seen
        seen.add(name.casefold())
    assert inventory(program) == manifest['files'], 'Complete sealed program inventory/digests differ'
    for name in ('opencodex-desktop.exe', 'ocxd.exe', 'sweep-state.exe', 'sweep-filesystem.exe'):
        assert name in {r['path'] for r in manifest['files']}, name
    policy = read(REPO / '.github/runtime-platforms-023.json')
    descriptor = read(program / 'runtime-bundle/runtime-bundle.manifest.json')
    assert descriptor['runtimeVersion'] == policy['runtime']['version']
    assert descriptor['runtimeArchiveSha256'] == policy['runtime']['sha256']
    return dict(build=manifest, manifestSha256=digest(program / 'core-build.json'))


def finish(root):
    expected = [(r['step'], r['name']) for r in read(PLAN)['steps']]
    result = summarize(root / 'sweep', expected)
    result['mode'] = 'core-sweep'
    try:
        hypothesis = read(root / 'sweep/hypothesis.json')
        by_step = {r['step']: r for r in result['steps']}
        compared = [dict(step=p['step'], predicted=p['status'], observed=by_step[p['step']]['status'],
                         firstError=by_step[p['step']]['firstError'], matches=p['status'] == by_step[p['step']]['status'])
                    for p in hypothesis['predictions']]
        result['hypothesis'] = dict(id=hypothesis['id'], claim=hypothesis['claim'], refutation=hypothesis['refutation'],
                                   comparisons=compared, verdict='inconclusive' if by_step['01']['status'] != 'pass'
                                   else ('not-refuted' if all(p['matches'] for p in compared) else 'refuted'))
        elapsed = time.time() - hypothesis['startedAt']
        result['cadence'] = dict(targetSeconds=[480, 600], elapsedSeconds=elapsed, withinTarget=elapsed <= 600,
                                 budgetsUnchanged=True, buildExcluded=True,
                                 note='Target is operational only; overrun never changes a test verdict or product budget')
    except (OSError, KeyError, ValueError) as error:
        result['hypothesis'] = dict(verdict='inconclusive', error=str(error).splitlines()[0])
    write(root / 'sweep/summary.json', result)
    print(json.dumps(result, ensure_ascii=False))


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('mode', choices=['admit', 'seal', 'verify', 'finish'])
    p.add_argument('root', type=Path)
    p.add_argument('--source')
    p.add_argument('--build-run')
    a = p.parse_args()
    if a.mode == 'admit':
        admit(a.root, a.source, a.build_run)
    elif a.mode == 'seal':
        seal(a.root)
    elif a.mode == 'verify':
        print(json.dumps(verify(a.root, a.source, a.build_run)))
    else:
        finish(a.root)
