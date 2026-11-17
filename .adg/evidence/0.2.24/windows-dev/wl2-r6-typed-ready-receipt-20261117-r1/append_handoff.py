from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent;p=ROOT.parents[4]/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
s=json.loads((ROOT/'final-status.json').read_text());source=json.loads((ROOT/'final-source.json').read_text())
marker='## 2026-11-17 r6 原typed Ready观察与保留身份读取';assert marker not in p.read_text(encoding='utf-8-sig')
text=f'''

## 2026-11-17 r6 原typed Ready观察与保留身份读取

源码{s['sourceHead']}/tree{s['sourceTree']}已推送，承接df0c8c24f/165f7fb88；TEAM16/IMP142/r6及共同源不变。3精确Windows允许文件，证据 .adg/evidence/0.2.24/windows-dev/wl2-r6-typed-ready-receipt-20261117-r1/。Owner.observe在原HMAC/health/current/live和phase校验后保留同一个不可Clone ServiceObservation；take_original_ready_for_stop需原Stop/Restart动作、stop_started且未stopped，重核原typed current/nativeReady，失败准确对象放回，不用PID/port/digest重建typed证明。公共分派仍v2、Start仍拒绝，actual观察/take/read factories只编译未执行。

NativeAction新增只读原Owner UUID/scope/token及准确Origin id/kind/outer/stop/end/admission比较，不改旧Mutation check/deadline，不发lease/写许可。原已接纳Origin真实到期后，元数据仍可比，原check仍Timeout，now+30替换截止拒绝，五身份/窗口替换拒绝；无假NativeAction/Owner/ServiceObservation构造。MSVC库raw0/{s['runs']['library01']['passed']}过0败，原Start/Stop/Status/recovery与shared typed传输/预算等12源片段核对不变（构造器仅初始化新增typed_ready字段）；无本轮非预期失败，继承首错保留。

产品MSVCdebug custom-protocol bins raw0/{s['runs']['product01']['elapsedMs']}ms，{len(source['productInputs'])}产品输入/{len(source['testRust'])}最终Rust核工作/Git（明确CRLF等价），预存schema未手改/暂存，制品SHA归product01-artifacts。本轮无新native目录/账号/Profile/PATH/策略残留，原失败场景不动。marenben7当前Disc已请求恢复普通交互桌面，控制器/Administrator不代验收；未启动官方服务。

后续完整liveReady只读原件/custody/anchors/current nonartifact/claim断言、v3 typed preStop/Closed/Status及后继，actual ordinary原包副作用/初始化/ownerreceipt/原缺席Codex安全退根/最大batch容量/普通MSI/CAP-trust/历史清理保持未完。Start/allAccepted/typedClosed/productionExecuted=false，不旧raw5/空GUI/W2/双SID/发布。
'''
with p.open('a',encoding='utf-8',newline='') as f:f.write(text)
print(json.dumps(dict(appended=True,source=s['sourceHead'])))
