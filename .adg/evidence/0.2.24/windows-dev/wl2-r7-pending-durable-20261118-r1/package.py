import hashlib
import json
from pathlib import Path
import shutil
import subprocess

run = Path(__file__).resolve().parent
first = run.with_name('r7-pending-durable-20261118-r1')
code = run.parents[3]
docs = Path.cwd()
evidence = docs / '.adg/evidence/0.2.24/windows-dev'
prior = evidence / 'wl2-r7-journal-readback-stages-20261118-r1'
out = evidence / 'wl2-r7-pending-durable-20261118-r1'

def git(*args):
    return subprocess.check_output(['git', *args], cwd=code)

def load(path):
    return json.loads(path.read_bytes())

def save(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as file:
        json.dump(value, file, ensure_ascii=False, indent=2)
        file.write('\n')

def sha(path):
    with path.open('rb') as data:
        return hashlib.file_digest(data, 'sha256').hexdigest()

def copy(origin, target):
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(origin, target)
    assert sha(origin) == sha(target)

def copy_logs(origin, target):
    for name in ('result.json', 'stdout.log', 'stderr.log'):
        copy(origin / name, target / name)

revision = git('rev-parse', 'HEAD').decode().strip()
sources = load(run / 'sources.json')
base = git('rev-parse', 'HEAD^').decode().strip()
assert sources['base'] == base
assert set(sources['sha256ByPath']) == set(git('diff-tree', '--no-commit-id', '--name-only', '-r', 'HEAD').decode().splitlines())
source_binding = []
for path, digest in sources['sha256ByPath'].items():
    current = code / path
    blob = git('show', f'{revision}:{path}')
    assert sha(current) == digest
    assert blob == current.read_bytes().replace(b'\r\n', b'\n')
    source_binding.append({'path': path, 'buildSha256': digest, 'gitBlobSha256': hashlib.sha256(blob).hexdigest()})
transport = run / 'code-transport'
assert any((attempt / 'stdout.log').read_text().split() == [revision, 'refs/heads/codex/024-windows-native'] for attempt in transport.glob('readback*'))
assert all(load(run / phase / 'result.json')['raw'] == 0 for phase in ('compile01', 'bins01'))
assert load(first / 'compile01/result.json')['raw'] == 0
artifact = load(run / 'test-binary/artifact.json')
products = load(run / 'product-artifacts.json')
first_artifact = load(first / 'test-binary/artifact.json')
assert len(products) == 3
assert all(sha(Path(item['retainedPath'])) == item['sha256'] for item in [artifact, first_artifact, *products])
suites = load(run / 'direct-results.json')
assert len(suites) == 7
assert all(suite['raw'] == 0 and suite['sha256'] == artifact['sha256'] for suite in suites)
passed = sum(suite['counts']['passed'] for suite in suites)
assert passed == 28 and all(suite['counts']['failed'] == suite['counts']['ignored'] == 0 for suite in suites)
cleanup = load(run / 'cleanup.json')
assert cleanup['fixtures'] and all(item['absent'] for item in cleanup['fixtures'])
assert not cleanup['unexpectedNewRoots']
roots = len({item['path'] for item in cleanup['fixtures']})
assert cleanup['schema']['absent'] and sha(run / 'generated-windows-schema.json') == cleanup['schema']['sha256']
prior_status = load(prior / 'status.json')
assert prior_status['code'] == base
readback = load(run / 'current-r2-readback.json')
assert len(readback['exportsExact']) == 10
readback_files = evidence / 'wl2-r7-r2-readonly-20261117-receive-r1/raw'
assert all(sha(readback_files / row['name']) == row['sha256'] for row in readback['exportsExact'])

out.mkdir()
for name in ('.gitattributes', 'verify_git_payload.py'):
    copy(prior / name, out / name)
for name in ('hypothesis.json', 'build.py', 'build.ps1', 'sources.json', 'package.py', 'transport.py',
             'run-direct.py', 'direct-results.json', 'fixture-state.json', 'cleanup.ps1', 'cleanup.json',
             'generated-windows-schema.json', 'current-r2-readback.json'):
    copy(run / name, out / name)
for folder in [run / 'compile01', run / 'bins01', *sorted(transport.iterdir()),
               *[item for item in sorted(run.glob('direct-*')) if item.is_dir()]]:
    copy_logs(folder, out / folder.relative_to(run))
for name in ('build.py', 'build.ps1', 'sources.json', 'retain-first.py', 'run-first.py', 'direct-results.json',
             'fixture-state.json', 'generated-windows-schema.json'):
    copy(first / name, out / 'first-attempt' / name)
for folder in ('compile01', 'direct-wl2_r7_pending_attachment_native_'):
    copy_logs(first / folder, out / 'first-attempt' / folder)

save(out / 'source-binding.json', {'source': revision, 'base': base, 'sources': source_binding})
save(out / 'artifact-sha256.json', {'source': revision, 'test': artifact, 'products': products,
                                   'earlierTestSource': 'first-attempt/sources.json', 'earlierTest': first_artifact})
save(out / 'first-errors.json', {
    'firstAttempt': {'cargoRaw': 0, 'wrapperRaw': 1, 'first': 'Generated untracked Windows schema was included in the post-build source allowlist assertion. Cargo logs retained. Wrapper traceback was observed in the tool output.'},
    'finalCompile': None, 'finalNativeTest': None,
    'transportFailures': [load(folder / 'result.json') for folder in sorted(transport.iterdir()) if load(folder / 'result.json')['raw']],
    'currentR2First': 'command-lineage raw64', 'currentR2Job': {'unknown': 24, 'raw': 50},
})
new = [
    {'item': 'r7 isolated future-host V2 partial capture attachment',
     'status': 'Source binds the original D pin, ordinary token, original worker, host, Owner session, capture session and failed Start action. A new recovery action can create, flush and read back only its own two-record attachment. Product Owner projection and real ordinary desktop remain unverified.'},
    {'item': 'r7 V2 partial capture component acceptance',
     'status': f'MSVC raw0 for tests and three development products. {passed} directed tests passed; collision raw183, injected flush raw1117, exact retained prefix and mismatch refusal. {roots} reported fixture roots absent.'},
    {'item': 'Current r2 read-only receipt reconfirmed',
     'status': 'All 10 Public exports match the committed originals. Current Job 24 unknown raw50; first command-lineage raw64 preserved. executionReady=false. No collection rerun or mutation.'},
]
contract = load(run / 'hypothesis.json')
status = {**prior_status, 'code': revision, 'baselineCode': base, 'governance': contract['governance'],
          'baselineStatus': (prior / 'status.json').relative_to(docs).as_posix(), 'matrix': prior_status['matrix'] + new,
          'testPassed': passed, 'cleanupUniqueRoots': roots, 'productionAccepted': False,
          'publicStartOpen': False, 'nativeTestFailure': None}
save(out / 'status.json', status)
summary = f'''输入 main {contract['sharedMain']}、governance {contract['governance']}、TEAM49／IMP188／r7。Windows 源码 {revision} 已推送并独立回读，11 个编译输入按原工作字节和提交 blob 绑定。

仅隔离准入提供私有 D。捕获前绑定原 D pin、普通令牌、原线程、宿主、Owner session、独立 capture session 和失败 Start action。原 worker 在显式恢复动作的原绝对期限内独占创建附件动作目录，写入严格 V2 Intent 和 Pending，两条均 flush，再逐字节回读、复核父目录与原 Owner 后才在私有回执投影 durable=true。部分附件保留诊断，原 Start、未知拒绝、claim／NLK 恢复门槛及公开 Start 拒绝继续有效。

最终 MSVC 测试制品编译和三个开发产品编译均 raw0。7 组定向共 {passed} 项通过。真实同名创建冲突 raw183，注入 flush raw1117；短写、flush、回读注入验证了准确前缀。首轮 Cargo raw0，包装器 raw1 是未排除生成 Windows schema；原日志保留。{roots} 个报告夹具根核实缺席，生成 schema 核摘要归档后清理。四份最终制品摘要见 artifact-sha256.json。真实普通桌面私有 V2 和实际原 Owner 的回执及原预算仍待验，合成测试只覆盖组件。

用户已运行的 r2 入口复核 10 份原输出，逐份大小和摘要匹配。归档沿用。首错 command-lineage raw64；原 Job 24 项 unknown/raw50，executionReady=false。原进程、三原件、锁和 seq9 均保留；未执行现场修复、重跑原包、W2、双 SID 或发布。累计 {len(status['matrix'])} 项状态见 status.json。
'''
(out / 'SUMMARY.md').write_text(summary, encoding='utf-8', newline='\n')
files = [{'path': file.relative_to(out).as_posix(), 'sha256': sha(file), 'bytes': file.stat().st_size}
         for file in sorted(out.rglob('*')) if file.is_file()]
save(out / 'payload-manifest.json', {'files': files})
print(json.dumps({'source': revision, 'passed': passed, 'roots': roots, 'matrixRows': len(status['matrix']), 'payloads': len(files)}))
