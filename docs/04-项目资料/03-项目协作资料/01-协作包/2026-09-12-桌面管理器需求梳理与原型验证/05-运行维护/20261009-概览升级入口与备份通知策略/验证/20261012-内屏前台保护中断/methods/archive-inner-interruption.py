from pathlib import Path
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import subprocess
import sys

ROOT = Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop')
CODE = Path('/Users/ezio/.codex/worktrees/ocx-0110-main')
RAW = Path('/private/tmp/ocx-0110-performance-20261012/candidate0110')
UI = Path('/private/tmp/ocx-0110-performance-guard-20261012')
REL = Path('docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-内屏前台保护中断')
OUT = ROOT / REL
IMP = Path('docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md')
RCP = Path('.adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-INNER-FG-20261012.json')
TASK = ROOT / '.adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def git(cwd, *args):
    return subprocess.check_output(['git', *args], cwd=cwd)

def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x') as stream:
        stream.write(json.dumps(value, ensure_ascii=False, indent=2) + chr(10))

assert git(ROOT, 'branch', '--show-current').decode().strip() == 'docs/governance-main'
assert git(ROOT, 'rev-parse', 'HEAD').decode().strip() == '1bbbb8f1fb01d83640f86eee42bbb811146b8b2a'
assert not git(ROOT, 'diff', '--cached', '--name-only')
before_status = Path('/private/tmp/ocx-0110-pre-actual-write-20261012.status').read_bytes()
assert git(ROOT, 'status', '--porcelain=v1', '-z', '--untracked-files=all') == before_status
assert git(CODE, 'rev-parse', 'HEAD').decode().strip() == 'fb34f5b91c3a6feb5b41f11f9527171c4bf742f5'
assert not git(CODE, 'diff', '--name-only')
assert not git(CODE, 'diff', '--cached', '--name-only')
assert not RCP.exists() and not (ROOT / RCP).exists()
task_sha = sha(TASK)
task = json.loads(TASK.read_text())
assert task['state'] == 'in_progress' and task['revision'] == 2
assert task['content_digest'] == '23908e0d312409d81886b6f0f570c39581d8f9fc413ac67b0d043102eecfc29c'
OUT.mkdir()
mapping = []

def copy(source, destination):
    target = OUT / destination
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open('xb') as stream:
        stream.write(source.read_bytes())
    assert target.read_bytes() == source.read_bytes()
    mapping.append({'source': str(source), 'archive': destination, 'size_bytes': target.stat().st_size, 'sha256': sha(target), 'copy': 'byte_identical'})

runs = ['guarded-mid-finder-20261012', 'guarded-mid-prearmed-20261012', 'guarded-mid-inner-20261012']
metrics = []
for run in runs:
    path = RAW / run
    for name in ['foreground.jsonl', 'guard-context.json', 'guard-result.json', 'settings-before.json', 'settings-after.json', 'launch.json']:
        if (path / name).exists():
            copy(path / name, 'runs/' + run + '/' + name)
    events = [json.loads(line) for line in (path / 'foreground.jsonl').read_text().splitlines()]
    samples = [row for row in events if row['kind'] == 'sample']
    valid = [row for row in samples if row['target_frontmost'] and row['target_window_on_screen']]
    invalid = [row for row in samples if not (row['target_frontmost'] and row['target_window_on_screen'])]
    result = json.loads((path / 'guard-result.json').read_text())
    pre = json.loads((path / 'settings-before.json').read_text())
    post = json.loads((path / 'settings-after.json').read_text())
    assert pre['preferences_sha256'] == post['preferences_sha256'] == 'aed374985a30e077a7093a4ea5983b5f7b61bce40cee7ccbf5cc957f19e9e8aa'
    assert pre['appearance'] == post['appearance'] == {'glow_render': 'mesh', 'interface_scale': 100, 'theme': 'light', 'visual_effects': 'mid'}
    assert result['resource_collector_exit'] is None
    assert not (path / 'summary.json').exists()
    assert (path / 'collector.stdout.log').stat().st_size == 0
    metrics.append({
        'run': run, 'event_count': len(events), 'event_kinds': dict(Counter(row['kind'] for row in events)),
        'sample_count': len(samples), 'valid_sample_count': len(valid), 'invalid_sample_count': len(invalid),
        'first_sample': samples[0], 'last_valid_sample': valid[-1] if valid else None,
        'first_invalid_sample': invalid[0] if invalid else None, 'ready_events': [row for row in events if row['kind'] == 'ready'],
        'resource_sample_count': 0, 'resource_collector_started': False, 'warmup_completed': False,
        'requested_warmup_seconds': 90, 'requested_measurement_seconds': 180,
        'result': result, 'preferences_sha256_unchanged': True,
        'actual_appearance': pre['appearance'], 'original_manifest_quality': 'high',
        'performance_acceptance': 'not_evaluated',
    })
