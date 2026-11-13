# 后继根启动准备与状态同源资格 r1

revision: 1；2026-11-13；IMP-34 P24-04；main 8b0dd88df。accepted=false，非固定候选。承接[多代来源与目录集合](client-root-lineage-r1.md)。本批接入实际 Active 启动准备/专用捕获选择、readonly status/recovery-preview，验证合成隔离场景；尚未运行原官方包连续两轮服务生命周期。

## 产品入口

Index::prepare_start_capture 是实际 Active 使用的统一准备边界。逻辑根缺席且无 index 时只走原 create-only 首次登记，原残留初始化证据仍拒绝；已有 index 则在准确当前闭合 RootAbsent 的原 action owner 内创建 successor，不释放后重新取得锁；逻辑根已存在则保留原普通 existing-root 入口，其历史/绑定不符继续拒绝。

Active 依据已登记原缺席与初始 Preparing.rootGeneration，分别调用首代或 successor 专用 reservation / Original 登记；后继初始元数据仍只保留在不可变事件。普通根继续走原 capture/advance；半成品 Preparing/CaptureReserved 重开不会续捕获、刷新 deadline 或重建 created fd。新用户根、链接、旧证据变化、未知 slot 或上下文不符不自动采用。

## 同源资格与只读边界

私有 successor_eligible_checked 仅消费已由完整 Index/observer resolve 审查的 Head、全 history 和独立 origin，核当前 RootAbsent、当前根缺席、相同 attempt 与准确 namespace。held-Index Start 与 readonly status/preview 调用同一判断。

完整常规根周期的九个事件在创建目录前预检容量：按当前准确 scope 和最大新根数字宽度构造序列化形状，UUID/hash 的绑定字节编码取安全最大宽度；保留原 4096 events、4MiB history、65536字节/事件和512代上限，不改成较低统一限额。只承诺有空间记录下一常规周期，不把有空间当服务成功或无限恢复重试承诺。容量不足时 status Required、preview Pending、实际准备拒绝，且不生成新 root/slot/event。

OfficialStatusSource.client_recovery 使用单次 observer 回调完成映射，不先读一次 Head 再重新查询资格。合格 RootAbsent 映射 NotRequired；已登记新 Preparing/Captured 映射 Required；不可信事实返回 Unknown。恢复预览合格闭合映射 Restored，同时保持已验证 Original/plan/completion；未闭合 Prepared 仍 Pending。该变化使用已有状态/控件，无新增持久或 IPC 格式。

observer 为根会话提供完整 bounded history，并在原只读 SQLite 事务内执行 projection。回调后重 resolve 与全部 binding/Head/history、VFS/paths/pins/namespace/归档检查，事务关闭后再次确认返回边界。另通过开锁前已有 pin FD 的 read_at 完整读取有界索引字节及 stat version，观察前后比较；不靠 immutable SQLite 缓存重读证明外部同 inode 没改，也不额外打开/关闭 SQLite inode 的辅助 fd。持续外部变化拒绝；不声称消除任意外部并发最后瞬间竞争。

## 验证与未完

最终127个唯一受影响函数通过（新增产品准备8、索引/捕获56、根归档37、多代8、命令4、状态来源14；5项ignore不计），strict all-target Clippy、fmt、diff通过。

证据目录 .adg/evidence/0.2.24/client-root-start-r1/ 固定8源/实际库测试 binary、开跑前假设、逐轮结果及清理。新增8项验证首代与后继/普通根区别、旧闭合→准备/捕获的真实产品准备路径、readonly 不写盘、实际 OfficialStatusSource 与预览/Start一致、半成品不能续捕获、过期无写、坏场景和回调后变化拒绝。

容量反例在自有SQLite构造4095个合法重复闭合事件，先证明历史仍有效，再证明新准备、readonly与status均不放行且未创建新目录。纯容量边界同时检查事件数、精确字节数、单事件和代数。外部子进程实际改写同一 inode 页字节，父进程保留原只读连接且不额外打开 SQLite fd，dev/ino 相等、子进程成功和 observer 拒绝均有断言。不是模拟“只返回错误码”。

本批不执行官方runtime/service或全局协调DB，不改日常配置/Shell/已安装app，不发WebDAV/模型请求或构建固定候选。原Start30/Status5/Stop10/Restart30及其Stop子预算不变，未知拒绝不变；仍需 release 原包至少两完整 Start/Status/normal Stop 循环，重点检验第二代真实阶段链、正常退出及旧归档保持。动作完成回执的可启动门（尤其容量耗尽等边界）、根中断入口和相关UI/原型收口仍随真实生命周期核验，不把这批状态投影通过冒全部门禁关闭。

缺Ready、其它客户端图路径/完整进程集合与活跃刷新、NAME、复杂代理、Windows W1/W2、版本发行验证/README和固定候选等 IMP-34 全范围保持，P24-04/05及整版未关闭；发布仍另依有效授权。
