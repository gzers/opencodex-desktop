# Windows 真实 30 分钟错误退避

2026-10-12，Asia/Shanghai。关联 IMP-20/TASK-182；只记录实际生产调度器的本次错误退避场景。此前“观测仍在运行”的记录为中间状态，本页补充终态。原 TASK 对象未恢复，不创建正式 RCP 或关闭证书。

## 实际样本

同一台 Windows Server 2025 Datacenter build 26100、同一个隔离根、同一个普通令牌应用 PID29936，全程未重启产品。源码为 `main@afff490a430a81f8c233d44928aa3681a23e91e0`，EXE SHA256 为 `bf8a39e977ce5c554923f5f826a02b8f71cc37ed00ac08c258e436f104d9a7a1`。

证据归 `.adg/evidence/OCX-0110-20261012-WIN-REAL-TIMER-BACKOFF/`。记录器透明包装实际 setTimeout/clearTimeout，安装后校验函数身份；原回调、延迟、performance.now 和 Tauri 桥均保留。未注入业务响应、加速时钟或解除此前已存在但起点未知的内部 floor。

生产资源 `index-BvePWDig.js`（SHA256 `dde7f6265ed0e8fa40546490177fffeb73e48f31d987193819ac60ff877b9535`）中的唯一匹配回调为 `()=>{t=null,u()}`。保存的调用栈和源码将完整 1,800,000ms 定时归于 catch 分支 `performance.now()+18e5` / `l(18e5)`；错误来源由受控 schema99 调度 JSON 与该分支推断，未直接采集原始 native 命令错误。

| 项目 | 记录结果 |
|---|---|
| 新错误分支锚点 | 2026-10-12 16:00:52.196（UTC+8） |
| 首次真实回调 | 2026-10-12 16:30:52.308（UTC+8） |
| performance.now 单调耗时 | **1,800,001.5ms**，无更早记录回调 |
| 周期观察 | 79 次，其中 60 次关联新锚点 |
| 唤醒 | 200 个生成的 route/online 源事件；不代表物理断网重连 |
| 计时器 | 关联 arm 154 次、真实 fire 1 次，观察中最多一个有效计时器 |
| 真实最小化/恢复 | 隐藏后有效计时器 0，恢复后 1，仍遵守原锚点截止 |
| 清理终态 | 16:31:10.772 完成，监测 exec session31752 exit0 |

此前只读 invoke 属性导致记录器未安装的失败准备保留在 `OCX-0110-20261012-WIN-REAL-BACKOFF/`，未改成通过，也未尝试绕过只读属性。

## 恢复与独立复核

调度文件测试前后原始 SHA256 均为 `dd496d70555b67640ccde0b03f47d04c6d772281c6d052de8b3eed7f2f5f3939`。清理恢复原 setTimeout/clearTimeout；独立原生读取确认自动检查恢复 false、通道 stable、invoke 身份不变。仍为唯一隔离候选，日常安装和真实数据未替换。

`verify-completed-run.py --build` 冻结原始字节清单，再无参数运行校验全文件集合、哈希、来源、实际回调耗时、真实隐藏/恢复、清理及独立事后读回。`independent-recalculation.json` 保存由原记录重算的结果；不以监测 PID 后续可能复用或旧状态文件单独证明监测仍存活。

本次只通过记录场景的真实 30 分钟 timer 下限。不据 timer 数推断原始网络/命令次数，不覆盖系统时钟变更矩阵、所有事件/在途通道、完整性能或 24h/6h 周期。原预算、既有性能 failed、其余原生/安装/更新门禁与 TASK 原件、脚本二次确认缺口保留；IMP 继续 in_progress，不关闭或发布。
