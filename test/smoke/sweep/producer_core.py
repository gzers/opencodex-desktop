"""Unresolved producer subproblem of core-sweep. Not dual-user acceptance.
Reuses immutable offline resources, never an old product executable for a new test.
"""
import argparse
import json
import os
import re
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import sys
import tarfile
import time

import core
from summary import summarize

PLAN = [('01', 'hypothesis-and-inputs'), ('12', 'frozen-producer-supply'),
        ('13', 'runtime-readiness-and-ports'), ('18', 'owned-producer-cleanup')]


def admit(root):
    h = json.loads(os.environ['CORE_HYPOTHESIS'])
    core.validate_hypothesis(h, core.source(), os.environ['CORE_BUILD_RUN'])
    assert h.get('validationScope') == 'candidate-producer-technical-only'
    assert all(p['step'] in {'12', '13', '18'} for p in h['predictions'])
    assert re.fullmatch(r'[0-9a-f]{40}', h.get('inputSource', ''))
    assert h.get('unchangedPassingEvidence'), 'Prior closed gates require explicit references'
    core.write(root/'sweep/hypothesis.json', dict(**h, accepted=False, diagnosticOnly=True, startedAt=time.time()))


def verify_input(root):
    h = core.read(root/'sweep/hypothesis.json')
    manifest = core.read(root/'program/core-build.json')
    metadata = core.read(root/'evidence/build-run.json')
    assert h['source'] == core.source()
    assert metadata['head_sha'] == manifest['source'] == h['inputSource']
    assert str(metadata['id']) == str(manifest['run']) == str(h['buildRun'])
    assert metadata['status'] == 'completed' and metadata['conclusion'] == 'success'
    assert manifest['accepted'] is False and manifest['diagnosticOnly'] is True
    assert manifest['policySha256'] == core.digest(core.REPO/'.github/runtime-platforms-023.json')
    assert manifest['files'] == core.inventory(root/'program'), 'Original sealed inventory differs'
    return dict(accepted=False, diagnosticOnly=True, resourcesSource=manifest['source'], testSource=core.source(),
                programExecuted=False, manifestSha256=core.digest(root/'program/core-build.json'))


def extract(archive, output):
    assert not output.exists()
    output.mkdir()
    with tarfile.open(archive, 'r:gz') as source:
        members = source.getmembers()
        assert len(members) < 100000 and sum(m.size for m in members) < 2*1024**3
        seen = set()
        for m in members:
            p = PurePosixPath(m.name)
            assert m.name and str(p) == m.name and not p.is_absolute()
            assert '..' not in p.parts and '\\' not in m.name and ':' not in m.name
            assert m.isfile() or m.isdir(), 'Links or special files refused'
            assert m.name.casefold() not in seen
            seen.add(m.name.casefold())
            target = output.joinpath(*p.parts)
            if m.isdir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with target.open('xb') as stream:
                    shutil.copyfileobj(source.extractfile(m), stream)


def supply(root):
    verify_input(root)
    resources = root/'program/runtime-bundle'
    descriptor = core.read(resources/'runtime-bundle.manifest.json')
    policy = core.read(core.REPO/'.github/runtime-platforms-023.json')
    assert descriptor['runtimeVersion'] == policy['runtime']['version']
    assert descriptor['runtimeArchiveSha256'] == policy['runtime']['sha256']
    archive = resources/descriptor['archiveName']
    assert archive.parent == resources and core.digest(archive) == descriptor['archiveSha256']
    extract(archive, root/'work/offline')
    maintained = resources/'maintained-runtime.tgz'
    assert core.digest(maintained) == policy['runtime']['sha256']
    subprocess.run([sys.executable, str(core.REPO/'test/smoke/build-runtime-record-prototype.py'),
                    '--archive', str(maintained), '--output', str(root/'work/record'), '--materialize'], check=True)
    supplied = root/'work/offline/runtime/opencodex/node_modules/@bitkyc08/opencodex'
    candidate = root/'work/producer'
    subprocess.run([sys.executable, str(core.REPO/'test/smoke/stage-runtime-lifetime-candidate.py'),
                    str(root/'work/record/package'), str(candidate), str(root/'evidence/producer-source.json'), str(supplied)], check=True)
    review = core.read(root/'evidence/producer-source.json')
    assert review['dependencies']['candidateBytesEqual'] and review['dependencies']['autoInstallDisabled']
    bun = root/'work/offline/runtime/bun/bun.exe'
    assert bun.is_file()
    # Recheck all dependency bytes immediately before execution; no network
    # resolver is enabled, and no old product program is invoked.
    source = core.REPO/'test/smoke/stage-runtime-lifetime-candidate.py'
    import ast, hashlib
    module = ast.parse(source.read_text())
    function = next(n for n in module.body if isinstance(n, ast.FunctionDef) and n.name == 'dependency_inventory')
    namespace = dict(Path=Path, installed=supplied, hashlib=hashlib)
    exec(compile(ast.Module(body=[function], type_ignores=[]), str(source), 'exec'), namespace)
    rows = namespace['dependency_inventory'](candidate/'node_modules')
    assert hashlib.sha256(json.dumps(rows,sort_keys=True,separators=(',', ':')).encode()).hexdigest() == review['dependencies']['inventorySha256']
    core.write(root/'evidence/producer-inputs.json', dict(accepted=False, diagnosticOnly=True,
               source=core.source(), bun=str(bun), package=str(candidate), dependencies=review['dependencies'],
               programExecuted=False, dualSidProductAcceptance=False))
    return review['dependencies']


