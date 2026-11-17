from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
p=ROOT.parents[4]/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
s=json.loads((ROOT/'final-status.json').read_text())
source=json.loads((ROOT/'final-source.json').read_text())
marker='## 2026-11-17 r6 同次Status与原动作完成历史读取'
assert marker not in p.read_text(encoding='utf-8-sig')
text=f'''

## 2026-11-17 r6 同次Status与原动作完成历史读取

源码{s['sourceHead']}/tree{s['sourceTree']}已推送，承接a653a51ac/ecb3b27c8；新fetch main fac8573e5/治理578230717及TEAM16/IMP142/r6不变。6精确Windows允许文件，证据 .adg/evidence/0.2.24/windows-dev/wl2-r6-completed-read-windows-20261117-r1/。HistoryReadWindow private/nonserde/nonClone，只作只读deadline locator：Status准确原when/end<=5s，动作直接借原ActionWindow.deadline并前后remaining，原parent/Restart30/Stop子10保持。CaptureSlot/Journal fresh source/audit pin/retained exit-supervisor/canonical/archive全读同一request，不改旧Stop mutation截止或创建now+budget/native许可。

首library01 raw0/{s['runs']['library01']['passed']}过；静态发现原ActionWindow截止错误映射Other，纠正为TimedOut后补实际原窗口到期反例，末library02 raw0/{s['runs']['library02']['passed']}过，real elapsed read和再次构造都TimedOut、原deadline不变。首轮source/EXE/75scene原样保留不归末轮。library03 release raw0/{s['runs']['library03']['passed']}过，新增native97真实原件关闭/旧截止后给fresh Start30 read仍InvalidData拒绝补签finalStop，精确phase/current/原immutable前缀不变；旧正常/Unknown/partial/late/Status5回归保持。21处原factory/变更/关闭/RoutePin源与函数逐字不变。

批次在写源/测试前核对到H route config/ancestor pin须同链显式native close intent/result，不能单纯take/drop声称全释放；scope-adjustment原样保留，本轮routes仍保管，未解除后继限制。记录器重复create-only旧scope文件FileExists拒绝，原快照留存，最终精确新文件名source-scope-verified记录，不覆盖证据。无非预期产品/测试失败；读取窗口不是actual ordinary fixedofficial final history正例。

下一步准确owned route-pin关闭/完整native状态与final seal同链，再接typed/public Status/dispatcher/后继；实际原包副作用/初始化/ownerreceipt/原缺席Codex安全退根/最大完整batch容量、普通MSI/CAP-trust/全部历史清理继续未完。产品custom-protocol MSVCdebug bins raw0/{s['runs']['product01']['elapsedMs']}ms，{len(source['productInputs'])}产品输入/{len(source['releaseRust'])}最终releaseRust核工作/Git（明确CRLF等价），预存schema未手改/暂存，三制品SHA归product01-artifacts。

{s['cleanup']['scenes']}scene私有ZIP完整hash/membership核验，全部plan/native/canonical现场保留；原canonical/快照/history另收存，本轮无删除/历史拒绝重试，ZIP非NTFS元数据克隆。不改官方/日常配置/PATH/账号/策略，不业务/模型请求；Start/allAccepted/typedClosed/productionExecuted=false，不旧raw5/空GUI/W2/双SID/发布。
'''
with p.open('a',encoding='utf-8',newline='') as f:f.write(text)
print(json.dumps(dict(appended=True,source=s['sourceHead'])))
