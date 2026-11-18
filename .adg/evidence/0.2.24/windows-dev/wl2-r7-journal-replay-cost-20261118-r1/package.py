import hashlib
import json
from pathlib import Path
import shutil
import subprocess

run = Path(__file__).resolve().parent
code = run.parents[3]
docs = Path.cwd()
directory = docs / '.adg/evidence/0.2.24/windows-dev'
prior = directory / 'wl2-r7-journal-append-readback-20261118-r1'
out = directory / 'wl2-r7-journal-replay-cost-20261118-r1'


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
sources = load(run / 'sources02.json')
assert sources['base'] == base
assert set(sources['sha256ByPath']) == set(git('diff-tree', '--no-commit-id', '--name-only', '-r', 'HEAD').decode().splitlines())
binding = []
for path, build_sha in sources['sha256ByPath'].items():
    current = (code / path).read_bytes()
    blob = git('show', f'{revision}:{path}')
    assert sha(current) == build_sha and blob == current.replace(b'\r\n', b'\n')
    binding.append({'path': path, 'buildSha256': build_sha, 'gitBlobSha256': sha(blob)})
assert (run / 'code-transport/readback01/stdout.log').read_text().split() == [revision, 'refs/heads/codex/024-windows-native']
prior_remote = run.parent / 'r7-journal-append-readback-20261118-r1/docs-transport/evidence-readback01'
assert (prior_remote / 'stdout.log').read_text().split() == ['99520867ba3836b5443f61fc4620e6968083987c', 'refs/heads/docs/024-windows-dev']
status = load(prior / 'status.json')
assert status['code'] == base
failed = load(run / 'compile01/result.json')
build = load(run / 'compile02/result.json')
assert failed['raw'] == 101 and build['raw'] == 0
artifact = load(run / 'test-binary/artifact.json')
assert sha(Path(artifact['retainedPath']).read_bytes()) == artifact['sha256']
suites = load(run / 'direct-results.json')
assert len(suites) == 3 and all(r['raw'] == 0 and r['sha256'] == artifact['sha256'] for r in suites)
passed = sum(r['counts']['passed'] for r in suites)
assert passed == 5 and all(r['counts']['failed'] == r['counts']['ignored'] == 0 for r in suites)
cleanup = load(run / 'cleanup.json')
assert cleanup['fixtures'] and all(r['absent'] and not r['error'] for r in cleanup['fixtures'])
roots = len({r['path'] for r in cleanup['fixtures']})
assert cleanup['schema']['absent'] and sha((run / 'generated-windows-schema.json').read_bytes()) == cleanup['schema']['sha256']
measurement = load(run / 'measurement.json')
replay = measurement['independentReplay']
native = measurement['nativeJournalReadback']
policy = measurement['estimatedPolicyShareOfReplayPercent']
percent = measurement['replayToNativeReadbackPercent']

out.mkdir()
for name in ('.gitattributes', 'verify_git_payload.py'):
    copy(prior / name, out / name)
for name in ('hypothesis.json', 'build.py', 'build02.py', 'prepare02.py', 'sources.json', 'sources02.json',
             'package.py', 'run-direct.py', 'measure.py', 'measurement.json', 'cleanup.ps1', 'transport.py',
             'direct-results.json', 'fixture-state.json', 'cleanup.json', 'generated-windows-schema.json'):
    copy(run / name, out / name)
for folder in ('compile01', 'compile02', 'code-transport/push01', 'code-transport/push02', 'code-transport/readback01', 'code-transport/fetch01',
               *[p.name for p in sorted(run.glob('direct-*')) if p.is_dir()]):
    for name in ('result.json', 'stdout.log', 'stderr.log'):
        copy(run / folder / name, out / folder / name)
for name in ('result.json', 'stdout.log', 'stderr.log'):
    copy(prior_remote / name, out / 'previous-docs-readback' / name)

save(out / 'failed-source-snapshot-sha256.json', {path.name: sha(path.read_bytes()) for path in sorted(run.glob('compile01-*.rs'))})
save(out / 'source-binding.json', {'source': revision, 'base': base, 'sources': binding})
save(out / 'artifact-sha256.json', {'source': revision, 'test': artifact, 'products': None,
                                    'previousProductEvidence': (prior / 'artifact-sha256.json').relative_to(docs).as_posix()})
