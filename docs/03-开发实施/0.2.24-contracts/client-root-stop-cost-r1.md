# 根恢复停止路径的同次核验与持锁事务去重

revision: 1；2026-11-13；IMP-34 P24-04；main ea9220c70。accepted=false，非固定候选。承接[首次缺根生命周期](client-first-root-lifecycle-r1.md)。首次Start通过的既有事实不变，本批四个定向原包Stop仍未满足原10秒；最后一轮原根已移出、官方writer已正常释放且磁盘Retire readback存在，但独立RootAbsent关闭未完成，不能把局部结果当成功。

## 测量先于修改

测试构建加入固定ROOT_STOP_COST阶段名/微秒trace，仅输出admission、registration、native、source frames、absence等白名单阶段，不输出路径、配置、Key或进程参数，release产品不包含该计时输出。首cost-diagnostic测得一次RootRetirementAuthority check约341.024ms，其中registration124.597ms、source215ms；source内部current Original打开118.370ms、scope25.248ms、重复Ready/Stopped读取67.493ms。admission约40–42ms且入口重复。该数据触发以下合并，没有修改原10秒或断言。

## 单次完整帧前缀

Originals::open_profile_origin_prefix返回本次刚完整核验的Original→Ready→PreStop→Stopped前缀，现有open_profile_origin仍丢弃帧只返回原件对象。它不在Originals保存cache，不学新SHA，不能构造生命周期许可。root Pending的live/archive/current prefix在同次来源验证中复用该前缀：避免open验四帧后再frame(Original)、重复open和Ready/Stopped递归读取。

check仍逐帧核独立digest/前驱/uid/root/data/profile identity、完整Protection/ACL/xattrs/root anchor和blob归档字节；当前完整树/Grok仍按原plan和Original匹配。session按同次前缀的Ready/Stopped字节验证startup-live摘要与scope/原refs，再核当前程序来源。通用verify_index_scope的无prefix路径保持原行为，prefix只由同plan/source完整观察给出；实际移动、writer release、写回执与提交后仍执行 fresh checks，没有跨写入复用。

工厂require_pending保留same Originals指针/plan id/hash/原source核对，去掉其后又立刻执行的重复authority.check；apply_completed_stop不再在apply_guarded前额外重复一次完整check。apply_guarded在登记helper目的地前、inner实际原生移动入口及每个原写入/移动/release/readback边界均保持当前许可验证，未产生可绕过mutation的空档。新私有FramePrefix/OpenedPrefix别名只整理类型，未禁用Clippy。

prefix-optimized实测source约67.944ms、一次authority200.995ms，前者明显减少重复I/O；Stop仍超时，保留失败，不把局部加速当预算通过。

## 同一个Codex Guard贯穿首根恢复

普通会话的restore_completed_stop保持原独立事务和释放/完整后查询。本批仅首根路径在artifact恢复前完成正常Stop完整admission，随后创建一次准确Codex Guard；restore_root_artifacts先用root_image核对同NativeLease指针、同Codex目标/当前Image及活体helper，在同permit/Guard保护内恢复原目标，再继续根计划/实际Index登记/retirement。直到原目录移动后才正常官方release、完整released tree及最终full absence。中间artifact outcome不是正常Stop结案或readback许可，错误始终传播，Drop清理不变成功。

apply_retaining_codex只由session的当前root-stop入口消费，当前permit、Codex Guard持续核对；若有Grok备份仍取得其独立Guard并正常finish，Codex不会代替Grok排他。供完整admission的Stop/Pair/lease/refs与deadline在整个事务中保持，去掉的是同一事务中先release再read_complete/重新取得同一Codex锁的重复；所有真正程序来源准入和外层完整镜像校验保持。首根恢复未增加第二种默认协议或改变配置迁移策略。

## 登记事实与一次完整原件核验组合

RootRetirementRegistration::require仍是当前Index facts→完整head(root resolve)→facts，首次登记/外部消费不变。RootRetirementAuthority内窄require_index_facts检查实际借用Index的data/action/pin/数据库VFS、原binding完整字节、origin/initial event、plan、当前完整Head、完整source event。该方法仅原件模块可见，本身不授移动权限。

