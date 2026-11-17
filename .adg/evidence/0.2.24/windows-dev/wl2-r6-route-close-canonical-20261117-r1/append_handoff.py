from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent;p=ROOT.parents[4]/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
s=json.loads((ROOT/'final-status.json').read_text());source=json.loads((ROOT/'final-source.json').read_text())
marker='## 2026-11-17 r6 原路由关闭进入最终canonical链';assert marker not in p.read_text(encoding='utf-8-sig')
text=f'''

## 2026-11-17 r6 原路由关闭进入最终canonical链

源码{s['sourceHead']}/tree{s['sourceTree']}已推送，承接b8e06be26/be0ff1fff；fetch main fac8573e5/治理578230717及TEAM16/IMP142/r6不变。15精确Windows允许文件，证据 .adg/evidence/0.2.24/windows-dev/wl2-r6-route-close-canonical-20261117-r1/。Store在actual退出/监督/qualified整树恢复/native关闭后，将准确原RoutePin移retained候选先装再prepare；actual CompletedStopGuard借候选与原source/Owner/action/内容scope/audit/退出监督，primary-last原生close不改。原native primitive/claim/整树恢复/关闭等17处实现逐字保持。

closure intent绑定完整routeCloseDigest；同chain RouteHandlesCloseIntent/current后实际close/readback再RouteHandlesClosedVerified，最后StopClosedVerified及actualguard final seal，原native原件关闭seq/tip/current/state保持准确前驱。DATA缺/跳/替换/额外或漏祖先/错对象roleflagcount拒绝，actual candidate产生的完整data通过严格shape；future同窗或原action只读核native routehistory和canonical digest一致，不采后继live config。完整current3980<=4096，仅余116字节不增上限/不删事实。首data56过后补历史读取join，末data{s['runs']['library02']['passed']}过，release{s['runs']['library03']['passed']}过均raw0；本轮无非预期产品/测试首错，初转换脚本marker不合原格式在写源前拒绝，修准确多行marker，旧失败证据保持。

actual新route/final factory和完整ordinary fixedofficial Stop组合正例及route专属phase/current故障窗口仍未执行；synthetic own-handle/data不授typedClosed或Stop10预算接受。公共adapter/Status/dispatcher/后继、官方config/auth/Grok/动态目录副作用/初始化/ownerreceipt/原缺席Codex安全退根/最大完整batch容量、普通MSI/CAP-trust/历史清理保持未完，Start拒绝。

产品MSVCdebug custom-protocol bins raw0/{s['runs']['product01']['elapsedMs']}ms，{len(source['productInputs'])}产品输入/{len(source['releaseRust'])}最终releaseRust核工作/Git（明确CRLF等价），预存schema未手改/暂存，三制品SHA归product01-artifacts。{s['cleanup']['scenes']}scene私有ZIP/hash/membership核验后仅3完成纯路由原语scene核fresh准确路径/hash/membership清理，{s['cleanup']['failedAndPendingRetained']}失败/pending及protectedUnknown/ADS原现场保留。原canonical/快照/history和ADS字节另导出，原流保留；ZIP非NTFS/ADS克隆，历史拒绝删除不重试。不改官方/日常配置/PATH/账户/策略，不业务/模型请求，不旧raw5/空GUI/W2/双SID/发布；Start/allAccepted/typedClosed/productionExecuted=false。
'''
with p.open('a',encoding='utf-8',newline='') as f:f.write(text)
print(json.dumps(dict(appended=True,source=s['sourceHead'])))
