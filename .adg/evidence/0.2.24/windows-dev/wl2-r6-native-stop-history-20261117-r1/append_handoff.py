from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
p=ROOT.parents[4]/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
s=json.loads((ROOT/'final-status.json').read_text())
source=json.loads((ROOT/'final-source.json').read_text())
marker='## 2026-11-17 r6 原Stop完成历史与最终同链回执'
assert marker not in p.read_text(encoding='utf-8-sig')
text=f'''

## 2026-11-17 r6 原Stop完成历史与最终同链回执

源码{s['sourceHead']}/tree{s['sourceTree']}已推送，承接18a079fa4/f568a540c；新fetch main fac8573e5/治理578230717，TEAM16/IMP142/r6不变。14精确Windows允许文件，证据 .adg/evidence/0.2.24/windows-dev/wl2-r6-native-stop-history-20261117-r1/，完整继承矩阵/raw/源码/EXE/工具首错归report/final-status。内部实际Stop恢复/独立fullinventory/native关闭后消费原NormalExit/Absent，每个成功into_closed_history先装Store再下一失败点，partial不重试/抹掉。CompletedStopGuard借actual source/Owner/StopAccepted/routes、audit源码pin、原内容scope/进程代次及原action end，不从DATA/flag构造。

同一canonical新增StopClosureIntent→StopClosedVerified，准确last handle-close seq/tip/current/native receipt和content/history/inventory/count/Ready全绑定；pending挡其它事件，final封链。原关闭回执保持为前驱；最终CompletedStop工厂显式需要actual guard并在原截止前核精确新current，不重用旧state或重开预算。内部completed_stop_at同原Status when/end<=5s，fresh Source/audit pin/retained退出监督历史/canonical/native archive/retired对象只读，未接publicStatus/typed tags/dispatcher/后继；Stop restore返回内部结果，公开Start仍拒绝。

library01 raw0/{s['runs']['library01']['passed']}过，随后加actual guard工厂参数及native96反例；最终library02 raw0/{s['runs']['library02']['passed']}过，v3/capture/closure反例逐字progress不变、3787<=4096 current通过；library03 release raw0/{s['runs']['library03']['passed']}过，旧native完成/Unknown/partial/late与原10秒/5秒保持，新actual原件关闭回执在旧截止后不能单独补签finalStop，准确phase/current不变。旧17函数/文件逐字核对，未删断言/改窗口。本轮无非预期产品/测试失败，source verifier初默认GBK读UTF8失败原文留存，只改记录器encoding；首轮source/EXE保留不归末轮。

实际ordinary fixedofficial新guard/closure/finalseal正例及新closure-specific append/current失败窗口仍未执行，不把DATA或旧native负例当typedClosed/生产Stop10接受。路由pin退出/adapter所有权、publicStatus同窗/后继、官方config-auth-Grok/动态目录副作用、初始化/ownerreceipt/原缺席Codex安全退根、最大完整batch容量及普通MSI/CAP-trust/历史清理保持未完。产品custom-protocol MSVCdebug bins raw0/{s['runs']['product01']['elapsedMs']}ms，{len(source['productInputs'])}产品输入/{len(source['releaseRust'])}最终releaseRust核工作/Git（CRLF等价明确）；预存schema未手改/暂存，制品SHA归product01-artifacts。

{s['cleanup']['scenes']}scene完整私有ZIP/hash/membership核验，全部plan/native/canonical现场保留，本轮无删除/历史拒绝重试；原canonical/快照/history另收存。ZIP非NTFS身份安全克隆。Start/allAccepted/typedClosed/productionExecuted=false，不旧raw5/空GUI/W2/双SID/发布。
'''
with p.open('a',encoding='utf-8',newline='') as f:f.write(text)
print(json.dumps(dict(appended=True,source=s['sourceHead'])))