同一许可checkpoint按facts→native/profile/home→fresh current_prefix完整root/Original/四帧/scope/live/当前程序来源→facts的顺序完成，随后原pair、原Stop全部捕获TCP/manager/运行记录/PID absence和deadline不变。前后facts夹住同次完整原件观察，避免先Index完整root复验再session重复完整root复验；没有返回可跨动作重用的观察结果或缓存。静态复核确认实际data/profile/home/root来源关系与原factory初始完整登记仍绑定，不退化成仅比ticket/plan。

最后facts-coalesced测得一次authority约79–80ms，registration约0.9ms，其间仍有约67ms完整source核验。它只是当前合成Profile实测，不是任意大小/路径的性能承诺。

## 四轮真实失败原样保留

[122文件manifest和verify_all](../../../.adg/evidence/0.2.24/client-root-stop-cost-r1/)含初假设、完整原输出/trace、阶段源码binary、原索引/原件字节、独立核验与清理。

| 定向轮次 | 首次Start | 根恢复进展 | Stop与独立状态 |
| --- | --- | --- | --- |
| cost-diagnostic | 9534ms | 取得基准计时，原root仍在 | 10310ms；Prepared8/null completion |
| prefix-optimized | 7290ms | 同次prefix减少重复；helper已prepared，尚未移动 | 10372ms；Prepared8/null completion |
| retained-writer | 7444ms | 同一Guard完成artifact和实际root移动/relocated，随后超时 | 10287ms；Prepared8，无Retire readback |
| facts-coalesced | 7206ms | 实际root移动、官方Guard released、精确Retire readback已写 | 10334ms；Prepared8/null completion，未成功关闭 |

四轮服务与launcher均真实正常exit0、捕获4个TCP均确认缺席，但Stop返回恢复待处理而非成功；所有accepted=false。最后writer正常release有原回执，磁盘readback按原计划released tree/SHA复验，却没有期限内完成的独立终态，不能把它当Restored/RootAbsent，不能放行新Start。当前下一次Start/根中断/终态UI原未完范围不变。

新Profile、固定原官方2.99及原Bun/Node，沙箱拒绝外网和日常HOME；只loopback合成发现，无真实模型或WebDAV请求。每轮独立源/帧链/计划source和事件1..8逐字核验；最后readback树去除仅本动作临时锁，SHA与内容对应，completion仍null。副本不冒活体文件身份或正常停止许可。verify_native从无损gzip字节重建只读SQLite；verify_all只读复核manifest/来源/最终断言，不修改证据。

## 回归与清理

最终52去重相关函数通过（root-stop4、root-move23、originals26，一项重叠），fmt/diff-check和最终all-targets strictClippy通过。修改scope/live/历史帧/blob、同字节新inode/unknown root/新文件、超时/新端口/错记录、未登记/过期凭证等反例保持拒绝；四实际exit91边界仍保留pending而不从盘上回执结案。原包失败源码与最终6源/binary完全对应：binary 69950e2b94cc6013396f55f0d95096cfcd2009313cdb0917c625d903828b6480，未拿早期binary冒最终来源。

保留Clippy复杂tuple类型/多余借用失败，整理为私有别名和正确引用；RefCell尾部temporary生命周期编译失败以局部结果值修正，未禁用lint或改判定。三次只读审查分别复核prefix、持Guard事务及事实夹完整观察，不编辑/执行native，不代签预算；子审查均已关闭。

四原包进程实际wait终止无残留；精确12个新N/K/historyDB经exclusive重获后删除，预存global L identity/权限/字节保持并重获释放。产品4监听证明仍在；独立清理只复查导出主端口，不冒重新捕获全部secondary。四Profile完整Codex/Grok/manager-state/合成HOME原字节先归档、逐字核验后清理四现场、复用运行包和重复临时证据根；unitTMP空，工程target保留。证据约1.35MB，无重复大型运行包。无日常配置/官方源码/真实业务/新.app/DMG/Windows候选/发布。

## 下一步

下一轮假设必须围绕最后readback之后的authority/最终full absence/Index-close/外层source尾段分开计时和复核，不能盲跑已失败原轮。仍保留所有实际mutation前后、原件/进程/监听/来源和原Stop10预算；只有整个Stop、协调释放、最终来源及期限都通过后才更新验收。随后还需next Start安全滚动、RootAbsent状态与明确中断恢复；Windows W1/W2、NAME/一般客户端图路径/完整进程集合和刷新/System代理/发布验证/README/固定候选等全范围保持。
