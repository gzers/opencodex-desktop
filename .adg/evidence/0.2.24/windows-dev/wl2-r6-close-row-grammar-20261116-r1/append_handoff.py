from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
p=ROOT.parents[4]/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
s=json.loads((ROOT/'final-status.json').read_text())
source=json.loads((ROOT/'final-source.json').read_text())
marker='## 2026-11-17 r6 关闭清单严格角色与节点序号'
assert marker not in p.read_text(encoding='utf-8-sig')
text=f'''

## 2026-11-17 r6 关闭清单严格角色与节点序号

源码{s['sourceHead']}/tree{s['sourceTree']}已推送，承接4e2640640/6b3770202；TEAM16/IMP142/r6及既定共同源不变。证据 .adg/evidence/0.2.24/windows-dev/wl2-r6-close-row-grammar-20261116-r1/ 于11月16日开始跨17日完成，目录名保留。6精确Windows允许文件，canonical close行只收实际producer语法，拒未知role/root、非规范有界ordinal、pendingwork、错coord ancestor/parent/bound及directory/history标记。原件/history-source逐项原计划，initialwork实际原工作代次、stoppedanchor停止快照准确非退休对象，不凭调用方路径/前缀认定。

debug v3数据{s['runs']['library01']['passed']}项全过，含27个额外坏行重算hash/count且完整原必需行仍在、逐字progress原子拒绝；knownproducer正例及tree ID/generation绑定通过。release受影响actual native handoff{s['runs']['library02']['passed']}项全过，真实关闭batch角色/ordinal与native对象匹配。独立full native ExpectedHandleInventory、CloseHandle/lease transfer/树detach、actual Stopguard/content/原预算等13处实现逐字保持；canonical旧work/lease行仍只是locator DATA，完整性/许可必须来自独立actual native观测，不反序列化授权。

本轮无非预期首错，继承早先失败/Unknown不改。实际ordinary fixedofficial Stop完成history/typedClosed/同次Status5/dispatcher/后继、config/auth/Grok/动态目录副作用、初始化/ownerreceipt/原缺席Codex安全退根、最大完整batch容量与普通MSI/CAP-trust/历史清理仍未完。Start拒绝，不把数据或组件raw0当生产/整版验收。

产品custom-protocol MSVCdebug bins raw0/{s['runs']['product01']['elapsedMs']}ms，{len(source['productInputs'])}产品输入/{len(source['releaseRust'])}最终releaseRust核工作/Git（明确CRLF等价），预存schema未手改/暂存，三制品SHA归product01-artifacts。{s['cleanup']['scenes']}scene私有ZIP完整hash/membership核验，所有plan/native/canonical scene保留，本轮无删除/历史拒绝重试，ZIP非NTFS身份安全克隆；原canonical/快照/history另收存。Start/allAccepted/typedClosed/productionExecuted=false，不W2/双SID/发布。
'''
with p.open('a',encoding='utf-8',newline='') as f:f.write(text)
print(json.dumps(dict(appended=True,source=s['sourceHead'])))
