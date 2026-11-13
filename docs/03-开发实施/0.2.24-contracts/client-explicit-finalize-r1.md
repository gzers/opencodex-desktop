# 0.2.24 已完成文件补偿的显式元数据收尾

revision: 1；2026-11-12。归IMP-34 P24-04，承接[只读恢复诊断](client-recovery-preview-r1.md)。代码main 317eb9dee6344a3298cdb2c303b9633cccfd198a。accepted=false、非冻结候选；本批只处理原计划文件补偿已经全部完成、会话索引尚未成功关闭的中断分支，不代替部分文件继续/回退或完整显式恢复UI。

## 分支选择与确认合同

Pending::already_completed只读核对原计划及原/Ready/PreStop/Stopped引用、完整planSHA终态complete outcome、每个实际目标的计划预备原件inode与字节/保护、旧后像归档原inode、stage全部缺席、原Capture三文件完整原字节/缺席。只看到outcome文件并不足够；部分应用、rollback或陌生对象均不具备收尾资格。

Originals::finalization_fingerprint再读整个Codex/Grok原始树和当前树、完整Protection、仅精确两个已采用锁例外与原Stopped绑定ID；若旧completion文件存在必须refs/plan/outcome/两树SHA/稳定ID逐项与当前一致，损坏/不一致不覆盖或学习新值。确认摘要绑定原Head、data/client/profile原生身份、两树SHA、outcome和旧receipt摘要，任何变动重新确认。

预览新增canFinalize/confirmationDigest；只有上述只读核对完整后才给metadata收尾选择，canContinue/canRollback仍false。请求固定attemptId/revision/confirmationDigest，无路径/任意内容/客户端配置参数。不把display/status布尔当写许可，不从旧Live或磁盘receipt重造NormalExitProof/PairExitProof。

新增client_recovery_finalize Tauri命令经当前运行操作租约、更新安装状态和storage写门检查；原DataRoot/HOME绑定未就绪拒绝。先只读预检防止无效请求初始化存储，再通过新增Index::open_existing取得独占action锁与原索引，缺manager/index/action/空index一律不创建；锁内重新核对确认指纹与StartupSession原独立Scope/Live/phase，完整准入当前实际程序镜像。

## 新当前事实与操作权限

这是用户显式发起的新metadata-only动作，独立原10秒预算（同既有stop_timeout），不是给旧失败Stop续时。重新要求当前HOME真实OS归属查询Stopped，原service与launcher两个PID均缺席、原记录port的完整TCP表缺席、运行记录不矛盾，查询前后重复核对。任何PID复用/陌生运行记录/未知进程或端口均拒绝，不强杀/抢端口。

finalize_readback在原固定计划文件已恢复的前提下，再做完整树与两稳定锁原生EXCLUSIVE重获/释放，沿既有原件保护流程；没有调用Pending.apply、没有重写/清理任何客户端文件。已有exact receipt逐字一致才重用，缺失时可按新完整读回创建原名字；坏旧receipt拒绝，不用新摘要冒旧登记。随后当前程序选择/完整镜像仍匹配才以真正新ReadbackProof提交Index.Restored/readback。原文件生成时发生的不确定事件不是从此重建成历史正常退出事实，本次只证明已完成文件恢复当前可安全关闭元数据。

其它平台明确unsupported；Windows独占模块未改，未声称Windows实机通过。前端增加typed finalize调用合同，**没有UI组件或按钮**，没有自动执行/重试。本批不能宣称用户已可从界面完成所有中断恢复。

## 定向检查

71后端去重函数通过：session21、restore21、index23、completion6；初组70后增加existing-open存储反例，重验受影响index/session/restore，原completion算法不再变。4个manual入口ignored不计执行。新增already_completed对部分/rollback/外部修改拒绝；open_existing对不存在manager、动作锁消失、index缺失/空文件均零初始化。原冲突保全、真实外部写者、突退/继续/回退、原件/权限及锁/VFS保护均保持。

3项前端合同、类型检查、cargo编译/格式/全目标严格Clippy通过。Tauri注册入口编译，前端invoke用mock；未执行真实GUI或MSVC/Windows。不是用测试stub给当前停止发许可。

