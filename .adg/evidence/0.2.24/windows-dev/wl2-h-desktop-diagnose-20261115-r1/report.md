# H桌面失败定位、共享修复与后继准备

2026-11-15 Asia/Shanghai。沿TEAM13/r3/r2，实际首桌面代码f93651d9f42b933c660558e49c22a6d8ceff1b85；修复代码最终ab87ad1261f927bf77a1c757342989105a40ad0a，tree b897a36c2bc69708009ba4c92ff4ec001482b0e7。allAccepted=false、productionExecuted=false；原Hmanager交接未通过，修复后桌面case在此检查点仍待用户启动。原30/5/10、Restart30含Stop子10、raw退出/Unknown拒绝不变。

## 原始事实与首错

实际原桌面observer退出101，等待manager-normal.json超时的原trace已在前轮保存。第一次sameSID采集export01 complete=false，成功取得三份privateJSON后，读取broker.stderr.log因ReadAllBytes默认FileShare.Read无法容纳原有写句柄而中止。首次export原字节/长度/SHA和导出错误均保留；这不是原ACL拒绝的绕过或删除，也不是重跑Hcase。

纠正采集为FileAccess.Read、FileShare.ReadWrite|Delete的有界长度快照，逐file独立记录错误继续采集，原句柄/ACL/原件字节不变。用户第二次从marenben原会话运行后，export02 complete=true、fileErrors=[]，10项原文件全部长度/SHA校验一致、stableLength=true。稳定长度只是该快照事实，不冒称所有写入间隙都被锁住或快照本身有native权限。

三份原native报告确认：bootstrap PID116552/FILETIME134391474143480828、SID1003/session7/elevatedfalse/elevationType1/noJobtrue；挂起原broker PID748/FILETIME134391474259863429，同SID/session/普通令牌，beforeResumeNoJob=true、bootstrapStillNoJob=true、observer当时保留原CreateProcess返回句柄。**H的普通桌面/noJob前提有真实正例**；此前只凭文件存在不能签的两项，现在按原始native报告独立接收，不据此宣告manager连续性或完整产品通过。

manager-normal.stderr.log原始code32，在desktop_tests.rs303对manager_probe结果unwrap失败；broker stderr/stdout均0字节，host-events仅host-created，没有manager认证事件。源码入口先PrivateRecord::open(rendezvous)再连接pipe，局部原生反例复现其双向共享冲突：原create保留read/write描述且shareRead，第二个read/denywrite读者与既存writer不兼容。由原trace＋原host事件＋同API真实32反例定位到该读取缺陷；并非已经拿到内核栈跟踪。原角色/官方包/SD没有参与此失败。

## 修复与定向结果

10d5f7a76：PrivateRecord创建写入flush后，关闭创建写描述；在保留原parent pin期间按只读/拒写/拒删重新打开，并复核原卷/文件ID、私有安全与准确bytes后才发布。更换/共享失败/内容漂移都拒绝，未修改旧ACL或使任何独立写者合法。现存旧未封存rendezvous不会被原地修改。

同提交增加locator_snapshot，只返回非秘密定位字节：原private父目录/对象/ACL/id/重复boundedread一致；允许只读快照兼容原creator writer，但不返回PrivateRecord、NativeSession、RetainedPeer、Owner或任何恢复资格。真正动作仍须同连接fresh挑战及实际OS peer/SID/FILETIME/current镜像准入，不从locator/PID/JSON重建Job。此次是同版本失败testscene显式恢复所需，不做旧版产品兼容/自动迁移。

3bcbc0343保留首次export01，采集修正输出新不可覆盖export02；ab87ad126准备一条用户桌面串行入口，先受限关闭原空宿主，再跑新的修复Hcase并自动导出，尚未实际执行。