save(out / 'first-errors.json', {'compile': {'raw': 101, 'message': 'Rust expected comma in test-only JSON println format'},
                                'finalCompile': None, 'native': None, 'currentR2First': 'command-lineage raw64',
                                'currentR2Job': {'unknown': 24, 'raw': 50}})
new = [
    {'item': 'r7 isolated journal append replay versus full native readback',
     'status': f'{measurement["appends"]} appends within original 10 s; paired native readback {native["sumUs"]} us, replay {replay["sumUs"]} us ({percent:.2f}% of native readback). No product acceptance.'},
    {'item': 'r7 pure fixed policy digest versus event digest calibration',
     'status': f'8192 iterations each. Estimated fixed-policy share {policy:.2f}% of replay in this fixture, separate loop. No production caching or skipped fences.'},
    {'item': 'r7 journal replay cost MSVC and clean fixture roots',
     'status': f'Compile raw101 retained, corrected compile raw0, {passed} tests passed; {roots} fixture roots absent and generated Windows schema archived/absent.'},
]
save(out / 'status.json', {**status, 'code': revision, 'baselineCode': base,
                           'baselineStatus': (prior / 'status.json').relative_to(docs).as_posix(),
                           'matrix': status['matrix'] + new, 'testPassed': passed, 'cleanupUniqueRoots': roots,
                           'productionAccepted': False, 'publicStartOpen': False, 'nativeTestFailure': None})
summary = f'''输入 main {status['main']}、governance {status['governance']}、TEAM48／IMP187／r7。Windows 源码 {revision} 已推送并回读；仅计量合成 Journal 的独立重放，生产仍沿用原核验、重放和回写顺序。上批证据 99520867ba3836b5443f61fc4620e6968083987c 已回读。

MSVC 首编译 raw101，测试日志格式解析错误，原日志在 compile01。修正后 raw0、{build['elapsedMs']}ms。三个定向组共 {passed} 项通过。TEST EXE SHA-256 {artifact['sha256']}。本批未构建产品制品。上一批三个产品制品按其自己的源码和证据范围使用。

{measurement['appends']} 次合成追加均在原 10 秒截止内完成。追加前后的 {native['count']} 次完整 Journal 读回共 {native['sumUs']}μs，独立重放共 {replay['sumUs']}μs，为前者的 {percent:.2f}%。最终独立复查未计入比较，截止尚余 {measurement['lastCheckRemainingUs']}μs。固定策略摘要与事件编码加摘要各用 8192 次纯数据循环校准。固定策略摘要估计占该夹具重放的 {policy:.2f}%，独立循环不代表原生调用动态计数。完整读回包含原始字节与 typed Current 核验。写入、同步后的读回和 CAS 另有开销，本批未计入两项比较。样本是一个 debug MSVC 合成节点前缀，无法给出完整恢复或原包预算结论。本批保留生产所有检查。

{roots} 个新报告夹具根核实缺席。生成 schema 归档核摘要后清理。历史失败和拒删路径继续保留。当前 r2 全归档已核验，首错 command-lineage raw64，原 Job 24 项未知、raw50、executionReady=false。普通桌面私有 V2 未收件，严格 V2 Pending 首次耐久边界继续等 Git 交接。公开 Start 保持拒绝，未运行原包、W2、双 SID 或发布。完整状态共 {len(status['matrix']) + len(new)} 项见 status.json。
'''
(out / 'SUMMARY.md').write_text(summary, encoding='utf-8', newline='\n')
files = []
for file in sorted(out.rglob('*')):
    if file.is_file():
        raw = file.read_bytes()
        files.append({'path': file.relative_to(out).as_posix(), 'sha256': sha(raw), 'bytes': len(raw)})
example = load(prior / 'payload-manifest.json')
print('manifest structure:', list(example))
save(out / 'payload-manifest.json', {'files': files})
print(json.dumps({'source': revision, 'passed': passed, 'roots': roots, 'matrixRows': len(status['matrix']) + len(new), 'payloads': len(files)}))