## 一次真实补偿后中断和另进程确认

全新Profile原样固定官方2.99完整供应，OS sandbox禁止日常HOME/外网，只允许loopback、自有根/精确新协调DB及既有globalL借用。测试专用cfg(test) seam只在带私有前缀的owned Profile于真实三文件补偿/整树/两锁读回后、索引关闭前返回明确故障；生产构建不存在该环境分支，不给用户开关跳过校验。

第一进程准确binary e1e9cac22f99b33e89d4b7f983d1c0b82b0c29bac7856c1aac6e5092ecd324ea执行真实Start11864ms/Status240ms、Stop8425ms（内部8420ms），原30/5/10内。原服务/父normal0、4捕获TCP缺席、真实补偿/完整树/两锁回执已完成，七路径原字节均true；故障按预期使原动作exit101/resultfailed、索引RestorePrepared revision6、界面Required保留。原失败summary和独立before-finalize-index.sqlite/json分别保存，后续不改写它。

复核后新增open_existing只收紧显式打开路径，并不改变前一进程普通Stop算法；最终另进程准确binary 067ced229dfeb4a99770e13dd4489c14ba869e706d03782bc4573746d13099a5（final-build/final-checks）读取这个现成合成Profile，使用真实RuntimeHandle/OS进程/完整TCP/镜像与文件事实，再执行真正finalization函数。原二阶段执行源差异逐项记录，不拿工作树SHA冒旧binary来源证明。

- canFinalize只在原文件已完整恢复且旧completion恰好匹配时true；新动作真实耗时6631ms，小于其10秒，索引revision6→7 Restored。
- 错attempt/revision/digest三类确认都拒绝，索引字节不变。
- 合成当前config.toml改为外部内容后preview不可收尾、执行拒绝且保留该内容/原索引；夹具恢复原合成字节后再测正例，没有产品自动补写。
- 合成陌生runtime-port记录仍存在时执行拒绝，不删除其记录/不改索引；测试端只移除自身创建的反例后正例才成功。
- 正例确认后preview Restored、original/plan/completion均verified，canFinalize=false，重复原请求拒绝，索引不再变。
- 显式动作没调用客户端apply；测试config原字节前后一致，最后额外原生Codex/Grok完整快照SHA与原receipt精确一致；三项归档旧后像字节逐项等于原Stopped frame。原普通Stop失败仍失败，不回写为成功。

整项另进程test wall22.083秒包含多次拒绝/预览正反例，**不是**单次finalization预算；只有实际正例内部6631ms与10秒比较，不能把8次场景总时间当一次动作超时或混用。该隔离补偿前置由真实官方链产生，但不冒“任意进程被强杀后所有阶段”完整崩溃恢复证据；部分发布/回退/初捕获中断仍待。

## 证据与清理

.adg/evidence/0.2.24/client-explicit-finalize-r1/保留先写假设、准确每版source/build/binary、71+3检查、普通原失败/故障trace/index6、另进程确认正反例和metadata收尾/index7、原件/Scope/Live/plan/completion/outcome/35内容引用、三项归档匹配、最终完整树和manifest，不含runtime secret/admin token或真实密钥。

原主端点经完整Darwin TCP表framing核对缺席，额外TCP仅准确产品trace，不假独立重放未导出端口表；自有service/launcher/Node/Bun/test/explicit/读取helper均退出。3精确新外部协调DB各EXCLUSIVE重获释放后与自有程序/Profile/全部双客户端blob/index/stage/archive/control测试副本和重复临时根清理。原globalL inode/字节/0600/owner保持、独立排他重获释放、无新sidecar，不删旧对象。无日常/系统/官方配置修改、真实业务请求、新release/app/DMG或发布。

下一步继续部分文件中断的独立当前授权/继续回退、确认界面和错误状态；完整一般图/首次缺根/外部Agent/跨卷/大树/活跃刷新/完整生命周期集合与其它监听、旧revision4首错、WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README/固定候选继续，目标active。