inner = metrics[-1]
assert inner['event_count'] == 172 and inner['valid_sample_count'] > 0
assert abs(inner['first_invalid_sample']['elapsed_s'] - 18.75080479157623) < 0.000001
assert inner['first_invalid_sample']['target_window_on_screen'] is True
assert all(m['resource_sample_count'] == 0 for m in metrics)

for folder in ['ui-guarded-mid-inner-20261012', 'ui-guarded-mid-finder-20261012']:
    for path in sorted((UI / folder).iterdir()):
        assert path.name.endswith(('.jpg', '.txt', '.json'))
        if path.is_file():
            copy(path, 'ui/' + folder + '/' + path.name)
copy(UI / 'native-readonly-check.swift', 'methods/native-readonly-check.swift')
copy(Path(__file__), 'methods/archive-inner-interruption.py')
save(OUT / 'run-metrics.json', {'derivation': 'target-only foreground.jsonl and guard-result/settings files; no resource collector output', 'runs': metrics})
save(OUT / 'placement-transcript-derived.json', {
    'provenance': 'derived from prior tool transcript, not a byte-identical saved native JSON result',
    'observed_at': '2026-10-12T02:09:33Z', 'target_pid': 31501,
    'target_running': True, 'target_active': True, 'workspace_frontmost_is_target': True,
    'ax_status': -25204, 'ax_focused_query': 'unavailable; no contradictory focus conclusion',
    'builtin_display_cg_bounds': {'x': -1728, 'y': 0, 'width': 1728, 'height': 1117},
    'external_display_cg_bounds': {'x': 0, 'y': 0, 'width': 2560, 'height': 1440},
    'main_window_cg_bounds': {'x': -1453, 'y': 211, 'width': 1180, 'height': 760},
    'small_target_window_cg_bounds': {'x': -1447, 'y': 217, 'width': 66, 'height': 20},
    'main_window_fully_inside_builtin_display': True, 'raw_native_json_saved': False,
    'check_source': 'methods/native-readonly-check.swift', 'repeat_execution': False,
})
save(OUT / 'source-map.json', {
    'scope': 'invalid foreground diagnostics; no product performance acceptance', 'items': mapping,
    'derived_files': ['run-metrics.json', 'placement-transcript-derived.json'],
    'omitted': ['empty collector/monitor logs and stop marker', 'application binaries', 'user payloads', 'raw preferences', 'WebView profiles', 'other app metadata'],
    'method_reference': '../20261012-前台采样保护工具准备/methods/run_guard.py',
})
readme = '''# 内屏前台保护中断（不计性能验收）

复用 deb740 的唯一隔离测试 PID31501，浅色 / 中档静态 / mesh / 100%。内屏位置已确认；本轮在 90 秒预热内中止，资源采集未启动，资源样本为零。窗口仍可见，只失去前台；原因未确定，不认定软件焦点缺陷。用户此前说明的人为前台切换不作软件问题。

## 实际轮次

- guarded-mid-inner：172 条前台事件，先 ready，最后有效样本 elapsed 18.644822 秒，首次无效 elapsed 18.750805 秒、UTC 02:10:27.817139。frontmost=false，窗口可见；monitor exit 3，resource_collector_exit=null，collection_completed=false。90 秒预热未完成，未产生 summary.json 或资源样本。
- guarded-mid-finder / prearmed：均在 10 秒内未前台就绪，monitor exit 2、not_ready_within_ten_seconds；无资源采样。保存各自原始日志与前后选定设置，没有这两轮的完整首末 UI，不计验收。
- 三轮首末 preferences SHA 均为 aed374985a30e077a7093a4ea5983b5f7b61bce40cee7ccbf5cc957f19e9e8aa。真实画质为 mid；guard-context 中旧 manifest 的 high 原样保留。launch.json 表示已有进程的场景预热起点，不是应用启动或启动性能样本。

## UI、位置与来源边界

内屏位置只读查询 UTC 02:09:33 的结果来自工具对话记录，未保存原始 JSON；placement-transcript-derived.json 明示这一来源。主窗口 x=-1453、y=211、1180×760，完整落在内屏 x=-1728、y=0、1728×1117 内。NSWorkspace 当时确认目标前台；AX 查询返回 -25204 不可用，不解释为相反的焦点结论。只读 Swift 方法保留，没有为补档重新查询或伪造原始结果。

内屏轮 start.jpg / end.jpg 已目视核对为同一浅色概览准备态，OpenCodex 未接入。开始 UI 时间 UTC 02:09:58.193，结束 UTC 02:11:10.090；截图不能证明两者之间持续前台。start.ax.txt 仅为“树未变化 / 概览焦点”的增量，不是完整 AX；end.ax.txt 是完整结束树。Finder 准备轮保留设置页与概览起点 UI，没有补造结束 UI。

代理打开的 Finder test-install 标签已关闭，原有“下载 / 存储 / work”三个标签保留。没有启动新测试实例、退出日常应用、修改系统设置或迁移真实数据；这一轮不再请求或占用前台重试。

来源文件按字节复制并列入 source-map / manifest，run-metrics 从目标前台 JSONL 独立计算。保护工具方法见[前台采样保护工具准备](../20261012-前台采样保护工具准备/README.md)。空日志 / stop marker 留在原临时目录，不提交应用二进制、配置载荷、WebView profile 或其他应用元数据。

## 门禁与下一步

该轮无效，不提供 CPU / RSS 或根因结论，也不覆盖[既有十分钟性能比较未通过](../20261012-deb740-十分钟性能对照/README.md)。前次比较亦缺连续前台与受控配对，只保留其原限制；调查线索尚未形成因果。暂无据此修改产品源码的依据。

后续先取得中档静态 / 高档 CSS / 必要的高档 mesh 持续前台诊断，确定原因后再改源码。正式门禁继续要求受控十分钟配对 CPU 增量 ≤0.5 百分点、RSS 增量 ≤10 MiB，冷 / 热各 ≥20 次、可交互 p95 增量 ≤100ms 且 ≤5%，无 >50ms 主线程任务，以及真实 stable 24h / beta 6h / 面板 24h 周期和睡眠 / 重启 / 离线 / 慢网 / 并发 / 通道隔离。双平台原生、安装、真实更新、事件与签名门禁继续开放；Windows runner 的 elevated token 不计普通 / 管理员双权限通过，macOS ad hoc 副本不计正式签名。

TASK-182 / IMP-20 保持 in_progress；review:scheduler-performance-0110 failed，八项 required checks 未整体闭合，acceptance_results 为空。不晋升核心事实，不替换日常安装，不执行发布。
'''
(OUT / 'README.md').write_text(readme)
items = [{'path': p.relative_to(OUT).as_posix(), 'size_bytes': p.stat().st_size, 'sha256': sha(p)} for p in sorted(OUT.rglob('*')) if p.is_file()]
save(OUT / 'manifest.json', {'scope': 'invalid diagnostic only; zero resource samples', 'items': items})
assert all(sha(OUT / i['path']) == i['sha256'] and (OUT / i['path']).stat().st_size == i['size_bytes'] for i in items)
save(OUT / 'archive-verification.json', {
    'captured_at': datetime.now(timezone.utc).isoformat(), 'manifest_sha256': sha(OUT / 'manifest.json'),
    'verified_items': len(items), 'byte_identical_source_copies': len(mapping),
    'total_evidence_bytes': sum(i['size_bytes'] for i in items), 'result': 'passed',
    'application_acceptance': False, 'resource_sample_count': 0,
})
appendix = '''
### 2026-10-12 内屏前台保护实际中断，不计性能验收

用户放好唯一隔离测试窗口后，只读位置查询确认 PID31501 主窗口完整位于 MacBook 内屏（x=-1453、y=211、1180×760），NSWorkspace 前台为目标；AX 查询 -25204 不可用，不作相反焦点结论。原 JSON 未保存，归档位置说明明确来自工具对话记录。继续复用 deb740 副本，浅色 / 中档静态 / mesh / 100%；旧清单 high 与真实 mid 的区别保留。

实际 run_guard 内屏轮先就绪，172 条事件中最后有效样本 elapsed 18.644822 秒，首次无效 18.750805 秒为非前台但仍可见；monitor exit 3、resource_collector_exit=null。90 秒预热未完成，资源采集未启动、样本零，无 summary.json；不计 CPU / RSS 或启动验收。此前 finder / prearmed 两轮均 10 秒未就绪、exit 2、资源零。三轮首末偏好摘要一致。窗口失去前台的原因未确定，不认定软件缺陷；用户此前人工切换按用户说明保留，不作软件问题。

内屏首末 JPEG 已目视核对同一概览准备态；开始 AX 仅增量，结束为完整树，截图不证明期间持续前台。Finder 准备轮无完整末 UI，不补造证据。已关闭代理打开的 test-install 标签，保留原“下载 / 存储 / work”三个标签；这一轮不再占用前台重试。[内屏中断证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-内屏前台保护中断/README.md)保留目标日志、原设置摘录、UI 与明确的衍生来源，按 manifest 核验；无产品源码变化。

既有性能比较未通过及其受控 / 连续前台限制保持原义，不能用本轮零样本解除。后续先取得可持续前台的场景诊断、确定原因再修源码，保留光层顶部 / 侧边覆盖。原性能预算、冷 / 热配对与主线程、真实周期及双平台原生 / 安装 / 更新 / 事件 / 签名门禁不变；review:scheduler-performance-0110 failed，八项 required checks 未整体闭合，TASK / IMP in_progress，acceptance_results 为空；无核心事实晋升、真实迁移、日常替换或发布。
'''
with (ROOT / IMP).open('a') as stream:
    stream.write(appendix)
