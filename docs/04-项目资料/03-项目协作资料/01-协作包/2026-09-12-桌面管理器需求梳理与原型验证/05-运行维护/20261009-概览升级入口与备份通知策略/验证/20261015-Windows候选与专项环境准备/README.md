# Windows 431 阶段候选与专项准备结果

2026-10-15，Asia/Shanghai。继续按新目标后台准备，不重复成功ETW，不启动本机产品/安装器。后续实际操作及需用户确认的账户/VM方案见[执行方案](执行方案.md)，方案尚未实施，原门禁和预算不改。

## 精确候选已就绪

固定工具分支`43142bdbcc7a0b9f1325103febae3f44dce407ab`，先核验同SHA push38705201081完整success、五必需job成功、远端/干净worktree及没有已有候选，再单次dispatch HTTP204取得run38709701621/attempt1。17:14:45读到completed/success，package成功/query-etw跳过，未新录ETW。

完整job日志实际checkout43142bdb；包锁回归35组/627passed/0failed/7ignored，MSI和NSIS均在临时runner真实新装/精确EXE首屏/卸载exit0。该证据只证明runner新装路径，不抵宿主0.1.9升级、自定义根/不可写、卸载保留或物理DPI。

| 制品 | 字节数 | SHA256 |
| --- | ---: | --- |
| 候选安装ZIP，artifact11960342769 | 7626479 | 5c91c10a2402d936a7bead216a11421a507c6f750c0651fa079aefd98fd97d58 |
| MSI | 4567040 | 2dc08092319fb91fc2866d010de850ca23bcc7716232ef2ac212b94c4d118dbe |
| NSIS | 3270061 | 6ae167452eead3662ab274dd61f8a2d9c7c78e92db6e35a646faf5cd01877bbc |
| MSI主EXE只读载荷 | 8777216 | 3be7ea60f549e76e80416eb1bee2cba2be4b2dd0f2f4a4d0a2964230b51f82f9 |

两个artifact均核验API摘要、ZIP CRC/安全条目、回执来源版本及包内字节；runner证据ZIP原件保留。宿主MSI以read-only数据库和嵌入CAB解包，主EXE版本0.1.10、完整431源码字面量及摘要匹配精确MSI回执。二进制放代码worktree忽略的`test/software/candidate-43142bd-38709701621/`；未启动应用或安装器，不替换日常版，也不把旧493原生证据改贴431。

## 历史计时复用边界

按旧manifest散列独立复核8份原件：stable/panel原last_success至next_due均86400秒，跨启动后新success距原success分别86564/86566秒，新next_due再次各86400秒，无关beta分区保持。只保留原同根持久截止与跨启动成功事实，不推定直接自动触发或完整请求计数。

旧真实状态失败timer floor重算1800001.5ms，单timer及真实隐藏恢复事实保持。旧afff490/be7395a与当前431的前端scheduler blob相同，后端schedule command/module两个blob已变，所以旧执行不直接判最终源码全路径通过；只补新版受影响路径、原始计数及睡眠恢复，不因退出清空历史记录或要求单进程24小时。

## 来源与环境的真实依赖

17:02:16公开beta元数据404；此前同轮API TLS读取错误保留为错误，不冒充404，17:05:54独立认证Release API才实际取得404。因此beta无可用成功基线，真实6h未启动、不计通过；未发布beta、改来源或时钟。

17:02只读环境快照：native0，Hyper-V模块可见/vmms Running/hypervisor存在，但普通权限Get-VM报VirtualizationException，VM数量/可用性仍未知。总物理内存约31.8GiB、当时空闲约5.0GiB；C/D盘有余量不等于可启动VM。WindowsSandbox.exe和拟用普通账户ocx0110_accept均不存在；未创建账户、改ACL、启动VM或配置网络。方案分别针对同物理机普通Profile性能和独立系统安装保留，二者不互相替代。

17:16:02独立保护检查本机native0、代码431干净，日常EXE43e06bc3…、旧隔离四摘要、原TASK3878bytes/SHA bc7965b4…保持，未正式Mac原生交接。具体正常账户/VM操作待确认，后续产品前台仍需新窗口。

52项独立清单及`verify-readiness.py`通过，含候选、原件/日志、只读载荷、历史复用及环境/来源边界；证据归`.adg/evidence/OCX-0110-20261015-WIN-431-READINESS/`。原失败窗口ETW和根因仍缺、Mac性能failed及全部原预算/八门禁继续开放。此候选是阶段准备用途，不声明版本最终通过；不合并、关闭、签名、晋升或发布。
