from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
p=ROOT.parents[4]/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
status=json.loads((ROOT/'final-status.json').read_text())
marker='## 2026-11-16 r6 独立完整原生关闭句柄清单'
assert marker not in p.read_text(encoding='utf-8-sig')
text=f'''

## 2026-11-16 r6 独立完整原生关闭句柄清单

源码{status['sourceHead']}/tree{status['sourceTree']}已推送，承接502858eb5/f3a72381a。TEAM16/IMP142/r6及既定main/治理共同源不变；证据 .adg/evidence/0.2.24/windows-dev/wl2-r6-exact-close-inventory-20261116-r1/，完整继承状态/首错/raw/source/EXE归report/final-status。7允许文件含原Windows runtime paths只读祖先观察接口；ExpectedHandleInventory private/nonserde/nonClone在detach前从原对象独立逐项观测tree original/history-source/stopped-anchor/initial-work/全部old-work和全部old/final lease四coord/file/claim-parent/ancestor/strict-parent/bound句柄。准确顺序/key/role/nativeID/kind/history/count在关闭intent前/actual close前后全等核对，不从prefix/JSON或count生成native完成。

原NativeHandleBatch.close_all/lease transfer/树detach函数体逐字保留，14处源/函数核对及公开Start拒绝保持。新原语反例missing/extra/role/actualID/changeddeadline在关闭前拒绝，无替换/关闭/Drop成功；原pin祖先handle全部原顺序/对象观察，无clone/reopen/权限改变。debug受影响{status['runs']['library01']['passed']}项全过，release关闭/完成history及原故障回归{status['runs']['library02']['passed']}项全过，原10秒操作/5秒历史窗口、raw退出/unknown拒绝不变。本轮无非预期首错，继承早先编译/测试失败保持。

实际ordinary fixedofficial Source/Owner/资格/恢复/关闭仍仅编译未执行；process/supervisor完成history、typedClosed/同次Status5/dispatcher/后继、官方config/auth/Grok/动态目录副作用/首次初始化/ownerreceipt/原缺席Codex安全退根保持未完。canonical DATA额外tree/lease key严格未知角色/ordinal语法仍待；最大1MiB/512node完整batch容量需进一步核验，不能靠排除原件过界。独立native full inventory不是放行公共Start或整版验收依据。

产品custom-protocol MSVCdebug bins raw0/{status['runs']['product01']['elapsedMs']}ms，184产品输入/138最终releaseRust核工作/Git（明确CRLF等价），预存schema未手改/暂存；制品SHA归product01-artifacts。{status['cleanup']['scenes']}scene完整私有ZIP/hash/membership核验，12完成合成inventory原语scene核fresh准确路径/字节后清理，{status['cleanup']['failedAndPendingRetained']}失败/pending及{status['cleanup']['crossVolumePendingRootsRetained']}个D根保留；原canonical/快照/history另收存。ZIP非NTFS身份安全克隆，历史拒绝删除不重试，普通MSI/CAP-trust/旧Profile/Public/未知broker清理仍未完。Start/allAccepted/typedClosed/productionExecuted=false，不W2/双SID/发布。
'''
with p.open('a',encoding='utf-8',newline='') as f:f.write(text)
print(json.dumps(dict(appended=True,source=status['sourceHead'])))
