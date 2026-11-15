# W-L2 单文件原对象句柄同卷移动原语

2026-11-15 Asia/Shanghai，TEAM13/IMP139/r3/r2。承接cf03db580及b8264f733证据，继续Windows允许native/test写集。allAccepted=false、productionExecuted=false、actualGUI=false；这是真实Win32单文件机制，不是完整ClientOriginals实现或生产验收。

## 原生机制与限定

file_move.rs新RetainedFileMove持有原DELETE-capable只读文件句柄（0x80030000，明确没有ACCESS_SYSTEM_SECURITY/SACL权限声明）、原父目录pin、原卷/fileID及有界2MiB内容。打开前后核ID/bytes，拒绝reparse/多硬链；FindFirst/NextStream只接受准确::$DATA，无额外stream，未知/错误不当完整。流和内容取原句柄/受固定父链的当前准确路径，不从serde/PID/日志重建文件权威。

move_into只接受一枚有效leaf及currentSID私有目标目录，核原父/目标原生目录身份不同且同卷；SetFileInformationByHandle(FileRenameInfo)操作原file handle，ReplaceIfExists=false，无跨卷复制/删除兜底，无目标删除/替换。native成功后立刻保留当前路径/目标pin，再做可能失败的字节/ID/stream/deadline后验。错误和超时不能丢掉已移动对象的位置；struct Drop不会自动恢复或删原件。restore只回原先固定父目录的准确原路径，仍用同一原句柄，不覆盖外部改动；先前rename成功但后验超时的状态按原location重新读回，不重复rename或补签动作成功。

普通内容身份观察的额外reader只请求与已有DELETE句柄相容的delete sharing；没有放宽原保留writer/delete权限给外部。它只用于读回同对象，不授予新Owner/Closed。新打开原件最初的native denywrite/delete共享仍阻止外部写/改名。公有API是底层机制，不是OriginalSession；生产capture调用点0，完整SD和刷盘意图仍先决，r3安全拒绝门未动。

只处理单文件，当前明确不处理目录/整根、完整安全描述符、官方工作副本、两锁释放、原件manifest/current-ref CAS或持久恢复。目标D异卷时只能存引用，原对象须留原卷，这个机制不把跨卷拒绝称作D/Profile异卷完整事务通过。原SD1314未重跑、无新权限/ACL/策略修改。

## 实际定向结果

hypotheses01先记主机制。native01两项：原卷/fileID/字节在同一原句柄移入私有目录并还回相同，目标占用和restore原路径被合成外部编辑占用均返回原raw183，两个原始内容保留；只在测试显式移除它自己创建的冲突哨兵后才继续同原对象恢复。ADS、多链接、非法name、过期窗口移动前拒绝。

hypotheses02新增已证明C/D不同卷的独立反例，native02只跑该1项：源247287385、目标3089487301，move前拒绝，源字节/ID/path仍在、目标leaf未创建，无copyfallback。随后代码复核补同原native父目录不可伪装为custody迁移，native03整批4项全部通过，复跑因runtime原guard改变而受影响。same-parent复核说明是在native03之后落盘，未倒签成先置hypothesis；原hypotheses01/02不覆盖。

本轮共4不同用例/7成功执行，没有失败case/编译，firstError=null。输入只为合成无凭据文件，不打开日常.codex/.grok；没有完整SD或OriginalSession/Closed正例。原native01 EXE和最终native03 EXE各自实际SHA保存；native02 EXE在native03前被覆盖且未独立采其hash，此缺口单列，不把finalhash冒前轮。

代码两笔已push codex/024-windows-native：d82e8493c9ce97ffa4c2f8097ddec2212d2a27b0（原file handle移动及native export）；8958a59b80c95501ba2a0920cad3f4f228975573（4项定向/跨卷/同父拒绝及独立入口），净4许可文件。未改共享trait、Cargo/API feature依赖、release参数、安装器/Mac或生产捕获门。final-source-manifest含最终tree/Gitblob/原字节，CRLF等价分列；old untracked schema不处理。

native03与product01每组38Rust逐字对最终源；native01/02分别为旧file_move/测试输入，后继guard和test变化完整列出，不冒最终执行源。product01实际head8958a59b8且dirtydiff空。

