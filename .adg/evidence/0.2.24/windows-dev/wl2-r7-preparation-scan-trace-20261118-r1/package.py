import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

run = Path(__file__).resolve().parent
docs = Path(sys.argv[1]).resolve()
previous = docs / '.adg/evidence/0.2.24/windows-dev/wl2-r7-pending-status-20261118-r1'
evidence = previous.with_name('wl2-r7-preparation-scan-trace-20261118-r1')
handoff = docs / 'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'

def load(path):
    return json.loads(path.read_bytes())

def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

def write(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as output:
        output.write(value)

def save(path, value):
    write(path, json.dumps(value, ensure_ascii=False, indent=2) + '\n')

def git(*args):
    return subprocess.check_output(['git', *args])

def copy_file(source, name):
    output = evidence / name
    output.parent.mkdir(parents=True, exist_ok=True)
    assert not output.exists(), output
    shutil.copyfile(source, output)
    assert digest(source) == digest(output)

source = load(run / 'sources.json')
code = git('rev-parse', 'HEAD').decode().strip()
assert git('rev-parse', 'HEAD^').decode().strip() == source['base']
assert set(git('diff', '--name-only', 'HEAD^', 'HEAD').decode().splitlines()) == set(source['sha256ByPath'])
for path, sha in source['sha256ByPath'].items():
    raw = Path(path).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == sha
    assert raw.replace(b'\r\n', b'\n') == git('show', f'{code}:{path}')
compile_result = load(run / 'compile01/result.json')
direct = load(run / 'direct-results.json')
fixture = load(run / 'fixture-state.json')
cleanup = load(run / 'cleanup.json')
scans = load(run / 'observed-scans.json')
artifact = load(run / 'test-binary/artifact.json')
product_compile = load(run / 'bins01/result.json')
products = load(run / 'product-artifacts.json')
assert product_compile['raw'] == 0
assert all(digest(Path(item['retainedPath'])) == item['sha256'] for item in [artifact, *products])
assert compile_result['raw'] == 0 and len(direct) == 1 and direct[0]['raw'] == 0
assert len(fixture) == 1 and fixture[0]['absent'] and cleanup['schema']['absent']
assert cleanup['schema']['sha256'] == digest(run / 'generated-windows-schema.json')
assert len(scans) == 3 and len({row['deadlineUs'] for row in scans}) == 1
assert [row['completed'] for row in scans] == [True, True, False]
assert scans[-1]['errorCode'] == 'runtime_image_unconfirmed'
for name in ('build.ps1', 'build.py', 'sources.json', 'hypotheses.json', 'run-scans.py', 'package.py',
             'direct-results.json', 'fixture-state.json', 'observed-scans.json', 'cleanup.json',
             'generated-windows-schema.json', 'test-binary/artifact.json', 'ordinary-owner-admission-request.json',
             'product-artifacts.json', 'product-schema/cleanup.json', 'product-schema/generated-windows-schema.json'):
    copy_file(run / name, name)
for name in ('compile01', 'bins01', 'direct-00'):
    for file in sorted((run / name).iterdir()):
        copy_file(file, f'{name}/{file.name}')
for name in ('.gitattributes', 'verify_git_payload.py'):
    copy_file(previous / name, name)
save(evidence / 'source-binding.json', {'code': code, **source, 'governance': load(run / 'hypotheses.json')['input'],
    'testInstrumentationOnly': True, 'gitBlobNormalizedBytesMatchBuiltWorktree': True})
save(evidence / 'artifact-sha256.json', {'test': artifact, 'developmentProductsSource': code, 'developmentProducts': products})
status = load(previous / 'status.json')
status.update(code=code, baselineCode=source['base'], baselineStatus=f'{previous.relative_to(docs).as_posix()}/status.json',
    testPassed=1, cleanupUniqueRoots=1, nativeTestFailure=None)
for item, result in (
    ('r7 preparation Source full image call site measurement', 'Test-only ordinals 1/2/3 at the existing preparation fences. At the existing complete verification call the optional same-thread observer records failed Start action, source digest, entry/exit and unchanged cutoff, budget, outcome and original managed error class. All native validation remains.'),
    ('r7 preparation Source native scan measurement component', 'A synthetic selected image was scanned three times by the actual Windows verifier under the same original cutoff. Two clean passes; third rejected a same-size payload change with the same error as direct verification. No real Owner or original official package executed, no budget acceptance.'),
    ('r7 preparation Source scan fixture cleanup', 'One new owned fixture root absent, no unexpected root, generated Windows schema digest-archived and absent. Exact raw MSVC compilation and test logs preserved.'),
):
    status['matrix'].append({'item': item, 'status': result})
save(evidence / 'status.json', status)
rows = '\n'.join(f"| {row['ordinal']} | {row['startedUs']} | {row['finishedUs']} | {row['remainingBeforeUs']} | {row['remainingAfterUs']} | {row['errorCode'] or '通过'} |" for row in scans)
write(evidence / 'SUMMARY.md', f'''# r7 准备前缀原包扫描观测

代码 {code}，治理输入 TEAM50／IMP189／r7。三处实际准备围栏保留全量原像校验。在 MSVC 测试构建中可在调用点观察序号、失败 Start action、来源摘要、原截止、进入退出、剩余预算和原结果。默认不启用记录，启用时在原线程缓冲三个观察结果，验后输出。

MSVC 测试编译 raw0／{compile_result['elapsedMs']} ms。原生定向 1/1，进程 raw0／{direct[0]['elapsedMs']} ms。下表为自有合成镜像的原生完整扫描。三个样本的共同截止偏移是 {scans[0]['deadlineUs']} μs。

| 序号 | 进入 μs | 退出 μs | 进入余量 μs | 退出余量 μs | 返回 |
| --- | --- | --- | --- | --- | --- |
{rows}

第三次读到等长内容变化，返回了与直接复验一致的 runtime_image_unconfirmed。定向走实际 Windows 整树及根 manifest 验证调用，尚未到达真实 Owner 的三个围栏。时间属于合成组件。固定副本原生子阶段沿用前次证据，正式原包、普通桌面及真实恢复预算继续待验。

本批一个自有夹具根缺席，两次构建生成的 schema 均核 SHA 后独立归档清理。三个开发产品完成原生构建，摘要见 artifact-sha256.json。当前 r2 仍为原 Job 24 项未知／raw50，executionReady=false，原进程与原件继续保管。累计 {len(status['matrix'])} 项见 status.json。
''')
with handoff.open('a', encoding='utf-8', newline='\n') as output:
    output.write(f'''
## 2026-11-18 r7 准备前缀的完整扫描调用计量

代码 {code}，main 6bafdd342e69ee83e842f4b35a20bafa1320645a、governance 6ef21eb5f144905318d94f5080b2acf408e258bf，TEAM50／IMP189／r7。用测试构建计量原准备围栏在实际全量 verify 进入与退出的序号、失败 Start action、来源 SHA、同一截止、剩余预算和实际失败。原 Source/Current/两次原引用读回及校验顺序不变。MSVC raw0／{compile_result['elapsedMs']} ms，三个产品 MSVC 开发构建 raw0／{product_compile['elapsedMs']} ms。原生定向 1/1。合成镜像同一截止下前两次成功，第三次等长字节改变仍返回 runtime_image_unconfirmed，与直接复验一致。一个自有夹具根和生成 schema 核实清理。证据 wl2-r7-preparation-scan-trace-20261118-r1。

合成镜像仅证明调用点观测与原验证结果一致。真实 Owner 同线程三次围栏及原 10 秒预算仍待独立普通桌面入场。独立入场的最小合同请求见同轮 ordinary-owner-admission-request.json。固定副本计量与合成端到端超时沿用。当前 r2 原 Job 完整覆盖和新的现场执行授权仍是独立依赖，公开 Start 保持拒绝。
''')
payload = [{'path': path.relative_to(evidence).as_posix(), 'bytes': path.stat().st_size, 'sha256': digest(path)}
           for path in sorted(evidence.rglob('*')) if path.is_file()]
save(evidence / 'payload-manifest.json', {'files': payload})
print(code, len(payload) + 1, len(status['matrix']))