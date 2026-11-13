# 按已登记闭合历史核对保留原件并取消准备

revision: 1；2026-11-12。归IMP-34 P24-04，承接[捕获前取消](client-prepare-cancel-r1.md)。后端main9ba2fc655、界面说明mainae3d08c5c。accepted=false、非冻结候选。本批扩展当前版本已登记且已闭合的保留原件目录；部分捕获/未登记原件仍不接管，普通Stop新实测超预算仍失败，不冒整版恢复或预算完成。

## 当前合同

先读独立索引事件，再查保管目录。Preparing/CancelledBeforeCapture观察者在原shared action锁内复用已有SQLite connection读取有界已提交events，不开第二个索引连接/辅助fd；排序revision必须从1连续到当前head、事件个数等于当前revision、payload与解码后原Head规范序列化逐字一致，最后event必须精确当前头。边界4096事件、合计4MiB/单条64KiB、最多512保留attempt；越界Unknown/拒绝，不截断或猜测。当前schema3中事件原文和当前CAS门保持。

从当前Preparing之前的事件引用推导每个attempt的最后状态，原Original/scope SHA不可被历史rebase；每个保留attempt的最后引用必须合法Restored或Unchanged且独立原件、scope/live、全部阶段/plan与已登记完成回执可读匹配。Rollback/未完成/损坏或缺失引用都拒绝。没有按目录名、文件最新时间或当前内容生成References。

只有这一闭合attempt集合能与当前client-originals目录根成员做精确比较：缺原件、多出未登记目录/文件、空保管目录却无历史引用、链接、不安全权限/目录替换或边界读取变化均拒绝保留。目录枚举仅作集合核对，不收养扫描发现物。原生pin和前后目录名/身份/保护读回绑定，旧历史回执也在每次新review/readback/关闭前复核；有任一陌生部分捕获都不给取消能力。

取消receipt schema2新增retainedVault证据，固定原事件历史摘要、保管目录原生身份/保护及每个独立attempt目录身份和闭合Head摘要；确认指纹同样绑定它。无vault/无历史的既有取消分支仍可用，unknown vault分支仍拒绝；这不是旧软件路径/运行包兼容或迁移。当前开发版receipt格式直接严格读取，未新增旧格式补救或忽略字段。闭合观察使用原Preparing之前的精确事件前缀重新核对retainedVault，不能从后来的新原件漂移修复旧receipt。

原cancel原生权限/完整镜像/当前OS stopped/runtime-pid缺席/当前完整两树稳定、Proof/CAS及原10秒均保持。不写/删/搬动任何旧原件或客户端文件，不把取消当原件恢复，不自动启动。取消后新出现未登记目录仍Unknown/拒绝下一Start，只有随后用户明确Start才能开始新的原件捕获。原完整保留目录里的合法用户文件不作为当前客户端替换来源。

产品IPC/Preview仍复用原取消命令与独立准备摘要，没有把历史原件内容或目录清单输出DTO。界面与唯一总原型确认改为“本次准备尚未产生新的原件目录”“保持当前配置和已保留原件”，消除有历史vault时误说整个vault不存在的错误。其它原型/index及界面流程不改，无真实API/文件演示。

## 实际检查与新失败

27去重后端：原25索引函数过；新增历史parser函数验证canonical/连续revision/当前Head精确一致/有界payload与事件数量；新增历史原生文件函数验证原Unchanged闭合原件保留、预览摘要等于独立原生review、未登记partial目录拒绝保留、坏旧完成回执拒绝保留、取消后原Original字节和receipt原文完整、晚到未知目录阻止下一Start、下一明确准备仍可取消。DTO不泄漏旧attempt路径/原件内容。最终只读阶段日志/测试helper加后编译记录有unused client warning，移除该测试变量，最终格式/编译/全目标严格Clippy通过。前端71项/类型/生产build和WebKit3软件/原型受影响说明场景通过，暗色确认截图已查看；原交互其它分支按未变证据复用。

真实一个新隔离Profile使用未改官方2.99，先构造有真实Restored原件的前置。Start9175ms/Status240ms；Stop原父子normal0/4TCP缺席、三文件补偿/全树与两稳定锁读回已完成，却在close-session-index处超过原10秒：Stop10539ms、原函数exit101/failed/RestorePrepared revision6。日志保留原阶段耗时和首错，**普通Stop预算仍失败**，没有因为文件全还原或后续成功改判通过。此轮没有进入预设的第二次Preparing，这是实际前置失败，不能把原计划当已执行。

不重跑整轮或安装器/双SID：先保留原index6/frames/plan/receipt，再沿已有显式metadata收尾处理同现场，9753ms独立动作完成Restored revision7。原Stop10539ms失败不变，不能拼两个10秒声称一次Stop过预算。随后仅一次明确Start在新capture前按预设中断：961ms/index8 Preparing，无新服务启动、原vault保留，回执StartingFailed且三动作false/恢复限制true，预览依据独立历史可取消。

再新明确cancel动作，错revision/digest、临时未知partial attempt目录、陌生runtime记录均拒绝保留现场；测试自行清除合成注入对象后，不重拍基线取原确认。实际取消8643ms、小于原10秒，index8→9 CancelledBeforeCapture；重复旧确认拒绝，真实新状态NotRequired、无PID/port、仅canStart。回执noVaultCreated=false明确是旧vault保持，不是本动作创建了新vault。

独立只读native完整当前Codex/Grok树摘要与新取消回执逐字匹配；新receipt摘要匹配index9/9events，保留原失败index6、独立收尾index7、新准备index8和最终index9。29历史原件/frame/blob/计划/outcome/旧完成receipt的字节SHA、inode和mode在取消前后完全相同；15原内容blob按名称摘要核对。另7当前客户端路径原字节/缺席匹配。没有把旧历史摘要当原正常退出或当前写许可。

来源界限：原生首次服务轮用native/source.json所列源；失败后新增定向next-Preparing测试helper和固定阶段/耗时日志后，另build的binary供显式收尾/新准备/cancel，摘要独立记录；最终只移除测试unused变量，不为此重跑服务。产品历史核对算法与确认语义由实际执行源/定向tests同源证明。未建新.app/DMG或固定签名候选GUI→IPC整链，不声称Windows本机通过。

## 证据和收口

[本批目录](../../../.adg/evidence/0.2.24/client-vault-history-r1/)与manifest保留原Stop超预算目录及独立后继动作结果，不成功覆盖失败。原编译warning和最终strict Clippy结果并存。native OS sandbox拒绝日常HOME/外网，仅loopback，无真实业务或模型请求。自有进程/原主端口明确缺席、3个实际新协调DB经EXCLUSIVE重获释放后与完整隔离程序/Profile/原件/测试副本清理；既有globalL inode/字节/模式/owner保持、可排他重获释放且无sidecar。unit TMPDIR为空，browser/Vite退出；归档逐字/SHA核对后删除重复临时根，正常target/dist保持，不发布。

下一步优先定位并改善新Stop10539ms预算前置，保持原10秒和完整性断言；只在新假设影响阶段做定向验证。历史vault取消这一分支虽原生过8643ms，仍不代表多历史/一般大树都能过预算。真正未登记/部分捕获、早期文件已变但无plan、首次缺根/一般客户端和路由图/完整进程集合/旧revision4首错、Windows贡献和集成/NAME/复杂System代理/官方发布验证/README/固定候选继续，目标active。Windows本批未查新远端，不以此前无回交状态冒本时点新事实。
