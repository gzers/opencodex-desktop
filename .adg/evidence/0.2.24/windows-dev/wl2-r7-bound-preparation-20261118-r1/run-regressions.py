import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import time


root = Path(__file__).resolve().parent
code = Path(sys.argv[1]).resolve()
build = root / sys.argv[2]
build_result = json.loads((build / 'result.json').read_text(encoding='utf-8'))
assert build_result['raw'] == 0, build_result
stderr = (build / 'stderr.log').read_text(encoding='utf-8')
paths = re.findall(r'^\s*Running unittests src[\\/]lib\.rs \((.+?\.exe)\)$', stderr, re.M)
assert len(paths) == 1, paths
exe = (code / paths[0]).resolve(strict=True)
assert exe.is_relative_to(code / 'test/windows024-native/target/w1c-product'), exe
with exe.open('rb') as source:
    artifact = hashlib.file_digest(source, 'sha256').hexdigest()
catalogue = subprocess.check_output([str(exe), '--list'], cwd=code, text=True)

filters = {
    'v2-reference': 'wl2_r7_v2_reference_',
    'v2-storage': 'wl2_r7_v2_storage_native_',
    'v2-storage-history': 'wl2_r7_history_primitives_native_v2attachment_',
    'v1-attachment': 'r7_abort_attachment_',
    'recovery-origin': 'wl2_r7_recovery_origin_',
    'original-adapter-send': 'wl2_r6_typed_adapter_retained_v3_slot_and_private_observation_types_are_send',
    'status-polling': 'modules::status::polling::tests::',
    'windows-status-same-sample': 'status_source::windows_sample_tests::',
}
selected = {
    label: [line.removesuffix(': test') for line in catalogue.splitlines()
            if case in line and line.endswith(': test')]
    for label, case in filters.items()
}
assert all(selected.values()), selected
assert len({name for names in selected.values() for name in names}) == sum(map(len, selected.values()))
out = root / 'affected-regressions'
out.mkdir(exist_ok=False)
report = {
    'build': build.name,
    'testExe': str(exe),
    'bytes': exe.stat().st_size,
    'sha256': artifact,
    'selection': selected,
    'currentR2Touched': False,
    'officialStarted': False,
    'productionAccepted': False,
}


def write_json(path, data):
    path.write_text(json.dumps(data, indent=2) + '\n', encoding='utf-8')


write_json(out / 'selection.json', report)
failed = False
for label, case in filters.items():
    run = out / label
    run.mkdir()
    args = [case, '--test-threads=1', '--nocapture']
    start = time.monotonic()
    with (run / 'stdout.log').open('xb') as stdout, (run / 'stderr.log').open('xb') as stderr_log:
        result = subprocess.run([str(exe), *args], cwd=code, stdout=stdout, stderr=stderr_log)
    elapsed = round((time.monotonic() - start) * 1000)
    expected = len(selected[label])
    raw_log = (run / 'stdout.log').read_text(encoding='utf-8')
    passed = f'test result: ok. {expected} passed; 0 failed; 0 ignored;' in raw_log
    record = dict(arguments=args, raw=result.returncode, elapsedMs=elapsed,
                  artifactSha256=artifact, selected=expected, exactPassCount=passed)
    write_json(run / 'result.json', record)
    print(json.dumps(dict(run=label, **record)), flush=True)
    failed |= result.returncode != 0 or not passed
raise SystemExit(int(failed))