| 检查 | 实际结果 | 限定 |
| --- | --- | --- |
| 新native共享反例 | 3项通过/native02 exit0 | 旧32复现、封存后并行reader及拒变更、locator仅字节；非桌面manager交接 |
| 原record身份/不可变性 | 1项通过/record01 | 原ID/byte/pin/拒写/拒重命名/拒覆盖断言保留 |
| pendingphase影响检查 | 2项通过/phase01 | 前缀/缺段/重放/闭包不能mint nativeClosed；无真实clientcustody |
| emptybootstrap反例 | 1项通过/library02 | schema/duplicate/truncate/length/extraUTF8等沿原断言；未重跑SD |
| MSVC product01 | bins exit0/143952ms | debug；62warnings原样保留；无strictClippy/release候选通过声明 |
| 脚本验证 | PSparse通过；Framework C#编译/反射通过 | 仅编译read-only witness与核ACL-at-creation重载；未在控制账户构造processwitness或启动broker |

最终Rust与native02/library02/phase01/record01/product01实际每组36份Rust逐字节对齐，后补脚本独立归档，不冒编译输入。library01为补充locator源码前的诊断检查，provenance-limitations.json保留其非最终源限制；早期诊断EXE已被后继覆盖且没单独存其hash，不补造。实际后继product/测试宿主摘要已收。

## 原空宿主的受限退出设计（待执行）

首次失败后observer的原创建句柄随observer退出已消失，不能靠原JSON重建官方Owner或补签连续性。原broker仍只是一台无官方Owner、无服务Job的empty诊断host。故回收仅发原host自己实现的explicit-recovery空宿主退出；**不是正常官方Stop或一般生产恢复**。

旧host要求exact旧manager镜像fileID，不能把新EXE强塞原I或改它的pin；新privateRecord修复也不会远程改变旧进程。恢复脚本先要求完整export02已归档，并校原D中original-broker/rendezvous真实snapshotSHA未变、原旧fixtureEXE SHA准确。只在新profile/ocx024-h-UUID/D中创建private、字节一致的两份非秘密locator数据副本，原record/ACL/I/scene不动。副本的fileID不同，**从不声称它是原对象或第二个授权journal**；它只让原旧manager以原始代码读到端点，再由原broker根据实际nativepeer、sameSID/session/原current image和freshchallenge认证。JSON不授予退出权限，任何原记录/source/peer漂移都拒绝。

受限退出前另持有QUERY_LIMITED_INFORMATION|SYNCHRONIZE观测句柄，独立核原broker FILETIME/SID/session/image/noJob；该新句柄只有观察和wait，既不是丢失的原创建authority，也没有terminate权。认证退出后只等这同一观察句柄signaled并读rawexit0；未确认则不force、不跑修复场景。原failedD及lookup-onlyD都另做原始Base64/sha导出并保留，不删Profile或现有账号。

新入口C:/Users/Public/ocx-h024-repaired-20261115-r1/repair-h024-desktop.cmd只从marenben自己会话7普通桌面运行。先执行上述一次受限回收，成功后运行修复版新Hbase；wrapper改为兼容共享read及自动同SID导出，不再让一次活日志读取错误掩掉其它独立原件。仍使用actualGUI=false的明确fixturemanager；尚未真实GUI或官方生产链。

## 完整状态表

