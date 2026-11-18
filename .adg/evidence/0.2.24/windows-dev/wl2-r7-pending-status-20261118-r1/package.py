import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

run = Path(__file__).resolve().parent
first = run.with_name('r7-pending-status-20261118-r1')
previous_product = run.with_name('r7-pending-durable-20261118-r2')
docs = Path(sys.argv[1]).resolve()
evidence = docs / '.adg/evidence/0.2.24/windows-dev/wl2-r7-pending-status-20261118-r1'
prior_evidence = evidence.with_name('wl2-r7-pending-durable-20261118-r1')
handoff = docs / 'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'

def load(path):
    return json.loads(path.read_bytes())

def save(path, pdata):
    with path.open('x', encoding='utf-8', newline='\n') as output:
        output.write(json.dumps(pdata, ensure_ascii=False, indent=2) + '\n')

def write(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as output:
        output.write(value)

def digest(path):
    with path.open('rb') as file:
        return hashlib.file_digest(file, 'sha256').hexdigest()

def git(*args):
    return subprocess.check_output(['git', *args])

def copy_file(source, output):
    output.parent.mkdir(parents=True, exist_ok=True)
    assert not output.exists(), output
    shutil.copyfile(source, output)
    assert digest(source) == digest(output)

source = load(run / 'sources.json')
code = git('rev-parse', 'HEAD').decode().strip()
assert git('rev-parse', 'HEAD^').decode().strip() == source['base']
assert set(git('diff', '--name-only', 'HEAD^', 'HEAD').decode().splitlines()) == set(source['sha256ByPath'])
for path, sha in source['sha256ByPath'].items():
    local = Path(path).read_bytes()
    assert hashlib.sha256(local).hexdigest() == sha
    assert local.replace(b'\r\n', b'\n') == git('show', f'{code}:{path}')

for name in ('build.ps1', 'build.py', 'run-projection.py', 'sources.json', 'cleanup.json',
             'generated-windows-schema.json', 'direct-results.json', 'fixture-state.json',
             'archive-first-fixture.ps1', 'archive-first.stderr.log', 'archive-parser.original.ps1', 'package.py'):
    copy_file(run / name, evidence / name)
for name in ('compile01', 'direct-00', 'direct-01', 'first-fixture-archive'):
    for entry in (run / name).iterdir():
        copy_file(entry, evidence / name / entry.name)
for name in ('sources.json', 'compile01/result.json', 'compile01/stdout.log', 'compile01/stderr.log',
             'test-binary/artifact.json', 'direct-00/stdout.log', 'direct-00/stderr.log',
             'first-probe-wrapper.stderr.log', 'first-probe-wrapper-corrected.stderr.log', 'runner-syntax.original.py'):
    copy_file(first / name, evidence / 'first-run' / name)
for name in ('.gitattributes', 'verify_git_payload.py'):
    copy_file(prior_evidence / name, evidence / name)

artifacts = {'test': load(run / 'test-binary/artifact.json'),
             'firstTest': load(first / 'test-binary/artifact.json'),
             'reusedDevelopmentProductsSource': source['base'],
             'reusedDevelopmentProducts': load(previous_product / 'product-artifacts.json')}
for artifact in [artifacts['test'], artifacts['firstTest'], *artifacts['reusedDevelopmentProducts']]:
    assert digest(Path(artifact['retainedPath'])) == artifact['sha256']
save(evidence / 'artifact-sha256.json', artifacts)
save(evidence / 'source-binding.json', {'code': code, **source,
    'main': '6bafdd342e69ee83e842f4b35a20bafa1320645a',
    'governance': '6ef21eb5f144905318d94f5080b2acf408e258bf', 'team': 50, 'imp': 189,
    'contract': 'r7', 'testOnly': True,
    'gitBlobNormalizedBytesMatchBuiltWorktree': True})
first_errors = {
    'firstCompile': load(first / 'compile01/result.json'),
    'firstFixture': {'failed': 1, 'processRawRecorded': False,
        'actual': 'LostWorkerUnknown', 'expected': 'FailedRetained',
        'reason': 'empty thread registration production observer classifies as finished',
        'fix': 'register a live owned fixture thread and join it'},
    'wrappers': [
        {'firstError': 'Python source generation broke newline escapes; SyntaxError, raw1', 'evidence': 'first-run/first-probe-wrapper.stderr.log'},
        {'firstError': 'First runner count parser failed after native test; raw1, original native output retained', 'evidence': 'first-run/first-probe-wrapper-corrected.stderr.log'},
        {'firstError': 'First archive wrapper had invalid -cjoin, raw1; no file action began', 'evidence': 'archive-parser.original.ps1'}],
    'currentR2': {'first': 'command-lineage raw64', 'jobUnknown': 24, 'jobRaw': 50, 'executionReady': False},
}
save(evidence / 'first-errors.json', first_errors)
compile_ms = load(run / 'compile01/result.json')['elapsedMs']
status = load(prior_evidence / 'status.json')
status.update(code=code, baselineCode=source['base'],
    baselineStatus=f'{prior_evidence.relative_to(docs).as_posix()}/status.json',
    governance='6ef21eb5f144905318d94f5080b2acf408e258bf', team=50, imp=189,
    testPassed=2, cleanupUniqueRoots=6,
    nativeTestFailure='first fixture harness lacked a registered live worker; corrected fixture passes')
for item, result in (
    ('r7 native V2 attachment result through production Status projection', 'Synthetic retained-worker state on a live fixture thread. Actual success durable=true, native collision raw183 and injected short-write, flush raw1117, readback durable=false. Original Start raw64 and both record digests preserved. Running and LostWorkerUnknown omit the attachment. Expired read refuses. 2 affected tests raw0.'),
    ('r7 partial preparation Source ordered full scan analysis', 'Conditional three complete source scans around two v2_reference_data reads under one original absolute recovery cutoff. No scan or verification boundary removed. Fixed copy component and timed-out synthetic recovery evidence reused; live Owner time unmeasured.'),
    ('r7 actual retained Owner entry admission dependency', 'Only current ExistingQualification pins D and requires exact old r2 I/D/H, ordinary broker token, original client/coordination references. Live actual Owner attachment and original deadline pending a contract-compliant separate fresh admission. Current r2 originals and Job Unknown forbid a rerun.'),
    ('r7 projection fixture cleanup and source binding', 'Five final success roots absent. First failure root archived byte-for-byte then exact owned members and empty directories removed. Generated schema archived and absent. MSVC two test EXEs retained in ignored target. Product EXEs reused from the prior production-identical source.'),
):
    status['matrix'].append({'item': item, 'status': result})
save(evidence / 'status.json', status)
summary = f'''# r7 V2 保全回执的 Status 定向验证

本轮使用 main 6bafdd342e69ee83e842f4b35a20bafa1320645a、治理 6ef21eb5f144905318d94f5080b2acf408e258bf、TEAM50／IMP189／r7。代码为 {code}，三处改动均为测试。开发产品制品沿用上一源码 {source['base']}，生产实现相同。

| 项目 | 结果 |
|---|---|
| MSVC | 首轮与最终测试编译 raw0。最终 {compile_ms} ms。受影响定向 2/2。 |
| Status 投影 | 使用实际原生附件写入结果及存活夹具线程。成功 durable=true。同名冲突 raw183，短写、flush raw1117、回读注入为 durable=false。首个 Start raw64、双记录摘要、附件原始错误和恢复 Required 保持。Running/LostWorkerUnknown 隐去附件。读取已过期拒绝。 |
| 首错 | 首轮定向夹具登记的线程槽为空，实际 Status 正确返回 LostWorkerUnknown，原测试期待 FailedRetained。保留原标准输出和错误。首轮包装器在解析计数前退出，未单独留存原测试退出码。 |
| 清理 | 最终五个夹具根缺席。首轮失败的两份原始记录核摘要归档后，清理已核定的自有文件及空目录。生成 schema 归档后缺席。 |
| 原包扫描 | 原 Source 存在且此前各步成功时，两次 V2 引用读取前后到达三次完整扫描，共用原截止，具体调用链见 source-call-chain.md。保留全部校验。 |
| 实际 Owner／普通桌面 | 保全附件私有 D pin 入口依赖旧 r2 I/D/H、原合成 Profile 和协调对象。真实回执与原 10 秒未验。独立的新任务必须从合同认可的真实普通桌面基线入场。 |
| 当前 r2 | 沿用全归档和十份回执，首错 command-lineage raw64，原 Job 24 项未知、raw50，executionReady=false。旧进程、三原件、锁、seq9 保留。 |

累计 {len(status['matrix'])} 项状态见 status.json。制品摘要、夹具原记录、包装器及原编译日志随附。公开 Start 保持拒绝。
'''
write(evidence / 'SUMMARY.md', summary)
with handoff.open('a', encoding='utf-8', newline='\n') as output:
    output.write(f'''
## 2026-11-18 r7 V2 保全附件 Status 投影与真实入场约束

代码 {code}，输入 main 6bafdd342e69ee83e842f4b35a20bafa1320645a、governance 6ef21eb5f144905318d94f5080b2acf408e258bf、TEAM50／IMP189／r7。仅测试改动。最终 MSVC 测试编译 raw0，受影响定向 2/2。实际附件成功 durable=true；原生同名冲突 raw183，短写、flush raw1117、回读注入均 durable=false。运行中和原线程失联不投影附件，过时 Status 拒绝。使用真实存活的自有夹具线程和合成 worker 状态，仍缺实际 Owner 回执。首轮测试缺少存活线程登记，源码观察正确返回 LostWorkerUnknown。原日志和两份失败夹具记录保全，六个本轮夹具根及生成 schema 已核实清理。

同一原恢复截止内，Source 存在且各检查通过时，两次原 V2 引用实际读取前中后三次调用完整包验证，source-call-chain.md 列调用与覆盖。上一固定副本单扫 2634.174 ms、仿真整链／部分解锁 raw101 沿用。没有跨两次引用读取锁住同一整包对象及字节的等价证据，现有三次完整扫描保持。下一次真实普通桌面应记录每次完整扫描进入、退出、失败阶段及余量，沿现有 r7 接收规则复核。

目前私有 D pin 唯一入场绑定旧 ExistingQualification 的固定 r2 I/D/H、原 SID/session、原 Profile 客户端和原协调对象。原 Job 24 项未知 raw50，executionReady=false，三个原件仍保管。真实实际 Owner V2 回执和恢复预算依赖新的合同合规普通桌面入场与实际同线程保全前缀；请在后续 handoff 接收时保留为待验。当前 R1 的 Job 覆盖和单独授权仍是独立依赖。既有普通 MSI、CAP、原缺席退根和固定原包单 SID 生产闭环继续待验。公开 Start 保持拒绝。证据 wl2-r7-pending-status-20261118-r1，累计 {len(status['matrix'])} 项。
''')
payload = [{'path': path.relative_to(evidence).as_posix(), 'bytes': path.stat().st_size, 'sha256': digest(path)}
           for path in sorted(evidence.rglob('*')) if path.is_file()]
save(evidence / 'payload-manifest.json', {'files': payload})
print(code, len(payload) + 1, len(status['matrix']))
