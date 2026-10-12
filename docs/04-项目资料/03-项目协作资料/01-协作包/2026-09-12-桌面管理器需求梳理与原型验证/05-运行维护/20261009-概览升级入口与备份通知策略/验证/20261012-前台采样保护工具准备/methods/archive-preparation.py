from pathlib import Path
import datetime,hashlib,json,subprocess,sys
ROOT=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop')
HERE=Path('/private/tmp/ocx-0110-performance-guard-20261012')
BASE=Path('/private/tmp/ocx-0110-performance-20261012')
REL=Path('docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-前台采样保护工具准备')
OUT=ROOT/REL
OUT.mkdir()
(OUT/'methods').mkdir()
(OUT/'results').mkdir()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,x):
 with p.open('x') as s:json.dump(x,s,indent=2,ensure_ascii=False);s.write('\n')
def command(argv):
 r=subprocess.run(argv,capture_output=True,text=True,timeout=50)
 return dict(argv=argv,exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr)
compile_result=command(['swiftc',str(HERE/'foreground-stream.swift'),'-o',str(HERE/'foreground-stream-verified')])
python_compile=command([sys.executable,'-m','py_compile',str(HERE/'run_guard.py'),str(HERE/'selfcheck-v2.py'),str(HERE/'stream-smoke.py')])
assert compile_result['exit_code']==python_compile['exit_code']==0
save(HERE/'compile-final.json',dict(captured_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),swift=compile_result,python=python_compile,swift_source_sha256=sha(HERE/'foreground-stream.swift'),swift_binary_sha256=sha(HERE/'foreground-stream-verified'),application_test=False))
sys.path.insert(0,str(BASE));import measure
config=measure.CONFIG['candidate0110']
apps=measure.all_apps()
tests=[a for a in apps if a['exe']!=measure.DAILY]
assert len(tests)==1 and tests[0]['pid']==31501 and tests[0]['exe']==config['exe']
daily=[a for a in apps if a['exe']==measure.DAILY]
assert len(daily)==1 and daily[0]['pid']==75764 and daily[0]['started']=='Fri Oct 9 22:00:24 2026'
assert sha(Path(config['exe']))==config['test_exe_sha256']
prefs=Path(config['root'])/'manager-state/preferences.json'
current=json.loads(prefs.read_bytes())
old_measure=ROOT/REL.parent/'20261012-deb740-中档静态中断诊断/evidence/methods/measure.py'
assert sha(BASE/'measure.py')==sha(old_measure)
context=dict(captured_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),test_process=tests[0],daily_process=daily[0],source_sha=config['source_sha'],test_exe_sha256=config['test_exe_sha256'],actual_appearance=current['appearance'],actual_updates=current['updates'],preferences_sha256=sha(prefs),original_manifest_appearance=config['appearance'],original_measure_sha256=sha(BASE/'measure.py'),original_candidates_sha256=sha(BASE/'candidates.json'),code_head=subprocess.check_output(['git','rev-parse','HEAD'],cwd='/Users/ezio/.codex/worktrees/ocx-0110-main',text=True).strip(),native_access='no new CUA call this continuation; last unlock response still had locked tool result, per handoff',native_readiness='unverified',application_acceptance='not_evaluated',live_harness_integration='not_run',note='process and selected settings read-only; no app launch/activation/input/stop, no raw preference payload archived')
save(HERE/'context-preparation.json',context)
mapping=[]
for src,dest in [('foreground-stream.swift','methods/foreground-stream.swift'),('run_guard.py','methods/run_guard.py'),('selfcheck-v2.py','methods/selfcheck-v2.py'),('stream-smoke.py','methods/stream-smoke.py'),('archive-preparation.py','methods/archive-preparation.py'),('selfcheck-v2-final.json','results/selfcheck-v2-final.json'),('stream-smoke-final.json','results/stream-smoke-final.json'),('compile-final.json','results/compile-final.json'),('context-preparation.json','results/context-preparation.json'),('selfcheck-v2-first-failure.txt','results/selfcheck-v2-first-failure.txt')]:
 a=HERE/src;b=OUT/dest
 with b.open('xb') as s:s.write(a.read_bytes())
 assert a.read_bytes()==b.read_bytes()
 mapping.append(dict(source=str(a),archive=dest,size_bytes=b.stat().st_size,sha256=sha(b),copy='byte_identical'))
