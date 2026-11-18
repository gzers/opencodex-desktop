from pathlib import Path
import hashlib
import json
import os
import statistics
import subprocess
import time

root = Path(__file__).resolve().parent
code = root.parents[3]
fixture = root.parent / 'r6-source-scan-cost-d57c32d10af74ab888fcde86e6ec97cd/I/runtime/versions/2.99.0'
baseline = root.parent / 'r7-late-stream-20261118-r2/opencodex_desktop_lib-bdc9ec5a3f66e096.exe'
candidate = json.loads((root / 'compile-test/artifacts.json').read_bytes())[0]
namespace = 'modules::runtime::program_image::windows_tests::'
exact = namespace + 'r7_image_fixed_pinned_entry_full_verify_with_original10s_deadline'

def save(path, value):
    with path.open('x', encoding='utf-8') as out:
        json.dump(value, out, indent=2)
        out.write('\n')

def digest(path):
    with Path(path).open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

assert digest(baseline) == 'd58b2616507adef02f51bfc8d1088051c5f4eca75a9cdd4b458808424dfcfdbc'
assert baseline.stat().st_size == 102793728
assert digest(candidate['path']) == candidate['sha256']
assert digest(fixture / '.runtime-image.json') == 'ab9314d3df4475481736245580b24a4d1a6fa52e8030512d9afc9340b6d6c3b6'
assert (code / 'apps/desktop/tauri/Cargo.toml').is_file()
results = root / 'direct'
results.mkdir()
fixtures = root / 'fixtures'
fixtures.mkdir()
env = dict(os.environ, OCXD024_R7_FIXED_SCAN_ROOT=str(fixture), TMP=str(fixtures), TEMP=str(fixtures))

def execute(name, executable, *args):
    folder = results / name
    folder.mkdir()
    argv = [str(executable), *args, '--nocapture', '--test-threads=1']
    started = time.monotonic()
    with (folder / 'stdout.log').open('xb') as out, (folder / 'stderr.log').open('xb') as err:
        raw = subprocess.run(argv, cwd=code, env=env, stdout=out, stderr=err).returncode
    summary = dict(raw=raw, elapsedMs=round((time.monotonic() - started) * 1000), argv=argv)
    save(folder / 'result.json', summary)
    return summary, (folder / 'stdout.log').read_text(encoding='utf-8')

regression, stdout = execute('candidate-regressions', candidate['path'], namespace)
assert regression['raw'] == 0 and 'r7_image_stale_enumeration_type_refuses_both_replacement_directions ... ok' in stdout
assert not list(fixtures.iterdir())

rows = []
for index, label in enumerate(('baseline', 'candidate', 'candidate', 'baseline') * 3, start=1):
    path = baseline if label == 'baseline' else candidate['path']
    run, out = execute(f'{index:02d}-{label}', path, '--exact', exact, '--ignored')
    reports = [json.loads(line.split('R7_PINNED_ENTRY_FULL_VERIFY ', maxsplit=1)[1]) for line in out.splitlines() if 'R7_PINNED_ENTRY_FULL_VERIFY ' in line]
    rows.append(dict(ordinal=index, label=label, raw=run['raw'], report=reports[0] if len(reports) == 1 else None))
    print(rows[-1], flush=True)
assert digest(fixture / '.runtime-image.json') == 'ab9314d3df4475481736245580b24a4d1a6fa52e8030512d9afc9340b6d6c3b6'
assert not list(fixtures.iterdir())
fixtures.rmdir()
valid = all(row['raw'] == 0 and row['report'] and row['report']['budgetMs'] == 10000 and row['report']['completeInventoryAndManifestReadback'] for row in rows)
medians = {label: statistics.median(row['report']['elapsedUs'] for row in rows if row['label'] == label) for label in ('baseline', 'candidate')} if valid else None
save(results / 'comparison.json', dict(valid=valid, regression=regression, fixture=str(fixture), fixtureRootAbsent=not fixtures.exists(), baselineSha256=digest(baseline), candidate=candidate, rows=rows, medianUs=medians, officialExecuted=False, productionAccepted=False))
assert valid
