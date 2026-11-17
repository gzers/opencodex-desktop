from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
p=ROOT.parents[4]/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
s=json.loads((ROOT/'final-status.json').read_text());source=json.loads((ROOT/'final-source.json').read_text())
marker='## 2026-11-17 r6 原路由pin准确所有权与显式关闭原语'
assert marker not in p.read_text(encoding='utf-8-sig')
text=f'''

## 2026-11-17 r6 原路由pin准确所有权与显式关闭原语

源码{s['sourceHead']}/tree{s['sourceTree']}已推送，承接fcc7a688d/e3daa9956；TEAM16/IMP142/r6及共同源不变。6精确Windows允许文件，证据 .adg/evidence/0.2.24/windows-dev/wl2-r6-route-pin-close-20261117-r1/。RetainedRouteClose先保管原RoutePin再做fallibleIO，独立expected原config/security及两完整祖先链，移全部actual Files至原close batch，主流bytes/hash/8/ADS/Token/原end fresh检查。成功仅全部ConfirmedClosed后签私有ownclose completion，后继独立H/config能写且旧facts不采用新字节；不是actual Stop/typedClosed/同链许可证，生产RoutePin未改变/释放。

首library01 raw101/47过1败：fault81原primary先关后READ_CONTROL-only观察句柄不能维持denywrite，原raw32断言遇Ok。保留原断言/准确source/EXE/60scene，改expected/actual关闭顺序primary最后；首失败probe前trace未输出，明确不补造/不重跑旧前提。library02 raw0/48，末加ADS fresh veto及attempt-before-assert trace后library03 raw0/49过。partial/protectedUnknown/allclosed-before-privatecompletion/wrongdeadline/data/inventory都保留精确Held/ConfirmedClosed/Unknown，未知不重试/重建/清保护。ADS移交后确实创建，close前全Held时InvalidData拒绝，原ADS保留且字节独立导出/hash；ZIP不当ADS或NTFS元数据克隆。source recorder首比对new选错impl已留first-error，准确native close原body/20源片段保持。

候选原语未接实际Owner/source/action/normal/supervisor及same canonical route-close intent/result/final Stop parent lineage；这是下一步，不能把drop/take或candidate raw0当完整release/后继许可。ordinary原包闭环/公共Status/typed adapter/副作用/初始化/ownerreceipt/原缺席Codex安全退根/最大完整batch容量、普通MSI/CAP-trust/历史清理仍未完，Start拒绝。

产品MSVCdebug custom-protocol bins raw0/{s['runs']['product01']['elapsedMs']}ms，{len(source['productInputs'])}产品输入/{len(source['testRust'])}最终testRust核工作/Git（明确CRLF等价），预存schema未手改/暂存，制品SHA归product01-artifacts。{s['cleanup']['scenes']}scene私有ZIP/hash/membership核验后全部保留，含首失败/protectedUnknown/nativepending/ADS，原canonical/快照/history另收存；本轮无删除/历史拒绝重试。不改官方/日常配置/PATH/账户/策略，不业务/模型请求，不旧raw5/空GUI/W2/双SID/发布，Start/allAccepted/typedClosed/productionExecuted=false。
'''
with p.open('a',encoding='utf-8',newline='') as f:f.write(text)
print(json.dumps(dict(appended=True,source=s['sourceHead'])))
