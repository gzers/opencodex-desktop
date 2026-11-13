# 保留逐帧来源拒绝的元数据核验 r1

revision: 1；2026-11-13；IMP-34 P24-04；main 098ae4a15353ff92cb42d3bd78e393c9d6909f57；accepted=false，非固定候选。承接 [恢复计划同事务登记](client-restore-registration-r1.md)。

## 不采用首末帧替代逐帧来源检查

只在 Original／last 检查来源会改变必要拒绝边界：Original 后临时篡改 created.json 内容、权限或 generation namespace，计划到 PreStop 后恢复，旧实现应在 Ready 的 archive 前拒绝；首末方案可能继续读取中间 archive 并最终通过。完整前缀最后才返回，也不能补回中间的前置门槛。静态审查已否决该方案，生产代码未实现它。

新增反例明确要求只观察到 Original，Ready 已进入 frame／provenance，但 archive 段仅进入一次；PreStop 的恢复回调不可达，篡改现场未被自动修复。该测试验证当前实现的提前拒绝，不冒称已经执行一个删检查的错误实现。每帧 provenance、四份来源文件鲜读／摘要／保护与动态 anchor/profile/client、Original baseline、archive 后 namespace、前缀末尾 namespace 全部保持。

## 实际优化

只合并 verify_indexed_anchor 中内容读取前的三套重叠目录元数据观察：仍先鲜读并执行原 profile／data／client leaf policy，再以既有 PinnedDirectory::recheck_set 双遍完整核验路径、FD、identity 和完整 Protection。全部成功后才进入 namespace 枚举和来源记录内容读取，后面的 intent.profileProtection 与 created.rootProtection 当前比较不变。safe_profile 的其它调用保留原 recheck＋原 leaf policy。

这调整元数据采样顺序，不声称每个并发时刻的观察轨迹完全相同；没有减少逐帧来源检查或把此前的结果变成缓存／权限。实际 diff 的独立只读补审无阻断项，结论限于完整内容读取前准入保持，不代替测试或原包验收。

## 验证与收益限制

58 项不同受影响测试通过：root creation 11（含 3 个新元数据／瞬态篡改用例）、closed history 20、root stop binding 9、root originals 9、restore registration 4、版本化命令 4，以及单独的新集成计量 1。命令中 1 项原有原包用例仍忽略。Clippy all-targets -D warnings、fmt、diff-check 通过。没有重复计入先行四项与 root creation 三项重叠。

单个元数据块的 protection 调用 47→21。四个独立同形二代恢复登记按旧／新／新／旧计量：总体 protection 40322／39334／39334／40322，耗时 469.826／458.168／466.978／462.210ms。每次 frame/archive 26、capture 217、provenance 38、Originals namespace 37、完整历史观察 3／内容双遍 6 均相同。总体属性读取少约 2.5%，时间差小且重叠，不声称显著端到端加速或正常 Stop10 已修复。

证据 [.adg/evidence/0.2.24/client-root-metadata-r1](../../../.adg/evidence/0.2.24/client-root-metadata-r1/manifest.json) 包含开跑前假设、方案否决和补审边界、全部命令及压缩原日志、计量、五个变更源文件和测试二进制摘要、重验脚本、清理回执。unit TMP 为空，原字节归档／解压逐字比较后清理本批临时根。无日常配置、Shell、已安装 app、官方 2.99 原件或业务请求变更。

## 后继

源码已固定并开始构建当前 release 库测试宿主，累计事务／目录／元数据优化后的新原包双轮诊断独立留证：严格保留 Start30／Status5／Stop10、正常退出、索引／原件／监听和清理证明，不用第一轮通过代替整轮。编译和诊断的意图不算通过，本批 unit 证据自身没有官方服务轮次。原上轮第二 Stop12143ms 失败保持。根中断／缺 Ready／一般客户端图路径与完整进程集合／刷新／NAME／复杂代理／Windows W1W2／交付验证和固定候选等 IMP-34 原完整范围仍待完成，目标 active，未发布。
