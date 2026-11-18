import hashlib
import json
from pathlib import Path
import shutil
import subprocess

run = Path(__file__).resolve().parent
code = run.parents[3]
docs = Path.cwd()
directory = docs / '.adg/evidence/0.2.24/windows-dev'
prior = directory / 'wl2-r7-journal-replay-cost-20261118-r1'
out = directory / 'wl2-r7-journal-readback-stages-20261118-r1'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=code)


def load(path):
    return json.loads(path.read_bytes())


def save(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as handle:
        json.dump(value, handle, ensure_ascii=False, indent=2)
        handle.write('\n')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def copy(source, target):
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)


revision = git('rev-parse', 'HEAD').decode().strip()
base = git('rev-parse', 'HEAD^').decode().strip()
sources = load(run / 'sources.json')
assert sources['base'] == base
assert set(sources['sha256ByPath']) == set(git('diff-tree', '--no-commit-id', '--name-only', '-r', 'HEAD').decode().splitlines())
binding = []
for path, build_sha in sources['sha256ByPath'].items():
    current = (code / path).read_bytes()
    blob = git('show', f'{revision}:{path}')
    assert sha(current) == build_sha and blob == current.replace(b'\r\n', b'\n')
    binding.append({'path': path, 'buildSha256': build_sha, 'gitBlobSha256': sha(blob)})

transport = run / 'code-transport'
readbacks = sorted(transport.glob('readback*'))
assert any((attempt / 'stdout.log').read_text().split() == [revision, 'refs/heads/codex/024-windows-native'] for attempt in readbacks)
previous_readback = run.parent / 'r7-journal-replay-cost-20261118-r1/docs-transport/evidence-readback01'
assert (previous_readback / 'stdout.log').read_text().split() == ['db96a3ebe7d816c081226297787c8a1fb0463d41', 'refs/heads/docs/024-windows-dev']
status = load(prior / 'status.json')
assert status['code'] == base
compile_reports = [load(run / phase / 'result.json') for phase in ('compile01', 'bins01')]
assert all(report['raw'] == 0 for report in compile_reports)
artifact = load(run / 'test-binary/artifact.json')
products = load(run / 'product-artifacts.json')
assert len(products) == 3
assert all(sha(Path(item['retainedPath']).read_bytes()) == item['sha256'] for item in [artifact, *products])
suites = load(run / 'direct-results.json')
assert len(suites) == 3 and all(suite['raw'] == 0 and suite['sha256'] == artifact['sha256'] for suite in suites)
passed = sum(suite['counts']['passed'] for suite in suites)
assert passed == 5 and all(suite['counts']['failed'] == suite['counts']['ignored'] == 0 for suite in suites)
cleanup = load(run / 'cleanup.json')
assert cleanup['fixtures'] and all(item['absent'] and not item['error'] for item in cleanup['fixtures'])
roots = len({item['path'] for item in cleanup['fixtures']})
assert cleanup['schema']['absent']
assert sha((run / 'generated-windows-schema.json').read_bytes()) == cleanup['schema']['sha256']
measurement = load(run / 'measurement.json')
stages = measurement['stages']

out.mkdir()
for name in ('.gitattributes', 'verify_git_payload.py'):
    copy(prior / name, out / name)
for name in ('hypothesis.json', 'build.py', 'build-bins.py', 'build-bins.ps1', 'sources.json',
             'package.py', 'run-direct.py', 'measure.py', 'measurement.json', 'cleanup.ps1', 'transport.py',
             'direct-results.json', 'fixture-state.json', 'cleanup.json', 'generated-windows-schema.json'):
    copy(run / name, out / name)
folders = [run / 'compile01', run / 'bins01', *sorted(transport.iterdir()),
           *[item for item in sorted(run.glob('direct-*')) if item.is_dir()]]
for folder in folders:
    for name in ('result.json', 'stdout.log', 'stderr.log'):
        copy(folder / name, out / folder.relative_to(run) / name)
for name in ('result.json', 'stdout.log', 'stderr.log'):
    copy(previous_readback / name, out / 'previous-docs-readback' / name)

