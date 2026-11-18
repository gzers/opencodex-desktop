from pathlib import Path
import hashlib
import json
import os
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parent
CODE = Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
PRODUCT = ROOT / 'product-scanner'
RUN = ROOT / 'fixed-after'


def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def emit(path, value):
    with path.open('x', encoding='utf-8') as out:
        json.dump(value, out, indent=2, ensure_ascii=False)
        out.write('\n')


assert read(PRODUCT / 'result.json')['raw'] == 0
log = (PRODUCT / 'stderr.log').read_text(encoding='utf-8')
[exe_path] = re.findall(r'Running unittests .*?\(([^\r\n]+\.exe)\)', log)
exe = (CODE / exe_path).resolve(strict=True)
assert exe.is_relative_to(CODE / 'test/windows024-native/target/w1c-product')
prior = read(ROOT / 'inputs.json')
fixture = Path(prior['fixture']).resolve(strict=True)
assert fixture.is_relative_to(CODE / 'test/windows024-native/target')
assert 'r6-source-scan-cost-d57c32d10af74ab888fcde86e6ec97cd' in fixture.parts
src = read(PRODUCT / 'source.json')['sources']
for row in src:
    assert sha(CODE / row['path']) == sha(PRODUCT / 'executed-source' / row['path']) == row['sha256']
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=CODE).decode().strip()
arguments = [str(exe), *prior['arguments'][1:]]
RUN.mkdir()
emit(RUN / 'inputs.json', dict(code=revision, sourceFiles=src, executable=dict(path=str(exe), sha256=sha(exe), bytes=exe.stat().st_size), fixture=str(fixture), arguments=arguments, deadlineMs=10000))
started = time.monotonic()
with (RUN / 'stdout.log').open('xb') as out, (RUN / 'stderr.log').open('xb') as err:
    result = subprocess.run(arguments, cwd=CODE, env=dict(os.environ, OCXD024_R7_FIXED_SCAN_ROOT=str(fixture)), stdout=out, stderr=err, check=False)
elapsed = round((time.monotonic() - started) * 1000)
marker = 'R7_PINNED_ENTRY_FULL_VERIFY '
reports = [json.loads(row.split(marker, 1)[1]) for row in (RUN / 'stdout.log').read_text(encoding='utf-8').splitlines() if marker in row]
assert len(reports) <= 1
result_data = dict(raw=result.returncode, processElapsedMs=elapsed, fixtureReport=reports[0] if reports else None, officialExecuted=False, productionAccepted=False, fixtureCleanup='reused read-only retained fixture')
emit(RUN / 'result.json', result_data)
print(json.dumps(result_data))
raise SystemExit(result.returncode)
