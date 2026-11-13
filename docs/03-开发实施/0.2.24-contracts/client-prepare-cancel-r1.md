# 捕获前准备的显式取消

revision: 1；2026-11-12。归IMP-34 P24-04，承接[早期原件确认](client-early-unchanged-ui-r1.md)。后端main2c6cd6e05、界面main1524b6a97。accepted=false，非冻结候选；只闭合独立Preparing且整个client-originals保管目录缺席的分支，不能代表全部Preparing/部分捕获恢复完成。

## 来源与适用边界

真实代码顺序固定为reserve_start持久Preparing→官方原生协调/Original及scope捕获→独立Captured提交→官方Start。Preparing无refs时不能据此生成Original或猜测历史内容。当前整份client-originals保管目录明确缺席时，说明这次Desktop没有走入原件发布后阶段；可以另起用户确认动作取消准备，但不称“原件保持完整”或“恢复完成”。

取消要求head严格Preparing、refs/plan/completion均空，revision/ticket及完整绑定不变；descriptor-relative无跟随lstat证明client-originals路径缺席，不读目录内容、不扫描或收养未登记原件。即便是空目录、合法历史会话留下的保管目录、部分frame/blob、symlink或未知不可读路径，也拒绝并保留。该严格条件会使“有任何既存保管目录”的准备中断仍待后续处理，不暗当旧版兼容或清空后重拍。取消只保存当前观察摘要，不将当前内容当丢失的启动前原件。

## 执行与独立终态

新macos_client_prepare_cancel.review/confirmation在既有shared action锁内绑定当前head/ticket、索引scope（UID/D/Profile/HOME/Codex路径和原生目录身份）以及当前完整Codex/Grok树摘要，复核保管目录仍缺席；这只是当前确认，不是还原来源或写许可。已有同ticket/revision取消回执必须私有且逐字匹配原head/current摘要，坏值保留拒绝，不能改写或自动收养。

新cancel动作Index.open_existing绝不创建缺失存储，排他锁内复核request attempt=ticket/revision/digest。完整当前官方镜像/入口/版本选择准入后，真实当前用户OS归属扫描必须Stopped、runtime/pid记录缺席，两次查询保持；metadata读回前后仍复核。全程不调用Originals.begin/Pending/apply，不修改、移动或删除任何客户端文件，不自动启动或建立新原件基线。当前完整两树在多次只读观察中保持，不因不可读/链接/权限问题而当空状态。

create-only私有prepare-cancel-<ticket>-<原revision>.json记录原Preparing head、绑定摘要和本动作当前两树摘要。独立不可序列化Proof绑定原head/索引binding/期限，Index.close_preparation重新核对原预约、回执、保管目录缺席和原CAS，append新事件变CancelledBeforeCapture。最后提交超时则追加回原pending阶段、原回执保留；后续新revision有新回执名，不覆盖旧失败证据。

CancelledBeforeCapture不同于Unchanged/Restored，不包含Original refs或restorePlan；观察者只读核对已登记回执、原Preparing/ticket/revision/binding；若取消后保管目录又出现，也保持Unknown/拒绝下一次Start，不用旧取消掩盖新原件。正常用户稍后改配置不是旧文件重放理由。独立核对合法取消终态才使新状态NotRequired，下一次用户明确Start可登记新ticket并正常捕获；旧取消/事件不删。坏receipt/错ticket/revision/另HOME/新vault均不放行。

## 产品和原型

Preview新增canCancelPreparation/preparationDigest，Preparing且无保管目录的preview原件/计划/完成均not_recorded，attemptId绑定原ticket；与canFinishUnchanged/Continue/Finalize/Rollback/Rearm互斥。存在任何vault时unknown，不提供取消。产品注册client_recovery_cancel_preparation，只收attempt/revision/digest，沿既有运行排他租约/更新安装与待重启拒绝/binding-ready/worker生命周期；其它平台unsupported，未替代Windows原生。

同诊断中心UiDialog显示“启动准备已中断，尚未开始捕获原件”，按钮“取消启动准备”。明确确认说明当前尚未创建保管目录、重新核对停止后取消、保持当前配置、不恢复或删除文件、不自动启动；可返回。正在处理可关闭重开，但Pinia持有同一promise，不重复提交。成功必须原ticket/revision+1及cancelled_before_capture，专用成功文案，不接受unchanged/restored冒同一结果；只有fresh live同D/H/Stopped/NotRequired及无PID/port才能解除原UI限制。失败清过期preview，限制保留；成功后读取失败只刷新状态，不再mutation。