def execute(root):
    inputs = core.read(root/'evidence/producer-inputs.json')
    fixture = root/'work/fixture'
    fixture.mkdir(exist_ok=False)
    env = dict(os.environ, OCXD_PRODUCER_PROBE_ROOT=str(fixture), OCXD_PRODUCER_PROBE_PACKAGE=inputs['package'])
    with (root/'evidence/producer.stdout.log').open('xb') as out, (root/'evidence/producer.stderr.log').open('xb') as err:
        child = subprocess.run([inputs['bun'], '--no-env-file', '--no-install',
                                str(core.REPO/'test/smoke/windows023-runtime-producer-probe.mjs')],
                               env=env, cwd=root, stdout=out, stderr=err)
    report = core.read(fixture/'producer-report.json')
    core.write(root/'evidence/producer-report.json', report)
    assert report['source'] == core.source() and report['accepted'] is False
    assert child.returncode == 0 and all(report.get(k) is True for k in [
        'startWithin30000', 'recordAndHealthAttestation', 'lifetimeResponseProof',
        'officialStopWithin10000', 'officialClientRestoreConfirmed', 'originalProcessExit',
        'controlledSetEmpty', 'leaseUnheld', 'recordAbsent', 'fixtureSafeToRemove']), (
        'producer: '+str(report.get('errorStage'))+' / '+str(report.get('requestPhase'))+' / '+str(report.get('stopReply')))
    return report


def cleanup(root):
    report = core.read(root/'evidence/producer-report.json')
    assert report.get('fixtureSafeToRemove') is True, 'Exact fixture retained: no proven original exit/Job empty/lease free/record absence; never force kill'
    for name in ['fixture', 'producer', 'record', 'offline']:
        path = root/'work'/name
        assert path.parent == root/'work' and not path.is_symlink()
        shutil.rmtree(path)
        assert not path.exists()
    return dict(ownedFixtureRemoved=True, terminationUsed=False, evidencePreserved=True, dailyConfigurationTouched=False)


def record(root, step, name, work):
    try:
        evidence = work(root)
        row = dict(status='pass', firstError='', evidence=evidence)
    except Exception as error:
        row = dict(status='fail', firstError=str(error).splitlines()[0] or type(error).__name__, evidence=None)
    core.write(root/f'sweep/{step}-{name}.json', dict(schemaVersion=1,step=step,name=name,accepted=False,diagnosticOnly=True,**row))
    if row['status'] != 'pass':
        raise SystemExit(1)


def finish(root):
    result = summarize(root/'sweep', PLAN)
    result.update(mode='core-sweep', focus='candidate-producer-technical-only', dualSidProductAcceptance=False)
    try:
        h = core.read(root/'sweep/hypothesis.json')
        rows = {r['step']:r for r in result['steps']}
        comparisons = [dict(step=p['step'], predicted=p['status'], observed=rows[p['step']]['status'],
                            firstError=rows[p['step']]['firstError'], matches=p['status']==rows[p['step']]['status']) for p in h['predictions']]
        result.update(untouchedGates=h['unchangedPassingEvidence'], hypothesis=dict(id=h['id'], claim=h['claim'],
                      refutation=h['refutation'], comparisons=comparisons,
                      verdict='inconclusive' if rows['01']['status']!='pass' or rows['12']['status']!='pass' else
                      ('not-refuted' if all(c['matches'] for c in comparisons) else 'refuted')),
                      cadence=dict(elapsedSeconds=time.time()-h['startedAt'],targetSeconds=[480,600],budgetsUnchanged=True))
    except (OSError, KeyError, ValueError) as error:
        result['hypothesis'] = dict(verdict='inconclusive', firstError=str(error).splitlines()[0])
    core.write(root/'sweep/summary.json',result)
    print(json.dumps(result))


if __name__ == '__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('mode',choices=['admit','verify','supply','execute','cleanup','finish'])
    parser.add_argument('root',type=Path)
    args=parser.parse_args()
    if args.mode=='admit': admit(args.root)
    elif args.mode=='finish': finish(args.root)
    else:
        step,name,work={'verify':('01','hypothesis-and-inputs',verify_input),
                        'supply':('12','frozen-producer-supply',supply),
                        'execute':('13','runtime-readiness-and-ports',execute),
                        'cleanup':('18','owned-producer-cleanup',cleanup)}[args.mode]
        record(args.root,step,name,work)