sys.path.insert(0, '/Users/ezio/.codex/skills/project-governance/scripts/lib')
from runtime.kernel import build_receipt, validate_object
validate_object(task)
changed = sorted([RCP.as_posix(), IMP.as_posix()] + [p.relative_to(ROOT).as_posix() for p in OUT.rglob('*') if p.is_file()])
receipt = build_receipt(task,
    receipt_id=RCP.stem, result='failed', changed_paths=changed,
    checks=[
        {'name': 'local:inner-foreground-archive-integrity', 'status': 'passed', 'evidence': (REL / 'archive-verification.json').as_posix()},
        {'name': 'review:scheduler-performance-0110', 'status': 'failed', 'evidence': (REL / 'README.md').as_posix()},
    ], acceptance_results=[],
    fact_impacts=[{'target': 'imp-20', 'path': IMP.as_posix(), 'after_digest': sha(ROOT / IMP), 'summary': '记录内屏位置与三轮无效前台保护尝试，资源采样零；性能及其他开放门禁不解除。'}],
    completed_at=datetime.now(timezone.utc).isoformat(),
    summary='内屏位置确认正确；保护轮在约18.75秒预热阶段失去前台但仍可见，资源采集未启动。另两轮未就绪亦无资源样本。来源归档核验通过，原因未确定、不认定产品缺陷，性能failed与空验收保持；只回写协作包、IMP和新RCP。')
validate_object(receipt)
save(ROOT / RCP, receipt)
assert sha(TASK) == task_sha
save(UI / 'inner-write-manifest.json', {'changed_paths': changed, 'task_sha256': task_sha, 'before_status_sha256': hashlib.sha256(before_status).hexdigest(), 'imp_after_sha256': sha(ROOT / IMP), 'receipt_sha256': sha(ROOT / RCP), 'archive_manifest_sha256': sha(OUT / 'manifest.json')})
print(json.dumps({'archive': str(OUT), 'manifest_items': len(items), 'source_copies': len(mapping), 'changed_paths': len(changed), 'total_bytes': sum(i['size_bytes'] for i in items), 'run_metrics': [{'run': m['run'], 'events': m['event_count'], 'samples': m['sample_count'], 'valid': m['valid_sample_count'], 'invalid': m['invalid_sample_count'], 'resource_samples': 0} for m in metrics], 'receipt_valid': True}, ensure_ascii=False))