唯一总原型只改client-recovery-024.js，新增prepare/slow/error/refresh-failed固定内存场景，同按钮/文案/返回/在途重开和状态重试；不改总index/其它并行原型，不读写真实文件/API/系统配置。

## 实际验证与证据限制

- 25个索引函数通过，含新增两函数覆盖无vault才成功、空vault/部分原件/symlink拒绝、晚到vault阻止关闭和新Start、错head/旧revision/期限/坏回执拒绝、取消后合法用户改动保留、旧ticket/event/receipt完整保留和下一明确Start新ticket。最终产品预览摘要与原生review同值、原件not_recorded、已有vault无取消能力的定向增量通过；原23项根据依赖复用，未修改原生OS严格性。编译/格式/全目标严格Clippy过。
- 一个全新隔离Profile供应未改官方2.99。在独立Preparing后、首次capture/官方服务Start前按预设中断：Start161ms、index1 Preparing/refsnull/vaultabsent、serviceStartedfalse/recordnull；真实失败回执StartingFailed且三动作false/clientRecoveryRequiredtrue，验证前批失败回执门已接入。原完整函数exit101/failed/index1保留，不冒Start成功。没有继续无关的Status/Stop或完整安装器/双SID验收。
- 另进程取消：错revision/digest拒绝且索引不变；临时合成partial vault原件被保留拒绝；synthetic runtime记录同样拒绝不删，测试自行清除这些注入fixture后再次取原确认。实际正例2722ms，仍原10秒，index1→2 CancelledBeforeCapture，重复旧确认拒绝，无vault创建/客户端重放。真实新状态NotRequired、无PID/port、仅canStart。独立只读native完整当前Codex/Grok树SHA匹配取消回执，回执摘要匹配index2，index1/index2+2events保留，另7客户端路径当前原字节/缺席保持。这里没有Original blob或历史退出证明，不能将当前摘要冒恢复来源。
- 71项前端/类型/生产build过，覆盖显式确认/返回零mutation、错误状态回执拒绝、方向/证据互斥、未知不显示能力、关闭重开单promise、成功后状态失败只读重试。DEV fixture/固定UUID/mock文本排除于生产dist，真实取消命令与确认存在。
- 首WebKit/Chromium各9个summary是软件5项完整流程+原型4项只打开窗口，原型取消交互分支误接到软件循环，虽green也不当完整原型验收。first-browser-coverage-limit.json明确此证据范围，首目录保持不改。检查缺失原型确认截图后修分支/去不可达重复代码，另存visual-complete-webkit/chromium各9检查，真实走原型确认/返回/slow关闭重开/error/状态重试，软件也过，pageerror空。软件暗色和真正原型确认截图已实际检查。没有通过放宽断言将首轮green当全过。

证据[本批目录](../../../.adg/evidence/0.2.24/client-prepare-cancel-r1/)及manifest保留原Preparing失败/独立取消/完整树与索引、实际source/binary、首浏览器覆盖局限和真正增量结果。原生取消成功使用build-native源，产品IPC/preview及页面后加由最终编译/测试另证，原生取消算法未改；不把早期binary当包含新产品入口。本批未构建新.app/DMG或固定签名候选GUI→IPC原生整链。

OS sandbox拒绝日常HOME/外网，仅loopback；无真实业务或模型请求。所有自有进程退出、原生从未启动服务、0新协调DB实际创建（预列路径不存在），自有Profile/程序/客户端fixture/回执副本清理，既有globalL inode/字节/权限/owner保持、排他重获释放，无sidecar。unit TMPDIR空，四轮browser/Vite均terminal；归档逐字核对后清重复临时根，正常target/dist保留。

Windows只读远端仍b2e89665d/0eef59a01，无新回交，不跑W2。后继仍需处理vault已存在的Preparing（包括当前版本合法历史目录）、真正部分捕获且引用未登记、早期客户端被改写但未有plan，以及首次缺根/一般客户端和路由图/完整进程集合/旧revision4首错、Windows集成/NAME/复杂System代理/官方验证/README/固定候选。原范围未删，目标active，不发布。