save(OUT/'source-map.json',dict(scope='external diagnostic methodology and preparation only',items=mapping,omitted=['compiled executables','__pycache__','user payloads','other app metadata','raw preferences','WebView profiles']))
readme='''# 前台采样保护工具准备（不计应用验收）

本记录只证明外部诊断工具的编译、合成测试和归档完整性。没有新的应用资源采样、首末 AX / 截图或完整前台测试，不能替代 0.1.10 性能、原生或安装验收。

## 方法与验证

- Swift 只读监视指定 PID 的前台状态和屏幕上 layer-zero 窗口，约每 0.1 秒输出；未就绪 10 秒、就绪后失焦 / 隐藏或间隔超过 1 秒即停止。只记录目标布尔值、PID 和时钟。
- Python 使用非阻塞原始管道读取消除 TextIO 缓冲风险；就绪后超过 1 秒无完整输出即终止，首次就绪另有 12 秒看门狗。只终止自己创建的采集与监视子进程，不启动、激活或退出应用。
- 采集前校验唯一隔离实例、EXE 摘要和真实持久化画质；预热后及结束再核验应用身份，首末偏好摘要须一致。原候选清单仍写 high，实际读取为 mid；原件保持原样。
- Swift / Python 编译通过，当前工具的 25 项合成自检与 4 项自有子进程 / 管道测试通过。首次 v2 自检的失败及测试修正说明保留在 results/selfcheck-v2-first-failure.txt。
- 这些测试没有连接真实应用或执行完整 run_guard.main；真实工具链集成、场景首末完整 AX / 截图、持续前台及相同基线配对均待验证。轮询不能证明采样之间从未发生短暂中断，也没有主线程或网络仪器。

## 使用边界

原生控制在用户此前回复解锁后仍返回锁屏；本轮没有追加 CUA 重试。准备快照只读确认唯一隔离 PID31501、deb740 测试 EXE 和 mid / mesh / light / 100%，日常 PID75764 与启动时间未变。此快照不证明原生界面当前可操作或当前场景。

待前台实际可操作并确认场景后，复用唯一隔离实例。实时工具位于 /private/tmp/ocx-0110-performance-guard-20261012；原资源采集依赖 /private/tmp/ocx-0110-performance-20261012/measure.py 与 candidates.json。开始后不得操作其他前台应用；失焦、锁屏、断流或身份漂移均使该轮无效，保留部分记录只作调查。完整首末 UI 核验仍须单独完成。

归档方法是诊断证据，不是产品实现。methods/run_guard.py 的上述临时路径是原环境出处；原 measure 方法在[中档中断诊断](../20261012-deb740-中档静态中断诊断/evidence/methods/measure.py)。离线自检可从 methods 运行 selfcheck-v2.py / stream-smoke.py，传入新的外部结果路径，避免改写归档。编译二进制和缓存未提交。

## 门禁与下一步

既有[十分钟性能对照](../20261012-deb740-十分钟性能对照/README.md)仍未通过：CPU 增量 +1.817629 百分点（预算 ≤0.5），平均 RSS 增量 +117.008965 MiB（预算 ≤10）。单次顺序观测未建立泄漏或根因；此前中档完整轮次亦无效，不用部分低 CPU 数据取消失败。

下一步补真实中档 mesh / 高档 CSS / 必要的高档 mesh 诊断，先确定原因再修源码，保持光层顶部与侧边覆盖。然后按原预算补受控十分钟对照、冷 / 热各至少 20 次、主线程 / 零请求及真实 stable 24h / beta 6h / 面板 24h 周期，并完成双平台原生 / 安装 / 更新 / 事件和签名门禁。

TASK / IMP 继续 in_progress，八项 required checks 未整体闭合，review:scheduler-performance-0110 failed，acceptance_results 为空，发布未授权 / 未验收；未迁移真实数据或替换日常安装。
'''
(OUT/'README.md').write_text(readme)
items=[]
for p in sorted(OUT.rglob('*')):
 if p.is_file():items.append(dict(path=p.relative_to(OUT).as_posix(),size_bytes=p.stat().st_size,sha256=sha(p)))
save(OUT/'manifest.json',dict(scope='methodology/preparation only, no application acceptance',items=items))
assert all(sha(OUT/i['path'])==i['sha256'] and (OUT/i['path']).stat().st_size==i['size_bytes'] for i in items)
save(OUT/'archive-verification.json',dict(captured_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),manifest_sha256=sha(OUT/'manifest.json'),verified_items=len(items),byte_identical_source_copies=len(mapping),total_evidence_bytes=sum(i['size_bytes'] for i in items),result='passed',application_acceptance=False))
print(json.dumps(dict(archive=str(OUT),manifest_items=len(items),source_copies=len(mapping),manifest_sha256=sha(OUT/'manifest.json'))))