save(out / 'source-binding.json', {'source': revision, 'base': base, 'sources': binding})
save(out / 'artifact-sha256.json', {'source': revision, 'test': artifact, 'products': products})
transport_errors = [load(folder / 'result.json') for folder in sorted(transport.iterdir()) if load(folder / 'result.json')['raw']]
save(out / 'first-errors.json', {'compile': None, 'native': None, 'transportFailures': transport_errors,
                                'currentR2First': 'command-lineage raw64', 'currentR2Job': {'unknown': 24, 'raw': 50}})
new = [
    {'item': 'r7 complete native Journal readback grouped by actual phase',
     'status': f"{measurement['wholeReadbacks']} reads; {measurement['wholeReadbackUs']} us. Fresh plus retained record checks accounted for {measurement['perRecordPercent']:.2f}%; Current bytes and typed validation {measurement['currentBytesAndTypedPercent']:.2f}%. Group coverage and bytes verified."},
    {'item': 'r7 Journal stage instrumentation gates',
     'status': f'MSVC raw0 for tests and three development products, {passed} scoped tests passed and {roots} reported fixture roots absent. No stage removed. Original action budgets preserved.'},
]
save(out / 'status.json', {**status, 'code': revision, 'baselineCode': base,
                           'baselineStatus': (prior / 'status.json').relative_to(docs).as_posix(),
                           'matrix': status['matrix'] + new, 'testPassed': passed, 'cleanupUniqueRoots': roots,
                           'productionAccepted': False, 'publicStartOpen': False, 'nativeTestFailure': None})
summary = f'''输入 main {status['main']}、governance {status['governance']}、TEAM48／IMP187／r7。Windows 源码 {revision} 已推送并独立回读。阶段计量只在指定合成测试开启。完整观察次序、保留句柄寿命、字节比较、typed Current、成员枚举、poison、拒绝和截止均保留。

{measurement['appends']} 次追加及最终独立复查共 {measurement['wholeReadbacks']} 次完整 Journal 读回，合计 {measurement['wholeReadbackUs']} μs。逐记录 fresh 核验 {stages['journal-record-fresh']['sumUs']} μs，保留记录核验和字节读取 {stages['journal-record-retained-read']['sumUs']} μs，合占 {measurement['perRecordPercent']:.2f}%。后者包含保留 pin、私有安全、对象身份及长度检查。Current 单独字节读取 {stages['journal-current-bytes']['sumUs']} μs，纯 typed Current 核验 {stages['journal-current-typed']['sumUs']} μs，合占 {measurement['currentBytesAndTypedPercent']:.2f}%。其余阶段及未归属耗时见 measurement.json。每组顺序和预期字节均核对。最后读回结束时原 10 秒尚余 {measurement['lastCheckRemainingUs']} μs。单个 MSVC debug 合成前缀只说明本样本成本，不签真实恢复预算，也不构成移除任何核验的等价证明。

MSVC 定向测试编译 raw0，{passed} 项通过，MSVC 开发版三个产品编译 raw0。测试 SHA-256 {artifact['sha256']}。{roots} 个报告夹具根核实缺席，生成 Windows schema 经归档核摘要后清理。当前 r2 继续使用已经核验的十份只读回执，首错 command-lineage raw64，原 Job 24 项未知、raw50、executionReady=false。普通桌面私有 V2 及严格 V2 Pending 首次耐久边界待验证。公开 Start 保持拒绝，未运行原包、W2、双 SID 或发布。
'''
(out / 'SUMMARY.md').write_text(summary, encoding='utf-8', newline='\n')
files = []
for file in sorted(out.rglob('*')):
    if file.is_file():
        data = file.read_bytes()
        files.append({'path': file.relative_to(out).as_posix(), 'sha256': sha(data), 'bytes': len(data)})
save(out / 'payload-manifest.json', {'files': files})
print(json.dumps({'source': revision, 'passed': passed, 'roots': roots,
                  'matrixRows': len(status['matrix']) + len(new), 'payloads': len(files)}))

