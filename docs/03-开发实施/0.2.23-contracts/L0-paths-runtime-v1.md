# L0 新版路径、受管程序与保存接口合同 v1

2026-11-06，总控基于用户目标模式授权选定。状态：具体设计基线，实际代码/制品验证仍待完成。仅新路径，无旧数据/旧程序兼容或自动迁移。

## 路径

| 域 | macOS默认 | Windows默认 |
| --- | --- | --- |
| 安装根I | 当前用户~/Applications/OpenCodeX Desktop | 当前用户LocalAppData/Programs/OpenCodeX Desktop |
| Desktop | I/OpenCodeX Desktop.app | I/opencodex-desktop.exe |
| 受管包前缀 | I/runtime/opencodex | I/runtime/opencodex |
| 稳定入口 | I/runtime/bin/ocx | I/runtime/bin/ocx.cmd |
| 用户集合根U | OS当前用户HOME/.codex-desktop | OS当前Profile/.codex-desktop |
| 管理器工作根D | U/desktop | U/desktop |
| 默认原版HOME H | U/opencodex/home | U/opencodex/home |

安装器允许明确其它安装根；运行时从真实本体安装上下文解析I并校验安装清单，不从U推导、不扫描旧路径、PATH或独立ocx。开发/隔离程序必须显式提供测试安装上下文，不能从工作目录静默猜I。macOSruntime位于签名.app外、同安装根内；本体更新不删runtime。新包默认与Desktop交付集成，依赖包/平台Node等资源必须固定来源/摘要后随相应制品供应，不让用户自行旧目录导入充当集成交付。无法满足离线供应的候选不得称完整安装通过。

SharedDataRoot表示D，已有manager-state/logs/cache/backups等相对消费者因此落入U/desktop；新入口DTO另给aggregateRoot=U。新data-root结构版本2，内部HOME明确映射H，不再D/opencodex-home。安装根I与D是两个不同参数，RuntimeStore记录在D/manager-state/runtime.json，实际包体/入口在I。程序来源仅managed/unresolved（现有非新版枚举读取不能回退旧来源）；启动/版本/状态/Doctor/安装恢复/更新/卸载共用受管解析器。

## 定位与权限

固定用户文件U/desktop/manager-location.json：{schemaVersion:1,activeRoot:absolute_U}。仅此文件拥有活动集合根指针。活动D/manager-state/data-root.json：{structure_version:"2",opencodex_home_mode:"inside"|"external",opencodex_home_path?:absolute_H}；不再重复active_data_root。内部运行结构可以保留active_data_root=D作为内存字段，但不落盘成第二定位权威。自定义集合根选择须先通过v2结构校验并明确切换，不复制旧数据；migrate_data对外移除或明确拒绝。

固定U/desktop/manager-startup.lock协调当前用户启动，D/manager-state/app.lock保护活动工作根；Windows另沿SID作用域协调并处理不同会话/提权的同用户归属。默认不同用户的D和IPC不同，不用整机单实例互挡。若两个用户显式选同一活动根应拒绝共享，不能以多用户目标允许共同写秘密。ACL/DPAPI属当前用户；缺失可初始化，未知/损坏/无权限明确失败，不扫描旧HKCU/AppData/安装旁data来恢复。

## L0对业务方提供

业务上下文继续使用root:D与home:H；所有业务模块只能使用注入上下文，不自行拼U/I。逐模型来源读取来自原版config的实际modelAdapters[modelId]或provider.adapter，对该模型保留原始来源证据；这不是产品渠道协议，也不由模型名推断。L1负责来源/人工/自动引用Schema和有效解析，L0的config_file_mapping接它的接口，不把导入custom一律作为人工override。

路径快照语义字段（序列化camelCase）：schemaVersion=1、aggregateRoot、desktopRoot、opencodexHome、configPath、installationRoot、runtimePrefix、runtimeEntry、managerExecutable；各有效性为valid/missing/unreadable/corrupted，程序另有packageVersion/manifestVersion/sourceVersion及consistency，不以路径存在等于已安装。L3依据该实际值展示/复制，不硬编码真实用户。

安装身份恢复：prepare只读，冻结operationId、目标程序/入口/包摘要、当前失配快照、选定受管源及服务状态；apply才备份/停止所证明的本用户实例/替换程序/读回入口包清单来源，失败保留旧程序与业务配置。status可重开；cancel未写入前取消，替换中不可假称撤回，完成后明确需重启。正式Rust请求/返回复用现有AppResult与独立DTO，不创建第二安装服务；必要API字段由L0在接线提交提供L3。

## Windows Git协作

L4只用已推送明确SHA和本合同，G0可立即核对本机SID/安装器事实；R1平台编码需WINDOWS文件逐项放行。禁止跨写共享lib.rs/data_root/runtime核心；Windows专属文件单写，跨边界回交接口需求。两真实SID、同SID多会话/提权、端口/IPC/凭据及NSIS/MSI受影响格必须实际证明。RDP可使用，不能拿同用户两个RDP会话冒充两SID。


### 2026-11-07 删除与启动共同准入补充

各显式集合根U下永久保留控制文件`.desktop-removal.lock`，位于D/H外，不保存业务配置和凭据。正常启动在创建D前、单实例生命周期和备份事务期间持共享lease；确认删除对固定与所选D的gate按规范路径排序取非阻塞独占lease，复核现有startup/app/backup锁和原计划后，先释放树内句柄，再在外部独占lease仍存活时删除已选D/H。gate不删除、不替换，避免不同inode分裂锁权威；有活动实例/备份立即拒绝，不停其它用户进程。不扩大旧路径扫描或迁移。取消/只读计划不新建gate。Windows实际证明待候选回归，本机定向51项通过。