| 项目 | implementation | test | 限定 |
| --- | --- | --- | --- |
| original-desktop-failure | unchanged preserved | observer101 | original f93651d code,00:30:14+08, manager-normal outputmissing, no originalpass promoted |
| sameSID-export01 | rawverified-partial | 3files + firstsharingfailure | partial bytes preserved, completefalse; first live logsharing read failed; no ACL alteration |
| sameSID-export02 | rawverified-complete | 10files;0errors;stablelengths | originalBase/SID/session match; allBase64/length/SHA independentlyverified; originalsource/content encoding retained |
| actual-bootstrap | nativeprerequisite | ordinary/noJob true | SID1003/session7/elevationType1/pid116552/FILETIME134391474143480828 |
| actual-suspended-broker | nativeprerequisite | noJob true beforeResume | sameSIDsession/elevationType1/pid748/FILETIME134391474259863429; originalcreationhandle retained at snapshot |
| actual-manager-firsterror | failurepreserved | rawWin3232 | oldmanagertrace unwrap atdesktop_tests303, oldhostevents onlyhost-created; source and nativecounterexample locate locator openconflict, no authobserved |
| record-share-reproduction | newnativecounterexample | passed | oldcreator readwrite share1 vs secondread share1 returns32 without anyotheruser/policyassumption |
| record-publication-fix | implemented | newnativepositive/refusalpassed | close writer afterflush, reopenoriginalid/privateSD/bytes readonlydenywrite/delete; pinnedparent spans transition; all mutation attempts stilldenied |
| locator-snapshot | dataonly implemented | newnativecounterexamplepassed | readonly share3 snapshot tolerates liveoldwriter, repeatedboundedbytes/id/privateACL read; returnsbytes, no PrivateRecord/Owner authority; livepeer auth required |
| original-record-regression | unaffectedassertions retained | 1passed | identity/content/pins/write/rename/overwrite refusals stillpass |
| pending-phase-regression | same phase semantics | 2passed | create-only/privatehandles/prefix/replay/gap/digest/refusal, noClosed authority fromrecord |
| bootstrap-regression | unchanged publicmode | 1passed | strictboundedframe/version/duplicates/truncation/tail/UTF8 no weakenedmask/deadline |
| MSVC-product | finalRustcompiled | exit0/143952ms | debug only; warnings preserved; no releasecandidate orClippy claim |
| old-emptyhost-recovery | newprepared restricted script | FrameworkCSharpcompile/reflection and PSparse passed | exact oldfixtureimage + sameSIDsession/freshnativeauth; lookup-only locatorcopies; original observerhandle waslost onpanic, so new witness is readonlyobservation not Owner. Same witness FILETIME/SID/image/noJob/exitwait only, no killrights |
| repaired-desktop | newPublicroot prepared | pending user launch | runs originalemptyrecovery once then newHcase; autologexport onboth outcomes; noofficialservice oractualGUI; no samefailedretest withoutsourcefix |
| O/fullproduction | existingblocked | not-run/not-repeated | ordinaryfullSD gap persists; ClientOriginals custodyCAS/restore plus completefamily/realGUI/production Stop stillnotclosed |
| cleanup | failedscenesretained | no delete or force | oldemptyhost maylive; newPublicinputsintentionallypending; currentmarenbenexistingaccount notdeleted/dailyconfiguntouched; no oldblockeddeletion retry |
| constraints | maintained | noW2/dualSID/release | no newaccount/tokenadjust/policy/realbusiness/modelsync; firstfailureandoldpublic unchanged |

## 实际制品

均MSVC debug/test，非签名release候选，不发布。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop.exe | 75435520 | cbabbabb3e9898b520e52fb67f66333ccc67acbe7ff8e569b71e4bae20b84510 |
| ocxd.exe | 5398528 | 87232b26c722012ed2fd225394a5cb5f4687851d6f8b53060cbe1a9dcca850f8 |
| ocx.exe | 306176 | 7c7b3d5123358cbfd17976e23dee84f81ed18356d41181e64a2cabe9c34bfbf8 |
| H修复测试宿主 | 93421568 | b8a4b8140571a93b2e30911c1d1ff7910515421d0a424f5c81c4349d209b9d75 |

## 清理与未完

原export01/02、原失败D/I/H及原Public根都保留；未改原ACL/所有权、未删除marenben/Profile、未用系统服务/管理员令牌或计划任务替代。新Public根只供一次修复轮输入/输出，ACL只应用于该新自有根，不是日常目录/策略。当前检查点原emptybroker清理尚未确认，后继新case未执行；不以tasklist无残留推断原Job或native成功。工程target/实际EXE原件继续留用；旧自动审批拒绝/Profile/Public/schema未完均未重试/覆盖。

O完整SD权限缺口仍复用，不重跑；ClientOriginals同卷保管/current引用CAS/全量原件恢复、D完整family角色及S/H真实生产组合尚未完成。原桌面noJob正例不抹掉O缺口。无官方runtime/业务同步/模型请求，不W2/双SID，不发布。allAccepted与cleanupComplete均false。等待用户执行新入口后，在另一个不可覆盖结果轮次记录真正后继结果，不把准备追溯改成成功。