最终MSVC debug bins exit0/69640ms，93warnings保留。不是签名release/冻结候选或原预算生产通过。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop.exe | 75458560 | 0f84dc04d7af715abe9eef3415e26e1ec72beb056e569bc124f28352a2330ce6 |
| ocxd.exe | 5398528 | db4a9b85d163ac659a51cf8a04cc837e9e16d12ebc5820a31edc4a813e5badf1 |
| ocx.exe | 306176 | 24a8a5941ceaeaab283c7816e9341e4ae7c8311b61217fbf943917cb4f37136c |

API参考仅列官方URL，无本轮下载正文/摘录hash声明；可记录的直接依据是固定windows-sys0.61.2结构签名及上述真实native结果。

## 完整状态表

| 项目 | 实现 | 定向结果 | 限定 |
| --- | --- | --- | --- |
| single-file-original-handle | implemented | native03 positive roundtrip passed | Native volume/fileID and bounded bytes verified through same original handle; no copy/rebuild. |
| destination-occupied | atomic no-replace | native183 preserved; both sides unchanged | Error retains original handle/location; no target deletion or overwrite. |
| restore-external-edit | same original object; no replace | native183 preserved; external edit survives | Explicit test removes only its own synthetic conflict then restores original; product has no automatic deletion. |
| ADS-hardlinks | refuse incomplete object | native stream+multilink negatives passed | Only exact unnamed DATA stream; stream-query errors refuse, never Unknown-to-absent. |
| invalid-name-and-deadline | bounded original inherited window | native negatives passed | No extra budget or implicit recovery; native success updates location before post-check failure. |
| cross-volume | native target parent volume checked | C247287385 vs D3089487301 refused | No source deletion or copy fallback, target leaf absent; two new exact D test roots removed by guards. |
| same-parent-no-op | native parent identity checked | native no-op/same-parent counterexample passed | Cannot call same original directory a custody transition, even a new leaf spelling. |
| MSVC-product | clean final38Rust compiled | exit0/69640ms;93warnings retained | Final clean source debug bins; no release timing/signed candidate claim. |
| full-original-adapter | incomplete | not run | Single-file mechanism only; no completeSD, intent/current-refCAS, directory/tree custody/work copies/restore/Closed. |
| H-real-GUI | incomplete | prior empty-host evidence reused | No official Owner/GUI integration acceptance. |
| S-D-production | partial | previous evidence reused | Full source-bound production/required family matrix not closed. |
| ordinary-fullSD-MSI | not accepted | no repeat | Ordinary SD environment missing; administrator MSI does not close ordinary MSI. |
| cleanup | synthetic objects only | all native success guards passed | No new account/Profile/Public/force; old rejected tree/Profile/Public/schema untouched; cache/evidence retained. |
| constraints | maintained | noW2/dualSID/publish | No official/daily config/PATH/policy changes; no real model/business sync. |

## 清理和未完

成功测试7个C任务fixture根及2个准确新D cache子目录按源内guard收尾，失败时guard会保留证据但本轮无失败；D guard只remove_dir精确新空目录，不递归父根。原句柄/父pin先释放，再删除自有synthetic树；不靠tasklist声称无原件残留。没有新账户/Profile/Public根/强清/官方服务/日常PATH配置策略写。cache/实际EXE/不可变证据留用；原7文件670767字节失败树及所有历史Profile/Public/schema自动审批拒绝未重试/绕过，overallCleanupComplete=false。

后继仍须将原完整SD捕获、意图/完成记录与current-ref CAS、同卷目录/整树custody和工作副本、全量原件恢复/Closed及broker生产路径真正组合。H实际GUI、S/D整组实机proof、普通MSI验收与普通完整SD/SACL环境缺口保持；不从本轮fileID正例外推完整权限或生命周期通过。原30/5/10、Restart30含Stop子10与Unknown拒绝不变。

无真实模型/业务同步，无W2/双SID/发布。离线python verify_evidence.py只检查归档/源/结果限定，不做Windows进程、移动或清理操作。

离线校验器首轮错误：把final-source中的build-wl2.ps1查到仅Rust映射导致KeyError/exit1，失败脚本及stdout/stderr/result在replay-attempt01完整保留；改为对actual captured source逐字比较，原native结果/源码和边界断言未改，最终重放另存。
