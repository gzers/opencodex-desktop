from pathlib import Path
import hashlib
import json
import os
import subprocess
import time

ROOT = Path(__file__).resolve().parent / "phases"
CODE = Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
PREVIOUS = ROOT.parent.parent / 'wl2-r7-bound-preparation-20261118-r1'
COST = ROOT.parent.parent / 'wl2-r6-source-scan-cost-20261117-r1'


def read(path):
    return json.loads(path.read_bytes())


def write(name, value):
    with (ROOT / name).open('x', encoding='utf-8') as out:
        json.dump(value, out, indent=2)


def git(*args):
    return subprocess.check_output(['git', *args], cwd=CODE)


hypothesis = (
    'Measure independent reference and native phase costs in the complete test executable. '
    'This diagnostic has no acceptance deadline. '
    'The original 10 second failure remains recorded.'
)
product = read(PREVIOUS / 'artifact.json')
exe = Path(product['path'])
with exe.open('rb') as stream:
    digest = hashlib.file_digest(stream, 'sha256').hexdigest()
assert digest == product['sha256']
revision = git('rev-parse', 'HEAD').decode().strip()
assert revision == read(PREVIOUS / 'source-git-verification.json')['codeSha']
assert not git('status', '--porcelain')
fixed = Path(read(COST / 'fixture-records/setup/image-create-intent.json')['versionRoot']).resolve(strict=True)
assert fixed.is_relative_to(CODE / 'test/windows024-native/target')
assert 'r6-source-scan-cost-d57c32d10af74ab888fcde86e6ec97cd' in fixed.parts
test = 'modules::runtime::program_image::windows_tests::r7_image_fixed_pinned_reference_same_full_bytes_and_digest'
args = [str(exe), '--exact', test, '--ignored', '--nocapture', '--test-threads=1']
prior = dict(lightweightFixedEntry='passed 6366.008 ms previously', ordinaryOfficialLifecycle='not run', currentR2Repair='executionReady=false', owningWorkerDecision='pending', publicStartOpen=False)
write('inputs.json', dict(hypothesis=hypothesis, prior=prior, code=revision, executable=product, fixture=str(fixed), arguments=args, fullTauriLibrary=True))
env = dict(os.environ, OCXD024_R7_FIXED_SCAN_ROOT=str(fixed))
started = time.monotonic()
with (ROOT / 'stdout.log').open('xb') as stdout, (ROOT / 'stderr.log').open('xb') as stderr:
    result = subprocess.run(args, cwd=CODE, env=env, stdout=stdout, stderr=stderr, check=False)
elapsed = round((time.monotonic() - started) * 1000)
lines = (ROOT / 'stdout.log').read_text(encoding='utf-8').splitlines()
marker = 'R7_PINNED_FIXED_SCAN '
reports = [json.loads(line.split(marker, 1)[1]) for line in lines if marker in line]
assert len(reports) <= 1
write('result.json', dict(raw=result.returncode, processElapsedMs=elapsed, fixtureReport=reports[0] if reports else None, officialExecuted=False, productionAccepted=False, fixtureCleanup='reused read-only retained fixture'))
print((ROOT / 'result.json').read_text())
raise SystemExit(result.returncode)
