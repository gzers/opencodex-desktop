---
id: IMP-OPENCODEX-DESKTOP-01
object_kind: implementation.change
state: in_progress
title: OpenCodex 桌面管理器实施契约
summary: 用户已确认冻结项目核心 51 项 FZ 与 10 类契约输入，建立数据根、状态、进程、日志、加密、扩展、同步、CLI 和验收的可实现基线；工程任务已分批授权进入实现；TASK-01 ~ TASK-19 已授权并完成隔离实现，TASK-20 ~ TASK-37 已滚动授权并完成状态、进程、真实采集、启动接线、状态周期轮询、进程动作真实路径联动、日志读取、通知中心接入、Doctor 只读诊断接入与扩展 Skills/MCP 只读发现与托盘/原生菜单基础接线、关于页真实契约、偏好真实持久化、数据根真实切换、加密配置迁移导出导入、扩展统一配置写入与 CLI/IPC 真实端点；TASK-43 已接入 WebDAV 端点配置、Keychain 引用、真实连接测试与 TLS 拒绝边界；TASK-44 已接入 WebDAV 真实快照上传/下载、覆盖前备份、冲突登记与同步状态投影；TASK-45 已接入扩展 Skills 软链接与 MCP 配置节点的跨客户端真实写入、路径边界校验、冲突确认、备份与 0600 原子写；TASK-46 已完成扩展资产操作真实执行，覆盖 Skills ZIP 导入/卸载备份/恢复与 MCP 新增/编辑/删除；TASK-47 已完成设置页升级区真实数据，接入官方版本发现、升级前配置备份与展示型官方升级引导；TASK-48 已收口状态采集实机可见：官方 schema 映射、受控状态环境与前端快照初始化全部通过 Release 验证；TASK-49 已完成环境发现默认路径修复，macOS 默认 Node/npm/ocx 统一为 ~/.local/bin 并通过 Release 实机验证，TASK-50 已完成设置页真实数据与原生操作收口，覆盖路径复制、受控目录/配置打开、Skills/MCP 真实路径、立即同步、日志/通知清理、关于页外链与内置授权/第三方文档入口；TASK-51 已将托盘退出按钮接入真实桌面壳退出契约，失败保留当前状态；TASK-52 已接入桌面壳自更新下载安装，本地签名校验通过后安装并 request_restart，失败保留当前版本；TASK-53 已补齐托盘数据目录与快速设置桥，注入 macOS 进程/视图/日志原生菜单，TASK-54 已接入 restore 风险事实与配置影响摘要，备份后仅展示官方 ocx restore 命令；TASK-55 已支持概览直接打开数据根与 OPENCODEX_HOME；TASK-56 已将面板页接入运行期官方面板本机承载；TASK-57 已完成 aarch64 Release 启动、内存与包体候选基准；TASK-58 已复核候选制品、测试、性能与限制，公开发布仍待单独确认。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-01", "TASK-OPENCODEX-DESKTOP-02", "TASK-OPENCODEX-DESKTOP-03", "TASK-OPENCODEX-DESKTOP-04", "TASK-OPENCODEX-DESKTOP-05", "TASK-OPENCODEX-DESKTOP-06", "TASK-OPENCODEX-DESKTOP-07", "TASK-OPENCODEX-DESKTOP-08", "TASK-OPENCODEX-DESKTOP-09", "TASK-OPENCODEX-DESKTOP-10", "TASK-OPENCODEX-DESKTOP-11", "TASK-OPENCODEX-DESKTOP-12", "TASK-OPENCODEX-DESKTOP-13", "TASK-OPENCODEX-DESKTOP-14", "TASK-OPENCODEX-DESKTOP-15", "TASK-OPENCODEX-DESKTOP-16", "TASK-OPENCODEX-DESKTOP-17", "TASK-OPENCODEX-DESKTOP-18", "TASK-OPENCODEX-DESKTOP-19", "TASK-OPENCODEX-DESKTOP-20", "TASK-OPENCODEX-DESKTOP-21", "TASK-OPENCODEX-DESKTOP-22", "TASK-OPENCODEX-DESKTOP-23", "TASK-OPENCODEX-DESKTOP-24", "TASK-OPENCODEX-DESKTOP-25", "TASK-OPENCODEX-DESKTOP-26", "TASK-OPENCODEX-DESKTOP-27", "TASK-OPENCODEX-DESKTOP-28", "TASK-OPENCODEX-DESKTOP-29", "TASK-OPENCODEX-DESKTOP-30", "TASK-OPENCODEX-DESKTOP-31", "TASK-OPENCODEX-DESKTOP-32", "TASK-OPENCODEX-DESKTOP-33", "TASK-OPENCODEX-DESKTOP-34", "TASK-OPENCODEX-DESKTOP-35", "TASK-OPENCODEX-DESKTOP-36", "TASK-OPENCODEX-DESKTOP-37", "TASK-OPENCODEX-DESKTOP-38", "TASK-OPENCODEX-DESKTOP-39", "TASK-OPENCODEX-DESKTOP-40", "TASK-OPENCODEX-DESKTOP-41", "TASK-OPENCODEX-DESKTOP-42", "TASK-OPENCODEX-DESKTOP-43", "TASK-OPENCODEX-DESKTOP-44", "TASK-OPENCODEX-DESKTOP-45", "TASK-OPENCODEX-DESKTOP-46", "TASK-OPENCODEX-DESKTOP-47", "TASK-OPENCODEX-DESKTOP-48", "TASK-OPENCODEX-DESKTOP-49", "TASK-OPENCODEX-DESKTOP-50", "TASK-OPENCODEX-DESKTOP-51", "TASK-OPENCODEX-DESKTOP-52", "TASK-OPENCODEX-DESKTOP-53", "TASK-OPENCODEX-DESKTOP-54", "TASK-OPENCODEX-DESKTOP-55", "TASK-OPENCODEX-DESKTOP-56", "TASK-OPENCODEX-DESKTOP-57", "TASK-OPENCODEX-DESKTOP-58", "TASK-OPENCODEX-DESKTOP-59", "TASK-OPENCODEX-DESKTOP-60", "TASK-OPENCODEX-DESKTOP-61", "TASK-OPENCODEX-DESKTOP-62", "TASK-OPENCODEX-DESKTOP-63", "TASK-OPENCODEX-DESKTOP-64", "TASK-OPENCODEX-DESKTOP-65", "TASK-OPENCODEX-DESKTOP-68", "TASK-OPENCODEX-DESKTOP-69", "TASK-OPENCODEX-DESKTOP-70", "TASK-OPENCODEX-DESKTOP-71", "TASK-OPENCODEX-DESKTOP-72", "TASK-OPENCODEX-DESKTOP-73", "TASK-OPENCODEX-DESKTOP-74", "TASK-OPENCODEX-DESKTOP-75"]
completion_summary: null
source_refs: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
---

# IMP-OPENCODEX-DESKTOP-01 OpenCodex 桌面管理器实施契约

## Metadata

| Field | Value |
|---|---|
| 状态 | `in_progress` |
| 授权链 | `DMD-OPENCODEX-DESKTOP-MANAGER` revision 3/4（需求主体 revision 3；revision 4 仅同步 IMP 对齐口径） → 项目核心 2026-09-14 定稿 → 本 IMP |
| 任务授权 | `TASK-OPENCODEX-DESKTOP-01` ~ `-19` 已授权进入工程初始化、底座实现、UI 原型还原、安装发现、数据根、进程、状态采集、共享契约、写入、同步、扩展、CLI、加密容器与 Skills 模块；`-20` ~ `-36` 已滚动授权状态、进程、真实采集、启动接线、状态周期轮询、进程动作真实路径联动、日志读取、通知中心接入、Doctor 只读诊断接入与扩展 Skills/MCP 只读发现、托盘/原生菜单基础接线、关于页真实契约与受控官方版本发现、偏好真实持久化；后续 TASK 按范围滚动创建 |
| 冻结范围 | 10 类契约 + `FZ-01` ~ `FZ-51` + 建议 `S-2` ~ `S-11` |
| 时间基准 | ISO 8601 / RFC 3339，UTC 存储，界面本地化展示 |
| 编码与路径 | UTF-8；内部路径用绝对路径或以数据根为基准的相对路径，不解释 `~` 为存储值 |
| 冲突原则 | 与 DMD 或项目核心冲突时，先回写权威事实，再修订本文件 |

## 1. 冻结结论

本 IMP 把项目核心里标 ⏳ 的 51 项变成实施基线。重点是：物理目录名用 kebab-case；三维状态内部与 `--json` 都用 `runtime` / `connection` / `operation`；加密用 Argon2id + AES-256-GCM，manifest 用独立 AEAD 包保护；钥匙串服务名固定为 `OpenCodex Desktop`；CLI 入口叫 `ocxd`；允许写入的六个客户端首期全部默认开启，但用户可以在设置里逐个关闭；托管安装只写数据根内私有前缀 `runtime/opencodex/`，默认禁脚本、离线包逐条校验后再原子就位。

- 状态字段内部名与 `--json` 字段名一致，减少二次映射。
- manifest 用「清单 AEAD 包 + AAD 覆盖设备身份与协议版本」的方案，不把口令派生密钥同时用于两类包。
- 扩展投影白名单只做四类机械改写，且每次重写都要展示确认；不做业务级迁移。
- 项目级 `ClientTarget` 只保留模型和落点校验，首期不开放编辑。

## 2. FZ 冻结索引

51 项全部已在 §3 冻结，编号连续无缺号。

| FZ | 冻结内容 | 落点 | FZ | 冻结内容 | 落点 |
|---|---|---|---|---|---|
| 01 | 数据根分区物理目录名 | §3.1 | 24 | Skills 分发方式 | §3.6 |
| 02 | 数据根初始化、重建与切换 | §3.1 | 25 | 配置修订号与比对 | §3.6 |
| 03 | 数据根结构版本 | §3.1 | 26 | 文件锁与并发控制 | §3.6 |
| 04 | 备份保留与清理 | §3.1 | 27 | 写入清单 fingerprint | §3.6 |
| 05 | 删除恢复窗口 | §3.1 | 28 | Skills 来源登记 | §3.6 |
| 06 | 三维状态字段名 | §3.2 | 29 | 冲突判定与解决 | §3.6 |
| 07 | 官方输出映射 | §3.2 | 30 | manifest 字段与版本 | §3.7 |
| 08 | 刷新策略与优先级 | §3.2 | 31 | 载荷切分与上限 | §3.7 |
| 09 | 子进程环境与工作目录 | §3.3 | 32 | 同步时限、重试与互斥 | §3.7 |
| 10 | 生命周期与取消 | §3.3 | 33 | 同步覆盖备份 | §3.7 |
| 11 | 退出码 | §3.3 | 34 | `ocxd` 命令集 | §3.8 |
| 12 | 错误摘要与脱敏 | §3.3 | 35 | `--json` 契约 | §3.8 |
| 13 | 前置缺失文案 | §3.4 | 36 | IPC 请求响应与确认 | §3.8 |
| 14 | 日志来源与脱敏 | §3.4 | 37 | IPC socket 访问 | §3.8 |
| 15 | 日志轮转与保留 | §3.4 | 38 | PATH 注册与卸载 | §3.8 |
| 16 | 容器头字段 | §3.5 | 39 | 单写者锁 | §3.8 |
| 17 | Argon2id 参数 | §3.5 | 40 | 调用审计 | §3.8 |
| 18 | AEAD 参数 | §3.5 | 41 | TLS 失败文案 | §3.7 |
| 19 | 结构白名单 | §3.5 | 42 | 本地口令风险口径 | §3.5 |
| 20 | 导入上限 | §3.5 | 43 | 通知级别 | §3.2 |
| 21 | 备份清单 | §3.1 | 44 | 凭据引用与生命周期 | §3.7 |
| 22 | 客户端清单与开关 | §3.6 | 45 | manifest 认证与防回放 | §3.7 |
| 23 | Skills 源存储 | §3.1 / §3.6 | 46 | MCP 投影白名单 | §3.6 |
| 47 | 托管前缀布局与入口 | §3.9 | 50 | 安装执行策略（脚本 / 代理 / 环境 / 脱敏） | §3.9 |
| 48 | 运行来源记录与历史 | §3.9 | 51 | 托管卸载判定（两级） | §3.9 |
| 49 | 离线包校验与解包加固 | §3.9 | | | |

## 3. 契约冻结

### 3.1 数据根、目录与备份

#### FZ-01 分区物理目录名

| 分区键 | 物理目录 | 约束 |
|---|---|---|
| `manager state` | `manager-state` | 不放明文凭据 |
| `opencodex home` | `opencodex-home` | 只在 `opencodex_home_mode=inside` 时承载；外部模式该目录可为空，不得重复存储 |
| `backups` | `backups` | 权限 `0700`；核心文件按来源收紧到 `0600` |
| `logs` | `logs` | 权限 `0700`；只读脱敏 |
| `exports` | `exports` | 容器文件 `0600` |
| `cache` | `cache` | 可整体清空 |
| `sync state` | `sync-state` | 索引摘要、最后接受快照与冲突记录 |

#### FZ-02 初始化、重建与切换

| 规则 | 冻结 |
|---|---|
| 首次启动 | 创建默认数据根；若目标已有 `data-root.json` 且结构版本可识别，直接引用，不重置 |
| 默认位置 | `~/Library/Application Support/OpenCodex Desktop/` |
| 建立流程 | `mkdir -p` 七个分区 → 写 `data-root.json` → 写 `structure.lock` → 失败回滚删除本次新建分区 |
| 切换检查 | 目标必须是真实目录；可写、空间充足、结构版本可识别；默认引用已有数据根 |
| 引用 | 修改 `manager-state/data-root.json` 后重启应用；代理不强制停止 |
| 迁移 | 停止应用并停止代理 → 备份 `manager-state` 与待迁移分区 → 复制 → `fsync` → 校验 → 原子改名或更新引用 → 清理；任何失败保留原数据根 |
| `OPENCODEX_HOME` | 默认 `inside` 时指向 `<数据根>/opencodex-home`；独立路径先停止代理再备份、引用或迁移 |
| 手工重建 | 设置里只提供「重建管理器索引 / 缓存」；不提供一键重建全数据根 |
| 空间不足 | `backups` 预留 2 GB 或数据量 ×2，取大者；其他分区预留 100 MB |
| 阻断 | 可写、权限、结构版本、空间任一失败即阻断，不进入写入 |

#### FZ-03 数据根结构版本

| 项 | 冻结 |
|---|---|
| `structure_version` | 字符串 `1` |
| 位置 | `manager-state/data-root.json` |
| 当前版本读取 | 主版本等于 `1` 接受；大于 `1` 拒绝并提示升级；小于 `1` 视为损坏 |
| 升级 | 显式升级流程 + 前置备份 + 可回滚；本版本不提供降级 |
| 损坏处理 | 备份现场后停止加载；提示从备份恢复，不自动重建 |

#### FZ-04 / FZ-21 / FZ-33 备份保留与清单

| 规则 | 冻结 |
|---|---|
| 根布局 | `<数据根>/backups/<YYYY>/<MM>/<action>/<backup_id>/` |
| `action` | `upgrade` / `import` / `sync-overwrite` / `data-root-move` / `extension-write` |
| `backup_id` | `bk_<utc14>_<8位随机 hex>`；UTC14 为 `YYYYMMDDHHMMSS` |
| 目录保留 | 每类最近 20 份 |
| 时间保留 | 30 天 |
| 清理 | 保留最近 20 份与 30 天内的所有记录；超出后可清理；默认手动清理，设置可开启后台清理 |
| 备份验证 | 写入后按原始 SHA-256 复验；复验通过才标记 `restorable=true` |
| 清单文件 | `backup-manifest.json`，`0600` |
| 字段 | `schema_version=1`、`backup_id`、`action`、`target_path`、`stored_at`、`created_at`、`sha256`、`bytes`、`file_count`、`restorable`、`note` |
| 目录备份 | 备份整目录为 tar；文件备份保留原名 |
| 引用 | `BackupRecord.content_hash` / `stored_at` 与清单字段同名同义 |
| 删除 | 清理日志只记录 ID 与原因，不记录内容摘要 |

#### FZ-05 删除类操作恢复窗口

| 项 | 冻结 |
|---|---|
| 回收区 | `<数据根>/backups/trash/<YYYY>/<MM>/<item_id>/` |
| 保留窗口 | 30 天 |
| 上限 | 1,000 项 / 5 GB；超限后最旧项先过期，仍提示可备份 |
| 恢复 | 校验摘要后恢复；同路径已有内容时要求确认 |
| 删除备份 | 需确认；删除动作写审计 |

#### FZ-23 Skills 源存储

| 项 | 冻结 |
|---|---|
| 默认源目录 | `~/.agents/skills/`（Windows：`%USERPROFILE%\.agents\skills\`）——Agent Skills 共享目录，管理器**只读**该目录。**这是默认值，不是固定值**，可由用户改为自定义目录 |
| 自定义源目录 | **允许**改为自定义目录；自定义目录同样**只读**。切换源目录只更新管理器引用，**不移动、不复制、不删除**任何已有 Skill |
| 选择方式 | 目录选择一律调用**操作系统原生目录选择器**（macOS 访达 / Windows 资源管理器），由系统负责浏览与授权；管理器**不自造目录浏览器**、不维护自建的目录列表 UI。选中的路径仍当作**不可信输入**，进入下一行的服务端校验 |
| 自定义源目录校验 | 必须满足：存在且可读；入口不是符号链接；解析后的真实路径不得与数据根、管理器写入区或任一客户端 Skills 目录重叠。不满足时**不切换、保留原值**并给出原因。**本行由 2026-09-19 决策新增，参数待安全评审确认** |
| 管理器写入区 | `<数据根>/manager-state/skills-store/`；ZIP 导入 / 从备份恢复 / 备份 / 迁移导出的产物落在这里 |
| 发现 | 合并源目录与写入区；同名时**源目录胜出** |
| 链接源 | 按名解析：源目录优先，其次写入区；两处都无该 Skill 才失败 |
| 卸载 | 只断开各客户端链接；**不删除**源目录内容；写入区中的副本先备份再移除 |
| 迁移 | 只复制登记的 Skill 内容；不搬运客户端目录；失败回滚 |
| 版本 | 源存储加 `store-version=1` |

> 2026-09-23 详情展示补充（用户决策，契约值未变）：扩展列表条目「名称 + 描述」可打开**只读详情**，其中「源目录 / 入口文件 / 文件数 / 体量」由本源目录**按需读取与统计**——只在打开详情时读取被打开的那一条，设大小上限与超时，越界如实报「未知」；不改变本表只读与路径校验立场。字段来源与掩码见 [扩展管理详情方案](../02-项目核心/扩展管理详情方案.md) §2 / §4。**原型已实现并通过原型级验证；生产读取器已于 2026-09-23 实施并通过自动化门禁（桌面端真机验收已于 2026-09-24 完成，证据 `.adg/work/evidence/imp02-accept/`）。**

### 3.2 状态与展示协议

> 2026-09-28，DMD Revision 12（REQ-31 / AC-15）：产品方向已调整为主线＋节点支线、统一结果/错误与简略详情。下列 FZ-06/07/08、FZ-43 字段枚举和时序参数保留为现有兼容基线，**不代表新方案已实施**；旧固定按枚举告警规则不再作为目标展示要求。目标语义见 `数据与状态.md` §2.7、§5.8，展示见 `UI规范.md` §19，字段迁移待按 `契约字段.md` §5.1 冻结。不得只改展示字符串就宣告完成；须验证操作核验、通知解除和各入口能力一致。

#### FZ-06 / FZ-08 状态字段与刷新

内部 Rust 字段和 `--json` 字段统一用：

| 字段 | 枚举 | 优先级高→低 |
|---|---|---|
| `runtime` | 11 项已定枚举 | `loading`、`starting_failed`、`external_takeover`、`at_risk`、`unreachable`、`pending`、`starting`、`stopping`、`running`、`stopped`、`not_found` |
| `connection` | 7 项已定枚举 | `conflict`、`syncing`、`connecting`、`failed`、`disconnected`、`unconfigured`、`synced` |
| `operation` | 8 项已定枚举 | `rolling_back`、`applying`、`backing_up`、`validating`、`failed`、`succeeded`、`cancelled`、`idle` |
| `gate` | 4 项已定枚举 | `missing_node`、`missing_npm`、`missing_ocx`、`checking` |

刷新规则：

| 项 | 冻结 |
|---|---|
| 常态轮询 | 运行状态 3 秒；连接状态 15 秒 |
| 事件优先 | 动作发起、操作结束、外部通知先刷新对应维度 |
| 指数退避 | 连续失败 ×2，上限 60 秒；成功恢复基线 |
| 后台降频 | 无窗口时运行 10 秒、连接 30 秒 |
| 去抖 | 同一维度 1 秒内只触发一次 |
| 超时 | 单次命令 5 秒未返回视为采集失败，不清空上次状态 |
| 未发现安装 | 运行来源未解析、或解析到的 `ocx` 入口已不存在 ⇒ `runtime=not_found`（清空 facts/port/pid）；**不沿用上次状态**（2026-09-28 TASK-160 修正「卸载后仍显示可启动」的缺陷） |
| 展示 | 转圈 >1.5 秒才出现，不闪烁 |
| 冲突呈现 | 先呈现安全阻塞；同维度按 §3.2 优先级 |
| 标签 | 状态显示 max 2 项，溢出聚合成「更多状态」 |
| IPC 推送 | 同一事件最高 2 次/秒 |

#### FZ-07 官方输出映射

| 输入 | `runtime` | `health` |
|---|---|---|
| `status --json` 运行且就绪 | `running` | `healthy` |
| 启动中 / loading | `starting` | `unknown` |
| 启动后端口可达但 `ready` 未通过 | `pending` | `degraded` |
| 端口不可达但进程存在 | `unreachable` | `unhealthy` |
| 进程不存在但安装存在 | `stopped` | `unknown` |
| 官方报告 at-risk | `at_risk` | `degraded` |
| 外部 provider 接管 | `external_takeover` | `degraded` |
| 未发现安装 | `not_found` | `unknown` |
| `health` / `ready` 解析失败 | 保持原状态 | `unknown` |
| 停止命令已发出且进程退出确认 | `stopped` | `unknown` |
| 停止命令超时 | `stopping` + 操作失败 | `unknown` |

规则：官方字段缺失先保持状态，记降级原因；未知值映射为原状态 + `health=unknown`，不得猜成失败。

#### FZ-43 通知级别

| `level` | 界面 | 触达 |
|---|---|---|
| `info` | 中性蓝 | 通知中心，不打扰 |
| `warning` | 琥珀 | 通知中心 + 可配置系统通知 |
| `danger` | 红色 | 通知中心 + 系统通知（可关闭系统触达） |

**级别与渠道分离**：`level` 表达严重程度，渠道由事件的反馈职责决定（六类渠道见 `数据与状态.md` §5.1）。`info` / `warning` / `danger` 不并入操作反馈的 `success` / `error`；替换需另做兼容设计。

##### FZ-43.1 展示渠道与出现时机

| 渠道 | 出现时机 | 位置 | 是否持久 | 是否计入未读 |
|---|---|---|---|---|
| 操作反馈 Toast | 用户刚发起的短操作返回 | 软件窗口内，避开原生控制区、标题栏与通知面板；面板展开时顺次落到面板下方 | 否 | 否 |
| 操作任务卡 | 长任务开始到终态 | 窗口内任务区，位于 Toast 同列 | 否 | 否 |
| 操作性通知 | 需要离开页面后仍知悉或需处理 | 顶部铃铛统一入口；面板页也可访问 | 是 | 是（未读且未解决） |
| 日志型记录 | 技术、审计与排障事实 | 诊断中心 → 日志历史 | 是（日志） | 否 |
| 阻断式确认 Modal | 高风险、不可逆或会覆盖数据的决策 | 居中 | 否 | 否 |
| 页面内状态 | 页面或区域持续存在的条件 | 对应组件内部 | 否 | 否 |

##### FZ-43.2 触达、去重与排队

| 项 | 规则 |
|---|---|
| 主展示渠道 | 一个事件默认只有一个主展示渠道；日志可同时记录，但不得反复叠加 Toast / 卡片 / Modal |
| 系统通知 | 仅 `warning` / `danger` 可配置系统触达；关闭系统触达后应用内待处理记录不丢失 |
| 去重 | 同一 `dedupe_key` 在存活期内只保留一条，重复事件只更新该条 |
| 去重范围 | 重复轮询不产生重复提醒；同一操作终态只触达一次 |
| 排队 | 同一业务域默认只允许一个活动任务；重复触发复用或拒绝 |
| 呈现顺序 | 按 `level`（`danger` → `warning` → `info`）与 `created_at`；同维度状态标签最多 2 项 |
| 低风险成功 | `sync-done`、更新成功等默认走 Toast，不进入持久未读与通知中心 |

##### FZ-43.3 关闭、超时与清理

| 项 | 规则 |
|---|---|
| 自动关闭 | 操作反馈成功 / 信息约 3–4 秒，失败约 6 秒；`danger` 不自动消失 |
| 主动收起 | 通知中心 / Toast / 任务卡右上角「收起到铃铛」；收起**不改动** `read` / `resolved` / `deleted`，也不取消后台任务 |
| 减少动态 | `prefers-reduced-motion` 下直接收起，不播放动画 |
| 等待超时 | 界面等待超时只把进行态收口为「仍在后台观察」，**不写成失败**，不靠固定延迟伪造成功 |
| 清理默认 | 「清理通知」默认只清理已解决；存在未处理通知时改文案并二次确认 |
| 过期 | `expires_at` 为 `null` 的通知不可自动过期；未解决风险、进行中任务、需用户决策的事项不得静默清除 |
| 删除 | 软删除保留历史判定依据；单条删除与批量清理均不伪造「已解决」 |

##### FZ-43.4 跨路由与承载约束

- 通知宿主是**全局壳层**（Toast + 任务卡 + 通知中心），正常路由与面板路由都生效；场景切换器只属开发 / 测试环境。
- 通知中心与「诊断中心 → 通知历史」共用同一实体；任一处改动同步。
- 通知正文与 `action_ref` **不带 S3**；「查看日志」必须同时激活「日志历史」Tab。
- 原生子 WebView 与 DOM 不是同一合成层，不得假定提高 `z-index` 就能盖住官方面板。**承载方案已定（2026-09-19）**：通知 / Modal 采用**协调显示**——通知中心 / Toast / 任务进度 / 确认 Modal 任一可见时**临时让出（隐藏）面板子视图**，表面消失后立即恢复；通知表面仍位于窗口级壳层 DOM，不重排为透明点击层、不复制面板页标记。方案须保证面板页也有可用入口、不遮挡点击、不留透明点击层。

> 视觉参数（材质、尺寸、令牌）只维护在 `docs/02-项目核心/UI规范.md`；本 FZ 只冻结行为契约。通知材质与颜色档已于 2026-09-19 确认，统一按《UI规范》§2.2 执行，不再列为候选。窗口内位置、系统通知范围继续保留候选；面板路由承载方案已按上条收敛。本次材质确认不改变触达、关闭或面板协调显示行为。

### 3.3 进程与错误

#### FZ-09 环境与工作目录

| 项 | 冻结 |
|---|---|
| 调用 | `ocx start` / `ocx stop` / `ocx restart` |
| 工作目录 | 用户家目录；不做项目上下文注入 |
| `OPENCODEX_HOME` | 始终注入解析后的绝对路径；`inside` 为 `<数据根>/opencodex-home`，`external` 为用户指定路径 |
| 继承环境 | `PATH`、`HOME`、`LANG`、`LC_ALL`、`HTTP_PROXY` / `HTTPS_PROXY` / `NO_PROXY` |
| 清理 | 其余环境不主动透传；确需透传时单独确认 |
| 日志 | 环境键可记录，值不记录 |

#### FZ-10 生命周期

| 项 | 冻结 |
|---|---|
| 启动超时 | 20 秒 |
| 停止超时 | 10 秒 |
| 重启上限 | 停止 10 秒 + 启动 20 秒 |
| `pending` | TCP 可达但 `/healthz` / 官方 ready 检查失败，3 秒 ×10 次 |
| 取消 | 未完成启动或停止时取消：停止等待中的动作，已发出的系统调用等待结束；不 `SIGKILL` |
| 取消后状态 | 以最后一次官方状态采集为准 |
| 重启中退出应用 | 显示确认，不停止代理；应用退出后按离线语义展示 |

#### FZ-11 退出码

| 码 | 语义 |
|---|---|
| 0 | 成功 |
| 2 | 参数错误 |
| 3 | 实例未运行或不可达 |
| 4 | 需要确认标志 |
| 5 | 权限或访问被拒绝 |
| 6 | 目标不存在 |
| 7 | 目标状态冲突 |
| 8 | 校验失败 |
| 9 | 执行失败 |
| 10 | 取消 |
| 11 | 超时 |
| 12 | 内部错误 |

官方子进程退出码不改写，原样记录；管理器包装层的失败用上表。

#### FZ-12 错误摘要与脱敏

| 规则 | 冻结 |
|---|---|
| 摘要长度 | 最多 280 字符 |
| 结构 | `[错误来源] 错误短语；hint` |
| 来源 | `ocx` / `fs` / `webdav` / `keychain` / `ipc` / `import` / `extension` |
| 保留 | 常见错误白名单短语 |
| 脱敏 | 移除 URL query 与 userinfo；token / password / authorization / key / secret / cookie 键值替换为 `[REDACTED]`；固定 `Bearer` 前缀处理；绝对路径中的家目录替换为 `~` |
| 日志 ID | 16 位 hex 关联完整日志，摘要里不携带敏感参数 |
| 上限 | 每个对象只保留最近 1 条摘要 |

### 3.4 发现、日志与升级

#### FZ-13 前置缺失文案

| 状态 | 文案 |
|---|---|
| `missing_node` + Homebrew | `brew install node` |
| `missing_node` 无 Homebrew | 前往 Node.js LTS 官方安装页 |
| `missing_npm` | 重装 Node.js，并勾选 npm |
| `missing_ocx` | `npm install -g @bitkyc08/opencodex` |

> 2026-09-23 生产实施说明：日志页按本方案落地「应用日志 / 调用日志」分类与「面板请求日志」入口；日志与 doctor 输出改为纯文本渲染（删除 `v-html`）。**发现并修复一处真实缺陷**：审计写入器 `AuditStore` 把 `audit.log` 写在**活跃数据根**根目录，而读取器原按 `<数据根>/logs/` 拼接，导致「调用日志」永远为空；现按活跃数据根解析（集成用例 `call_log_is_read_from_the_active_data_root_not_the_logs_partition`）。分类日志缺失时不再回退 `app.log`，只有 `agent.log`（尚无写入方）保留回退。**仍未做**：读取器总字节硬上限。

#### FZ-14 / FZ-15 日志

> 2026-09-22 演进说明：下表保留原始日志文件契约；首期分类展示按 [日志分类展示方案](../02-项目核心/日志分类展示方案.md) 进入原型评审。分类名称不等同文件名称，不要求新增 agent/sync 写入器，不改变 FZ-15 轮转策略。当前范围收敛为应用日志、调用日志及官方面板入口；不接入独立代理请求查询。**原型已按两分类调整完成并通过原型级验证；生产分类已于 2026-09-23 实施并通过自动化门禁（桌面端真机验收已于 2026-09-24 完成，证据 `.adg/work/evidence/imp02-accept/`）。**

> 命名口径：界面**分类显示名**为「调用日志」；表格中的 `调用审计` 指 `FZ-40` 机制名与 `audit.log` 文件来源，二者不是同一个东西，改名不涉及文件与契约。

| 项 | 冻结 |
|---|---|
| 命名 | `app.log` / `agent.log` / `audit.log` / `sync.log` |
| 来源 | 应用、托管代理、调用审计、同步 |
| 读取 | 只读；默认最近 200 行 |
| 脱敏 | S3 替换为 `[REDACTED]`；URL query 清除；家目录压缩为 `~` |
| 大小 | 单个日志 5 MB |
| 轮转 | 5 份 |
| 保留 | 30 天 |
| 磁盘 | 日志总上限 125 MB；超限先删最旧 |
| 阻塞 | 脱敏失败整行丢弃并记录计数 |
| 访问 | 只允许应用进程读取 |

#### 升级补充（FZ 之外的既定边界）

本体升级引导官方 `ocx update`，前置备份；套壳自更新按项目核心 §15 状态链，`stable` / `beta` 默认 `stable`，签名校验失败即终止。官方素材不进入发布包；原型快照只用于需求阶段视觉判断。

#### 自更新实施补充

| 项 | 冻结 |
|---|---|
| 发布通道 | `stable` / `beta`，默认 `stable`；本轮只生成 macOS aarch64 发布候选 |
| 更新通道 | `stable.json` / `beta.json`；字段包含 `version`、`platforms.aarch64-apple-darwin.signature` 与 `platforms.aarch64-apple-darwin.url` |
| 签名与公钥 | 公钥录入 Tauri 配置；密钥不入仓库；`TAURI_SIGNING_PRIVATE_KEY` 与密码只由发布机安全注入 |
| 校验 | 签名缺失、不匹配或公钥缺失立即终止；不安装未校验包 |
| 重启 | 更新完成由 Tauri updater 重启；不停止托管代理，重启后重新接管状态展示 |
| 失败 | 下载、校验、安装失败保留当前版本，仅展示失败原因与恢复建议；不自动重试 |
| 门控 | 公开发布前补齐 macOS 签名与公证；当前阶段只验证候选构建与签名链 |

### 3.5 加密容器与导入

#### FZ-16 容器头

```json
{
  "magic": "OCXDCONF",
  "format_version": 1,
  "kdf": {
    "algorithm": "argon2id",
    "salt_b64": "...",
    "memory_kib": 262144,
    "iterations": 3,
    "parallelism": 2,
    "output_bytes": 32
  },
  "aead": {
    "algorithm": "aes-256-gcm",
    "nonce_bytes": 12
  },
  "created_at": "2026-09-14T00:00:00Z",
  "app_version": "0.1.0"
}
```

- `salt_b64` 为 Base64 无 URL-safe、无换行。
- 导入时按头重建派生密钥；不兼容即拒绝。
- 容器为单文件 `0600`。

#### FZ-17 / FZ-18 / FZ-42 加密参数

| 项 | 冻结 |
|---|---|
| KDF | Argon2id |
| 盐 | 16 字节随机 |
| 内存 | 256 MiB |
| 时间成本 | 3 |
| 并行度 | 2 |
| 输出 | 32 字节 |
| AEAD | AES-256-GCM |
| nonce | 12 字节随机 |
| 密文布局 | `[nonce][ciphertext+tag]` |
| 密钥重用 | 一个口令只派生一次；相同口令每容器重新派生 |
| 解密 | 认证失败整体拒绝，不部分导入 |
| 内存上限 | 512 MiB，避免格式化炸弹 |
| 风险口径 | 本地离线口令可被高性能设备尝试；产品依赖用户口令强度与钥匙串保存，不在文档中承诺不可离线爆破 |

#### FZ-19 结构白名单

容器解密后 JSON 顶层：

```json
{
  "container": { "structure_version": 1 },
  "extension_config": { "skills": [], "servers": [] },
  "sync_endpoints": [],
  "preferences": {}
}
```

| 规则 | 冻结 |
|---|---|
| 必填 | `container` |
| 可选 | 其余三个 |
| 未知字段 | 顶层和嵌套对象都拒绝 |
| 数组 | 元素按领域实体白名单校验 |
| 路径 | 只接受绝对路径或数据根相对路径；拒绝 `..`、符号链接逃逸、越界 |
| `env` | 只接受 `{ "ref": "keychain://..." }` 或客户端约定的环境变量引用字段 |
| `credential_ref` | 只接受 `ref_id` 引用，不携带真实值 |
| 引用解析 | 目标必须存在于本机钥匙串；不存在则导入该条目失败，不静默替换 |
| 服务器命令 | 只接受 `command` + `args` 或 `command` 数组；不解析 shell 字符串 |

#### FZ-20 导入上限

| 项 | 上限 |
|---|---|
| 密文文件 | 512 MiB |
| 解密明文 | 128 MiB |
| JSON 解析缓冲 | 256 MiB |
| 数组条目 | 20,000 |
| 嵌套深度 | 64 |
| 单文件 | 32 MiB |
| 解压条目 | 50,000 |
| 解压总量 | 1 GiB |
| 单线程内存 | 512 MiB |

#### 导入流程

校验容器头 → KDF → AEAD → JSON 白名单 → 掩码摘要 → 用户确认 → 备份 → 临时区 → 原子替换 → 复验 → 成功；任一失败不部分写入并回滚。

### 3.6 扩展管理

#### FZ-22 客户端清单与写入开关

| `client_id` | 用户级 MCP | Skills | 默认写入 | 用户可关闭 |
|---|---|---|---|---|
| `codex` | `~/.codex/config.toml` | `~/.codex/skills/` | true | true |
| `claude` | `~/.claude.json` | `~/.claude/skills/` | true | true |
| `gemini` | `~/.gemini/settings.json` | `~/.gemini/skills/` | true | true |
| `grok` | `~/.grok/user-settings.json` | 项目级 `.agents/skills/` | true | true |
| `opencode` | `~/.config/opencode/opencode.json[c]` | `~/.config/opencode/skills/` | true | true |
| `hermes` | `~/.hermes/config.yaml` | `~/.hermes/skills/` | true | true |

- 首期 `scope=user`；项目级模型保留但 UI 不开放。
- `writable = installed && user_enabled`；客户端未安装时跳过且不建目录。
- 用户关闭某客户端时，不删除已投影内容，只跳过后续写入；UI 提示可手动清理或恢复开关后同步。

> 2026-09-24 补注（用户决策 ②，契约值未变）：**首装没有 `manager-state/extension-config.json` 时，
> 写入命令按默认配置执行**，并在这次写入落盘时把统一配置建出来——不再返回 `NotConfigured`。
> 默认源目录是 `<主目录>/.agents/skills`（`source_store = null`），分发方式是 `symlink`，
> 启用矩阵为六个客户端全开；用户之后可在设置页自行改成自定义源目录 / 统一同步方式。
> 首装第一次点某个客户端图标只会连**被点的那一个**客户端（逐客户端意图以「实际已落地」为基线，此时为空）。

#### FZ-24 Skills 分发

| 项 | 冻结 |
|---|---|
| 首选 | symlink；安全校验通过后创建 |
| 回退 | 复制 |
| symlink 深度 | 40 |
| symlink 检查 | 拒绝逃出源目录、目标目录、数据根、项目根；拒绝 FIFO / 设备 / 目录符号链接循环 |
| 复制 | 保留文件模式，不复制真实值 |
| 冲突 | 目标已存在且哈希不同进待处理 |
| 未安装 | 跳过，不落盘 |

#### FZ-25 / FZ-27 / FZ-29 修订与冲突

| 项 | 冻结 |
|---|---|
| 修订号 | 64-bit 无符号整数，`1` 起步 |
| 位置 | `manager-state/extension-config.json` |
| 变更 | 任意统一配置或启用矩阵变化 +1 |
| 投影指纹 | SHA-256 |
| fingerprint 键 | `fingerprint` |
| fingerprint 对象 | `{"schema_version":1,"fingerprint":"<hex>","updated_at":"<UTC>","source":"opencodex-desktop"}` |
| 比对 | 预期指纹 vs live SHA-256；哈希算法不匹配按损坏处理 |
| 冲突 | `external_modified` / `client_inconsistent` / `half_written` |
| 判定 | 文件级；扩展同步后单条节点 |
| 解决 | `keep_local` / `keep_remote` / `rewrite` / `ignore` |
| 侧字段 | `local_side` / `remote_side`；远端只有客户端 live 内容时也用 `remote_side` |
| 防丢 | 被覆盖侧强制备份，备份失败不执行 |

> 2026-09-24 补注（用户口径，契约值未变）：统一配置新增**逐客户端分发意图** `skill_targets`
> （`Skill 名 → [客户端]`，fingerprint schema 升到 **3**，旧配置缺该键时按「所有启用客户端」读取，
> 不误判 `external_modified`）。**源头 Skill 只读**：`~/.agents/skills`（或自定义源目录）是唯一事实源，
> 客户端目录里只是链接；行内客户端图标 = **只连/断被点的那一个客户端**，
> 写入指令相应带客户端维度：`link_skill{name, client}`、`unlink_skill{name, client}`、
> `write_mcp{name, client}`、`remove_mcp{name, client|null}`（`null` = 删除定义，作用于所有启用客户端）。
> 断开 = 只删管理器建立的软链接：同名真实目录（用户自有内容 / 「文件复制」产物）一律保留，
> 源目录与本 Skill 本体在任何路径下都不被删除。`skills_linked` 取值统一为 `<client>/<name>`。
> 规范见 `docs/02-项目核心/UI规范.md` §1.3 与 `能力地图` `CAP-11.1` / `CAP-11.3`。

> 2026-09-24 补注（缺陷修复，契约值未变）：MCP 的 `write_mcp` **与** `remove_mcp`
> 都必须按**目标客户端自己的配置格式**落盘（Codex = TOML、Claude / Gemini / Grok / OpenCode = JSON、
> Hermes = YAML），一律走 `serialize_target_payload`；不得用 JSON 序列化写回非 JSON 目标。
> 任一客户端配置文件不可解析时，发现与写入会如实报「解析失败」的真因
> （界面提示规范见 `docs/02-项目核心/UI规范.md` §14），不再笼统归为「同名冲突或配置不可写未覆盖」。
> 回归护栏：`src-tauri/tests/extensions.rs::mcp_remove_keeps_toml_target_parseable`。

#### FZ-26 文件锁与串行

| 项 | 冻结 |
|---|---|
| 范围 | 每个 `resolved_config_path` 一把 |
| 锁文件 | `<target>.lock` |
| 实现 | `flock(LOCK_EX)`，超时 3 秒 |
| 作用域 | 覆盖校验 → 备份 → 写入 → 复验 |
| 单实例锁 | `<数据根>/manager-state/app.lock` |
| 获取顺序 | 先 app，再 target |
| 阻断 | 单一状态锁 + 文件锁 |
| 冲突 | 返回码 7 |

#### FZ-28 Skills 来源登记

```json
{
  "source": "local|imported|shared_store",
  "source_uri": "file:///...",
  "source_sha256": "<hex>",
  "verified_at": "<UTC>",
  "size_bytes": 123,
  "file_count": 12,
  "origin_trust": "user",
  "note": null
}
```

- `source_uri` 不放 token。
- 目录导入登记整目录 SHA-256；内容变化即进入冲突。
- 压缩包导入只接受解压后条目数 ≤1,000、总量 ≤128 MiB、单文件 ≤32 MiB，禁止路径穿越。

#### FZ-46 MCP 投影差异白名单

| 改写类 | 允许 | 示例 |
|---|---|---|
| 形状 | `stdio` / `remote` | OpenCode `local` / `remote` 标签联合；Grok 数组容器 |
| 键名 | 稳定映射 | `env` → `environment` |
| 参数 | 参数数组裁剪或重排 | `--stdio` |
| 结构 | 容器转换 | map ↔ array、标签联合 |

限制：

1. 不新增未声明的业务行为。
2. 不注入 shell 字符串。
3. 不改写凭据值。
4. 不删除目标文件其他节点。
5. 每次改写展示 before / after（敏感值掩码）并确认。
6. `args_override` 只能是字符串或 `{"args": ["..."]}`。
7. 未知差异进冲突。

> 2026-09-23 详情展示补充（用户决策，契约值未变）：详情弹窗按本契约回读 `shape` / `command` / `args` 与配置落点，供用户判断执行边界（`env` 只显示键名，`args` / `headers` 掩码）；详情是展示、**不构成写入授权**，展示层**不执行**任何 MCP 命令。方案见 [扩展管理详情方案](../02-项目核心/扩展管理详情方案.md)。**原型已实现并通过原型级验证；生产回读与详情页已于 2026-09-23 实施并通过自动化门禁（桌面端真机验收已于 2026-09-24 完成，证据 `.adg/work/evidence/imp02-accept/`）。**

### 3.7 WebDAV 同步

#### FZ-30 / FZ-45 manifest 与防回放

manifest 明文结构：

```json
{
  "format": "OCXDSYNC",
  "version": 1,
  "compat_version": 1,
  "snapshot_id": "snap_<utc14>_<12位随机 hex>",
  "created_at": "2026-09-14T00:00:00Z",
  "device_name": "<本地设备名>",
  "artifacts": [
    {"name": "manager-state/extension-config.json", "size": 1234, "sha256": "<hex>"}
  ]
}
```

manifest 传输包：

| 字段 | 类型 | 说明 |
|---|---|---|
| `manifest_magic` | string | 固定 `OCXDSYNCPKG` |
| `manifest_format_version` | int | 1 |
| `kdf` / `aead` | object | 与容器同算法，但独立盐和 nonce |
| `aad` | object | `{ "format":"OCXDSYNC", "version":1, "compat_version":1, "device_name":"...", "schema_kind":"sync-manifest" }` |
| `manifest_ciphertext_b64` | string | manifest 明文 AEAD 密文 |

| 规则 | 冻结 |
|---|---|
| 保护 | manifest 单独 AEAD，不作为载荷 AEAD 的 AAD |
| AAD | 固定格式、版本、设备名；清单业务字段进密文 |
| 逐件绑定 | 明文 `artifacts` 的名称、大小、SHA-256 参与认证 |
| 上传 | 逐个加密载荷 → manifest 包最后 |
| 下载 | manifest 包 → AEAD → 比对 AAD → 解密明文 → 逐件校验 |
| 回放 | 旧 `snapshot_id` 或 `created_at` 不晚于本地最后接受时间 → 冲突，不应用 |
| 缺失/乱序 | 缺 manifest、多 payload、载荷名不在 `artifacts`、哈希不一致 → 拒绝 |
| 解密失败 | 整体拒绝，保持本地不变 |
| 备份 | 覆盖前备份被覆盖侧 |
| 冲突 | 默认每次询问 |

#### FZ-31 / FZ-32 载荷与运行

| 项 | 冻结 |
|---|---|
| 载荷命名 | `<remote_path>/<snapshot_id>/payload/<sha256前8>.ocxd` |
| manifest | `<remote_path>/<snapshot_id>/manifest.ocxd` |
| 切分 | 单载荷，不拆分；超过 256 MiB 阻断 |
| manifest | 16 MiB |
| 单载荷 | 256 MiB |
| 快照条目 | 10,000 |
| 快照总量 | 1 GiB |
| 连接 | 15 秒 |
| 单件传输 | 10 分钟 |
| 总时限 | 30 分钟 |
| 重试 | 3 次，退避 2/4/8 秒；认证、回放、解析类失败不重试 |
| 互斥 | 每 endpoint 内存锁 + `<数据根>/sync-state/<endpoint_id>.lock`，`flock`，超时 3 秒 |
| ETag | manifest 上传后记录 ETag 与 SHA-256，仅作变更检测 |
| 断点续传 | 首期不提供 |

#### FZ-41 TLS 文案

| 场景 | 文案 |
|---|---|
| 连接 | “TLS 证书校验失败，已停止连接。请确认服务器证书、系统时间与代理设置。” |
| 下载 | “TLS 证书校验失败，已取消下载；本地内容未修改。” |
| 上传 | “TLS 证书校验失败，已取消上传；远端内容未修改。” |
| 详情 | 展示证书主题、失效时间、错误分类；不显示 URL query、凭据或 cookie |

#### FZ-44 凭据引用

| 项 | 冻结 |
|---|---|
| 存储 | macOS Keychain |
| service_name | `OpenCodex Desktop` |
| account_key | `ocx.dav.<ref_id>` / `ocx.sync.<ref_id>` |
| `purpose` | `webdav_credential` / `encryption_password` |
| `ref_id` | `cred_<uuid>`，小写 |
| 时间 | ISO 8601 UTC |
| 创建 | 后端取值后写钥匙串；不进命令行 |
| 读取 | 后端按需；界面 / 日志 / 通知 / `--json` 只掩码 |
| 更新 | 替换值，不改 `ref_id` |
| 删除端点 | 提示后默认同步删除；可显式保留 |
| 导出/导入/同步 | 只带引用 |
| CLI 口令 | `--password-stdin` |

### 3.8 CLI 与 IPC

#### FZ-34 `ocxd` 命令集

```text
ocxd status
ocxd start --confirm
ocxd stop --confirm
ocxd restart --confirm
ocxd data-root show
ocxd data-root switch --target <path> [--migrate] --confirm
ocxd backup create --confirm
ocxd backup list
ocxd export --output <path> --password-stdin --confirm
ocxd import --input <path> --password-stdin --confirm
ocxd sync run --confirm
ocxd update check
ocxd --help
```

- `data-root switch --target` 接收目录路径；路径是操作目标，不是口令。
- 官方 `ocx update` 仍由用户在 GUI 或官方入口执行；`ocxd update check` 只检查。
- 命令名、子命令、参数是最终契约；未列命令不开放。

#### FZ-35 `--json` 契约

顶层：

```json
{
  "schema_version": 1,
  "command": "status",
  "ok": true,
  "data": {},
  "error": null,
  "generated_at": "<UTC>"
}
```

错误：

```json
{
  "schema_version": 1,
  "command": "start",
  "ok": false,
  "data": null,
  "error": {"code": "require_confirm", "message": "This action requires --confirm."},
  "generated_at": "<UTC>"
}
```

| 规则 | 冻结 |
|---|---|
| 字段 | `snake_case` |
| 版本 | `schema_version` 整数，`1` 起步 |
| 未知字段 | 调用方必须容忍；返回方不新增本表外的顶层字段 |
| 幂等 | 同输入同响应；`start` / `stop` / `restart` 完成后返回当前终态；重复 `create` 不重复备份 |
| 掩码 | 不输出 S3 |
| 时间 | UTC ISO 8601 |
| 状态字段 | `runtime` / `connection` / `operation` / `gate` |

#### FZ-36 IPC 请求与响应

```json
{
  "request_id": "req_<uuid>",
  "command": "status",
  "args": {},
  "confirm": true,
  "contract_version": 1
}
```

```json
{
  "request_id": "req_<uuid>",
  "ok": true,
  "data": {},
  "error": null
}
```

| 规则 | 冻结 |
|---|---|
| 需确认命令 | `start` / `stop` / `restart` / `data-root switch` / `backup create` / `export` / `import` / `sync run` |
| `confirm=false` | 返回 `require_confirm`，退出码 4 |
| 未知命令 | `unknown_command` |
| `contract_version` | 整数 `1`；不匹配即拒绝 |
| 响应 | 不新增顶层字段；`data` 按命令定义 |
| 日志 | 只记 command、参数键、请求 ID、结果码 |

#### FZ-37 socket 访问

| 项 | 冻结 |
|---|---|
| 路径 | `~/Library/Caches/OpenCodex Desktop/ipc/opencodex.ipc` |
| 权限 | 目录 `0700`；socket `0600` |
| 校验 | macOS `getpeereid` 比对 UID；失败拒绝 |
| 启动 | 启用 CLI 后创建；关闭时移除并确认不可连接 |
| 崩溃恢复 | 启动时检查既有 socket 不可用则安全移除；可用则不创建第二端点 |
| 不监听 | 不监听 TCP / 外部地址 |

#### FZ-38 PATH 注册

| 项 | 冻结 |
|---|---|
| 位置 | `/usr/local/bin/ocxd` |
| 形态 | symlink 指向应用内二进制 |
| 默认 | 关闭 |
| 检查 | 目标是普通文件或 symlink；已存在且指向其他目标时不覆盖，提示用户处理 |
| 卸载 | 只删除指向本应用二进制的 symlink；系统目录权限不足给 `sudo` 引导 |
| 审计 | 注册 / 卸载记录 |

#### FZ-39 单写者锁

| 项 | 冻结 |
|---|---|
| 锁文件 | `<数据根>/manager-state/app.lock` |
| 实现 | `flock(LOCK_EX)` |
| 超时 | 3 秒 |
| 行为 | 超时返回码 7 |
| 崩溃 | 进程死亡释放 |
| 内容 | 可写 pid / 启动时间，不做判断依据 |

#### FZ-40 调用审计

| 字段 | 说明 |
|---|---|
| `schema_version` | 1 |
| `event_id` | `evt_<uuid>` |
| `request_id` | 请求 ID |
| `source` | `cli` / `ipc` / `gui` |
| `peer_uid` | 本机 UID |
| `command` | 子命令 |
| `args` | 参数键名，不记录值 |
| `confirm` | bool |
| `result` | `succeeded` / `failed` / `cancelled` |
| `error_code` | 错误码或 null |
| `started_at` / `finished_at` | UTC |

保留 90 天 / 5,000 条；写入 `audit.log`，单文件 5 MB、5 份轮转；导出 / 导入只记动作结果，不记路径和内容摘要。

### 3.9 托管安装

来源：`安全评审输入.md` §12.4（威胁 T-I1 ~ T-I10 / 阻塞项 B-6 ~ B-11）、`DMD` §2.2 / §8 / §10（Revision 9）、`能力地图.md` `CAP-1.4` ~ `CAP-1.7`、`数据与状态.md` §1.1 `runtime` 分区与 §3 `runtime.json`、`UI规范.md` §18。评审已通过（有条件通过；B-6 ~ B-11 转为已确认行为要求）。

#### FZ-47 托管前缀布局与入口

| 项 | 冻结 |
|---|---|
| 默认前缀 | `<数据根>/runtime/opencodex/`——等价一个私有 npm 项目：`package.json` + `package-lock.json` + `node_modules/` |
| 入口 | `<数据根>/runtime/bin/ocx`（稳定路径；运行来源指向它，不指向 `node_modules/.bin`） |
| 安装清单 | `<前缀>/.runtime-manifest.json`：`package`、`version`、`tarball_sha256`、`installed_at`、`npm_path`、`scripts_enabled`、`source`（`registry` / `offline`） |
| 自定义前缀 | 由系统目录选择器或路径文本框给出；必须通过下方全部校验，视为授权动作但**不放宽校验** |
| 校验 | 绝对路径；解析后为**已存在或可创建**的普通目录；**拒绝符号链接**（含任一层父目录解析出的别名）；可写；**不得落在系统保护目录内**；不得与数据根只读分区、客户端 Skills / MCP 落点重叠 |
| 系统保护目录 | macOS：`/`、`/bin`、`/sbin`、`/usr`、`/etc`、`/System`、`/Library`、`/private`、`/var`；Windows：系统盘根、`Windows`、`Program Files*`、`ProgramData`、`Users\Public` |
| 原子性 | 先装到 `<前缀>.new-<随机>`（临时区），校验（入口可执行 + 版本可读 + `name` 匹配）通过后**原子替换**；任何失败清理临时区、**不改动现有前缀与运行来源** |
| 并发 | 目标前缀持跨进程文件锁（复用 `FZ-26` 实现），锁文件 `<前缀>.lock`；检测到外部摘要不符即转冲突待处理（`FZ-29` 口径） |

#### FZ-48 运行来源记录与历史

| 项 | 冻结 |
|---|---|
| 文件 | `<数据根>/manager-state/runtime.json`（分区键 `manager state`） |
| 结构版本 | `schema_version`，固定 `1`；未知版本只读降级、不写回 |
| 当前来源 | `source`：`explicit`（用户显式指定）/ `managed`（托管安装）/ `discovered`（自动发现）/ `unresolved`；附 `path`、`resolved_version`、`resolved_at` |
| 优先级 | 用户显式指定 > 托管安装 > 自动发现候选；**不读取 PATH** |
| 版本归属（2026-09-26 明确） | `resolved_version` 描述**当前来源**的版本：来源路径变化（换到别的路径或变 `unresolved`）时清空；安装终态按落地产物版本回填 |
| 历史条目 | `history[]`：`action`（`install` / `uninstall`）、`result`（`succeeded` / `failed` / `rolled_back`）、`package`、`version`、`target`、`at`、`reason?` |
| 写入时机 | **只在终态**写入；安装进度（百分比、命令行明细、离线段）**不落盘** |
| 禁止写入 | 代理地址、令牌、任一凭据、安装明细原文 |
| 生效语义 | 写盘后由共享解析句柄重新解析；安装 / 卸载完成即切换；**代理进程正在运行时提示「需重启才生效」**，不在运行中替换在用的文件 |
| 保留 | 最近 50 条 history；更早条目按 `FZ-04` 口径清理 |

#### FZ-49 离线包校验与解包加固

| 项 | 冻结 |
|---|---|
| 接受格式 | 仅 `.tgz` / `.tar.gz`（按魔数判断，不只看扩展名）；**不自动二次解包**（包内再含归档不解） |
| 文件名模式 | `opencodex-<semver>.tgz` 或 `bitkyc08-opencodex-<semver>.tgz`；不匹配即拒绝并给出期望模式 |
| 包名白名单 | 包内 `package.json` 的 `name` 必须为 `@bitkyc08/opencodex`，`version` 必须等于文件名中的 semver；任一不符整体拒绝 |
| 完整性 | 计算并展示 tarball SHA-256；随包提供 `.sha256` / `.sha256sum` 时比对，不符即拒绝 |
| 上限 | 条目 20,000 / 解压总量 512 MiB / 单文件 128 MiB / 路径嵌套深度 16；超限即拒绝（不截断） |
| 路径规则 | 逐条规范化后必须落在目标根内；拒绝绝对路径、`..` 越界、**符号链接与硬链接条目**、设备 / FIFO。**任一条不合格整体拒绝**，不做部分展开 |
| 顺序 | 魔数校验 → 文件名与大小预检 → 条目枚举逐条校验 → 解到临时目录 → 元数据校验 → 原子就位；失败清理临时目录 |

#### FZ-50 安装执行策略（脚本 / 代理 / 环境 / 脱敏）

| 项 | 冻结 |
|---|---|
| 包名与版本 | 包名**硬编码** `@bitkyc08/opencodex`，不接受用户改写；版本默认 `latest`，可显式指定具体版本；装后回读 `package.json` 校验 `name` / `version`，不符即失败回滚 |
| npm 来源 | 使用已发现的 npm **绝对路径**（`FZ-13` 门禁结果），不读 PATH；npm 缺失即阻断 |
| 安装脚本 | 默认 `--ignore-scripts`；装入后若入口缺失 / 不可执行，提示「该包需要执行安装脚本」并要求**用户在二次确认框显式开启**后才带脚本重跑一次（高风险提示：会执行包内代码）；用户拒绝则回滚并保持原运行来源 |
| 受控环境 | `cwd` = 目标前缀；`PATH` 只含 node / npm 所在目录与系统最小集；**不注入应用持有的任何凭据环境变量**；不继承 `npm_config_*`；`--prefix` 指向目标前缀 |
| 代理（无凭据） | `--proxy <scheme>://host:port`；`scheme` ∈ `http` / `socks5h`（SOCKS5 一律用 `socks5h://`）；只作用于本次调用 |
| 代理（含凭据） | **凭据不进命令行参数**（argv 可被同机其它进程读取）：改写入临时 `--userconfig` 文件（Unix `0600`，内容只含 `proxy` / `https-proxy` 行），安装结束（含失败与取消）立即删除 |
| 代理持久性 | **不写系统代理、不改全局 npm 配置、不落盘、不进安装记录**；关闭弹窗即失效 |
| 掩码 | 界面与日志中的代理统一显示为 `<scheme>://***@host:port`；用户 `~/.npmrc` **不读取、不解码、不展示**；令牌不作为安装记录的字段 |
| 明细 | 联网安装把 npm 输出逐行就地推送（`UI规范.md` §18.2）；落日志前统一走掩码层，轮转沿用 `FZ-15` |
| 取消与超时 | 安装可取消：终止 npm / 解包子进程 → 清理临时前缀 → 运行来源不变；默认无硬超时，但空闲 120 秒无输出即提示可能卡住 |

#### FZ-51 托管卸载判定（两级）

| 项 | 冻结 |
|---|---|
| L1 范围 | 只删**安装记录登记的托管前缀**（默认 `runtime/opencodex/`；`FZ-47` 允许自定义前缀，此时以记录里的 `target` 为准）与 `runtime/bin/ocx`；不触碰任何配置。判据（2026-09-26 明确）：按历史从最近一条往回取「尚未被成功卸载过的成功安装 `target`」，没有可用记录才回落默认位置；卸载 / 重装同版本与安装弹窗预填共用同一判据 |
| L1 前置校验 | 安装记录存在、`package` 匹配、`target` 与记录一致；目标非符号链接；解析后真实路径等于记录路径；`package.json` 的 `name` 匹配。任一不符 → 拒绝并说明（不误删） |
| L1 备份 | 备份 `runtime.json` 与前缀内 `package.json` / `package-lock.json`（**不备份包体**，可按记录重装同版本） |
| L1 来源回退 | 删前按「显式指定 > 其它自动发现候选」重算来源；无候选则置 `unresolved`；结果写 `history` |
| L2 | 停代理 → 备份 → 二次确认 → **引导用户执行官方 `ocx uninstall`**；应用**不执行该命令、不代删** service / shim / config；仅展示官方输出的（掩码后）结果与退出码 |
| 冲突 | 前缀被外部改写（`package.json` 或摘要与记录不符）→ 拒绝 L1 删除，提示可能已被外部工具接管 |

## 4. 前端工程选型

结论已冻结：原生可完整实现原型视觉，不引入 UI 组件库。

| 项 | 冻结 |
|---|---|
| 视觉基础 | 原型提取的 CSS Design Token，覆盖色彩、圆角、阴影、间距、字体栈与明暗主题 |
| 布局与组件 | CSS Grid / Flex + 自建小组件层；不引入 Element Plus、Ant Design 等组件库 |
| 兼容基线 | 当前 Tauri 平台 WebView 支持 `color-mix()`、`:has()`、`backdrop-filter`、CSS Grid、`aspect-ratio` |
| 跨端策略 | 统一 Token 与组件 CSS；平台差异只处理滚动条、字体回退与 WebView 默认样式 |
| 工程栈 | TypeScript + Vite；跨组件状态用 Pinia |
| 覆盖范围 | 不含 Tauri 原生标题栏、托盘和系统菜单，这些保留在桌面壳与 Rust 侧 |

## 5. 建议项处置

| 项 | 处置 |
|---|---|
| S-1 | 已在项目核心冻结：容器 `0600`、不写符号链接目标 |
| S-2 | §3.5 / §3.7 上限已冻结 |
| S-3 | §3.5 风险口径已冻结 |
| S-4 | §3.1 备份保留已冻结 |
| S-5 | §3.6 来源登记已冻结 |
| S-6 | §3.8 审计与 `--json` 已冻结 |
| S-7 | §3.6 fingerprint 已冻结 |

## 6. 平台目标矩阵

本节记录平台支持口径与实施边界，作为 `WP-01` 工程底座和 `WP-22` 打包签名的约束。

| 目标 | 类别 | 构建方式 | 验证要求 | 发布口径 |
|---|---|---|---|---|
| `aarch64-apple-darwin` | 正式支持 | 本机 / CI | 功能、验收与打包冒烟全覆盖 | 可承诺 |
| `x86_64-apple-darwin` | 计划支持 | CI 交叉构建；必要时出 universal 包 | 自动化测试 + Apple Silicon Rosetta 启动冒烟；真实 Intel Mac 冒烟待补 | 未完成 Intel 真机验证前不承诺完全支持 |
| `x86_64-pc-windows-msvc` | 计划支持 | Windows CI runner | 编译、单元 / 集成测试、CLI 与核心文件路径自动化冒烟 | 自动化通过后可承诺基础支持；GUI 完整冒烟待真实环境确认 |
| `aarch64-pc-windows-msvc` | 实验支持 | 可选 CI 实验 | 不作为验收目标；不承诺原生兼容 | 本轮不生成；如后续产出只标注 experimental，不进正式通道 |

平台差异必须隔离在基建层，不得散在业务模块里：

| 能力 | macOS | Windows 计划支持 | 边界 |
|---|---|---|---|
| 凭据存储 | Keychain | Windows Credential Manager / DPAPI | `CredentialRef` 语义不变，后端按平台选择实现 |
| 文件权限 | Unix `0600` / `0700` | ACL | 语义等价于「当前用户最小权限」；不做 1:1 权限位映射 |
| IPC | Unix domain socket + `getpeereid` | Named pipe + 等价调用方校验 | 对上保持同一 `IpcEndpoint` 契约 |
| 文件锁 | `flock` | `LockFileEx` | 锁超时、返回码和串行化语义一致 |
| 进程控制 | Unix 进程等待与信号 | Windows Job Object / 终止语义 | 禁止 SIGKILL 等价语义；保留取消与等待规则 |
| 打包签名 | macOS 签名与公证 | Authenticode + 安装器 | 各平台签名失败都终止安装 |

实施规则：

1. 本轮正式验证目标只有 `aarch64-apple-darwin`。
2. `x86_64-apple-darwin` 与 `x86_64-pc-windows-msvc` 可以在 CI 生成和测试，但未完成对应验证前不得写入正式支持。
3. `aarch64-pc-windows-msvc` 本轮不生成构建产物。
4. 平台差异测试要记录平台、构建方式、验证范围和未覆盖项。

## 7. 验收环境与 mock 数据

- macOS 首期；不连接真实 CLI / 文件系统 / WebDAV / 钥匙串。
- 所有 fixture 在仓库内使用临时路径，不引用真实家目录内容。
- 覆盖三维状态矩阵、前置缺失、WebDAV 连接与同步、加密导出容器、冲突、通知。
- 用场景驱动替代真实外设；同一 fixture 可重建。
- 回归必须覆盖 AC-01 ~ AC-13 的正常 / 失败 / 取消 / 恢复路径。

## 8. 派生引用回写

本节只记录本轮同步的派生产物，不作为事实源。2026-09-14 用户确认 IMP 后，已按首批范围创建五个执行任务：

| TASK | 范围 | 当前状态 |
|---|---|---|
| `TASK-OPENCODEX-DESKTOP-01` | 工程底座（WP-01） | `in_progress` |
| `TASK-OPENCODEX-DESKTOP-02` | 测试基线最小集（WP-21A） | `in_progress` |
| `TASK-OPENCODEX-DESKTOP-03` | 构建基线（WP-22A） | `in_progress` |
| `TASK-OPENCODEX-DESKTOP-04` | 写入原语（WP-17A） | `in_progress` |
| `TASK-OPENCODEX-DESKTOP-05` | 实例约束基础（WP-18A） | `in_progress` |

| 历史派生对象 | 调整 |
|---|---|
| `IMP输入清单.md` | 开头指向本 IMP，后续稳定契约以本文件为实施权威 |
| `docs/README.md` | 实施行改为「IMP 已创建，`planned`」 |
| 协作包 `02-开发实施/README.md` | 前置说明改为 `FZ-01` ~ `FZ-46` |
| 协作包确认与评审 | 下一步改为「确认 IMP → 拆 TASK」 |

## 9. 变更记录

| 日期 | 变更 |
|---|---|
| 2026-09-28 | DMD Revision 12 状态收敛方向回写：§3.2 增兼容基线与目标语义边界。随后按同方向完成**现行原型收敛与宽/窄窗口验证**（见 IMP-04 §19.31）；**仅改文档、原型与原型测试，未修改字段、生产代码或通过生产验收**。详细方案与静态发现见状态专题。 |
| 2026-09-14 | 创建 IMP，冻结 46 项技术契约。 |
| 2026-09-14 | 补充前端工程选型：原生 CSS Token + TypeScript + Vite + Pinia，不引入 UI 组件库。 |
| 2026-09-14 | 项目核心同步 IMP 对齐；`R-17` 生产面板承载方式保持发布前门禁，本轮不指定。 |
| 2026-09-14 | 补充平台目标矩阵：macOS arm64 正式支持；macOS x86_64 与 Windows x64 计划支持；Windows ARM64 本轮不生成。 |
| 2026-09-14 | 用户确认 IMP 与首批 C-01 ~ C-05；创建并授权 `TASK-OPENCODEX-DESKTOP-01` ~ `-05`，进入工程初始化。 |
| 2026-09-15 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-31`：通知中心接入后端共享通知模型、IPC 契约、前端真实状态操作与安全/视觉/测试/构建证据。 |
| 2026-09-15 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-32`：Doctor 接入受控官方只读命令，统一超时、截断、脱敏与失败保留，并修复顶栏重复与运行态地址显示。 |
| 2026-09-15 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-33`：扩展管理接入 Skills/MCP 受控只读发现，统一 DTO、脱敏、失败保留与原型视觉，写入边界仍未启用。 |
| 2026-09-15 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-34`：接入原生托盘动态状态、菜单门控与动作请求桥，运行地址仅在运行态展示。 |
| 2026-09-15 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-35`：关于页接入 Tauri 应用契约与本机受控 `ocx --version` 只读发现，失败保留上次结果，补齐明暗主题与响应式视觉。 |
| 2026-09-16 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-36`：设置页偏好接入 `manager-state/preferences.json` 真实读写，冻结默认值、白名单校验与原子替换，保持 0600 权限与失败保留。 |
| 2026-09-16 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-37`：数据根与 `OPENCODEX_HOME` 接入真实运行时配置，补齐目标校验、安全阻断、0600 原子写、重启生效与前端契约测试；修复顶栏重复并限定运行态地址显示。 |
| 2026-09-16 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-40`：WebDAV 同步内核接入 manifest 独立 AEAD、AAD 绑定、逐件校验、防回放、互斥锁、覆盖前备份与冲突登记；真实网络客户端保持后续边界。 |
| 2026-09-16 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-41`：应用自更新接入 stable/beta 通道与签名校验；修正手动检查 IPC 与失败原因优先级，失败保留当前版本。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-42`：`ocxd` CLI 接入 Unix socket IPC 真实端点，覆盖冻结命令集、确认标志、机读输出、权限与 peer UID 校验、崩溃恢复、脱敏审计与可选 PATH symlink；bundle 内 `ocxd status --json` Release 实机验证通过。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-43`：WebDAV 同步接入受控 reqwest 网络客户端、PROPFIND/GET/PUT 边界、macOS Keychain 凭据引用、端点配置 0600 原子写、连接测试、TLS 证书校验拒绝与状态投影；真实快照上传下载与冲突收口保持下一任务。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-44`：WebDAV 同步接入真实快照上传/下载组合执行、本地加密、远端逐件校验、覆盖前备份、冲突登记、状态投影与 CLI/IPC 脱敏摘要；未配置端点显式提示且不产生网络写入。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-45`：扩展统一配置接入跨客户端真实写入，覆盖 Skills preferred 源软链接与 MCP 四类配置机械改写；补齐路径边界校验、冲突确认、备份、0600 原子写与删除可恢复，扩展页真实刷新发现结果。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-46`：扩展资产操作接入 Skills ZIP 导入、卸载目录备份与最新备份恢复，MCP 新增、改名编辑与删除按启用客户端真实写入；补齐 ZIP 路径穿越拒绝、执行边界确认与前后端契约测试。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-47`：设置页升级区接入官方版本发现、升级前配置备份与失败恢复建议；只展示并复制 `ocx update`，不执行官方更新，也不接管 npm 包管理器。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-48`：状态采集接入真实官方输出、受控 HOME/PATH 环境与官方 schema 映射；前端直接快照初始化并同步运行状态，Release 实机验证通过且不再停在加载中。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-49`：修正 macOS 默认 Node/npm 发现路径为 `~/.local/bin`，绑定默认路径契约测试；Release 实机验证 Node.js/npm/ocx 三项发现、安装形态 npm global 与版本 2.50.0。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-50`：设置页路径复制、目录/配置打开、Skills/MCP 真实路径、立即同步、日志与通知清理、关于页外链与内置授权/第三方文档入口全部接入真实契约；显式路径经后端白名单投影，未配置端点显式提示且失败保留。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-51`：托盘页“退出桌面壳”按钮接入 Tauri process exit；前端契约测试覆盖退出码 0 与失败提示，失败时保留当前状态，Release 前端构建与视觉证据通过。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-52`：桌面管理器自更新接入 Tauri updater 下载安装，签名校验通过后安装并重启，失败保留当前版本且不停止托管代理。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-53`：托盘补齐打开数据目录与快速设置，所有导航动作经请求桥路由；macOS 原生菜单注入进程、视图与日志入口，Release 实机菜单与前端构建验证通过。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-54`：restore 风险链接入官方 startup 状态与配置存在性摘要，用户确认后生成受控备份；仅展示并复制 `ocx restore`，桌面壳不代理、不包装、不接管官方 restore。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-55`：概览运行详情直接打开当前数据根与 OPENCODEX_HOME；后端白名单精确放行两个共享根，缺失或失败保留当前窗口。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-56`：面板页基于运行期端口接入官方 Web 面板 iframe 承载；非运行/不健康显式阻断，失败支持重试与受控本机浏览器回退。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-57`：采集 aarch64 Release 启动到窗口 687ms、运行态 RSS 113.3MiB 与 app bundle 7.0MiB 候选基准，记录签名公证与 R-17 限制；不执行公开发布。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-58`：复核 aarch64 Release 候选制品、测试、性能与限制清单，写入 `REL-OPENCODEX-DESKTOP-01`；生产签名/公证与公开发布保持单独门禁。 |
| 2026-09-17 | 治理复核补齐 `TASK-25` ~ `TASK-30` 的 TASK/RCP/CERT 原子收口闭环；实施记录、验证证据与授权范围不变，RCP 历史摘要按当前权威口径登记。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-59`：加密迁移容器接入扩展 Skills 名称与启用矩阵、WebDAV 端点 Keychain 引用；兼容既有凭据对象形态，修正设置页过时文案与重复通知 IPC 注册；Release 实机导出与设置页视觉验证通过。 |
| 2026-09-17 | 滚动授权并完成 `TASK-OPENCODEX-DESKTOP-60`：`run_sync_now` 接入远端 latest 指针、manifest/载荷下载、AEAD 解密验证、防回放冲突登记、覆盖前备份、0600 原子写与 Skills 真实链接恢复；冷同步显式阻断。 |
| 2026-09-19 | 就地修订 `FZ-43`（`TASK-OPENCODEX-DESKTOP-61`）：级别与渠道分离，补齐六类渠道的出现时机、触达、去重、排队、关闭、超时、清理与跨路由约束；视觉参数移交 `docs/02-项目核心/UI规范.md`。 |
| 2026-09-19 | 就地修订 `FZ-43.4`（`TASK-OPENCODEX-DESKTOP-64`）：收敛面板路由通知层承载方案为**「协调显示」**——通知表面可见时临时让出面板子视图、消失后恢复；通知宿主仍为窗口级壳层 DOM，不复制面板页标记、不留透明点击层。同步 `docs/02-项目核心/UI规范.md` §4.7。 |
| 2026-09-19 | 就地修订 `FZ-23`（用户决策）：源目录由「**恒为**标准目录、不提供自定义」改为「**默认为** `<主目录>/.agents/skills`，**允许改为自定义目录**（只读，且需通过路径校验）」；原「不引入任意路径输入」的安全立场随之调整为「引入但受校验约束」——新增「自定义源目录校验」行，参数标记待安全评审确认。同步回改 `docs/02-项目核心/领域模型.md` §10 `source_store`。现状差异：正式应用仍按前一版（固定标准目录）实现，原型已带自定义选择器；**代码侧对齐待单独授权**（该「待授权」已在同日下一条落地）。 |
| 2026-09-19 | 落地 `FZ-23` / `FZ-24` 的实现：`ExtensionConfig` 新增 `source_store`（`null` = 默认 `<主目录>/.agents/skills`，可为自定义目录）与 `sync_method`（用户可选项）；新增 `source_dir` 模块承担源目录解析与 R-23 校验（绝对路径 / 存在可读 / 非符号链接 / 不与数据根·写入区·客户端 Skills 目录重叠）；发现、链接解析、设置页路径投影全部改为读配置；写入路径按 `sync_method` 决定建软链接或写副本，软链接失败自动回退复制（`FZ-24`）；新增 `set_source_dir` / `set_sync_method` / `resync_skills` 与只读的 `list_extension_directories`；`ProjectionFingerprint` schema 升到 2 并支持旧配置迁移（不误判 `external_modified`）；移除偏好项 `skills_sync_mode`（改由统一配置承载）。门禁：`cargo fmt` / `clippy -D warnings` / `cargo test` 全通过，前端 `vue-tsc` / `vitest` / `vite build` 全通过。 |
| 2026-09-19 | 就地修订 `FZ-23`（用户决策）：新增「选择方式」行——目录选择一律走**操作系统原生目录选择器**，管理器不自造目录浏览器。据此移除自建选择器：删除 `list_extension_directories` 命令及其 DTO、`source_dir` 的 `list_subdirectories` / `picker_start_dir`、前端 `SourcePicker.vue`；改为引入 `tauri-plugin-dialog`（`dialog:allow-open` 最小权限）并经 `@tauri-apps/plugin-dialog` 的 `open({directory:true})` 取路径。路径仍按「自定义源目录校验」行当作不可信输入校验，R-23 结论不变。 |
| 2026-09-19 | 统一 `FZ-23` 措辞（用户决策）：源目录的**解释性定语**由「Agent Skills / cc-switch 生态标准目录」改为「Agent Skills 共享目录」，界面文案统一为「默认目录；也可以自定义目录」；用户可见文案不再出现参考实现（cc-switch）名称。**契约值一字未动**（`~/.agents/skills`、默认可覆盖、只读、路径校验均不变），故不构成契约变更。cc-switch 仅作为实现参考基线保留在 `安全评审输入.md` §15 等内部材料中。 |
| 2026-09-20 | 修复 `FZ-24` / `FZ-41` 相关的 **WebDAV 连接不可用**（契约符合性修复，契约值未变）：`reqwest` 此前以 `default-features = false` 声明且未补 TLS feature，编译产物**没有任何 TLS 后端**，`https://` 请求在 scheme 阶段即被拒绝（`invalid URL, scheme is not http`），而 `WebDavConfig::validate` 又强制要求 `https://`——两者互斥导致 WebDAV 从未真正连通过。现补 `rustls-tls-native-roots`（系统根证书），并新增 `tests/webdav_tls.rs` 两条护栏（编译期 `use_rustls_tls` 可用性 + `https` 必须走到传输层），缺 TLS 时门禁直接失败。 |
| 2026-09-20 | 修复 `FZ-44` **凭据分位被违反**导致的 WebDAV 认证失败：保存端点时 `password` 与 `encryption_password` 都经 `store_webdav_password` 写入同一个 `ocx.dav.<ref_id>` 账户位，后者**覆盖**前者——WebDAV Basic 认证实际使用的是加密口令，而 `load_encryption_password` 读取的 `ocx.sync.<ref_id>` 从未被写入。现补 `store_encryption_password` / `delete_encryption_password`，保存按 `purpose` 分位写入，删除端点时两个账户位一并清理；新增真机往返护栏（默认 `#[ignore]`，手动执行）。**迁移提示**：此前保存过的端点，其钥匙串两个账户位内容是错的，修复后需**重新保存一次端点**。 |
| 2026-09-20 | 修复桌面壳**应用菜单缺少「编辑」子菜单**：`build_app_menu` 只构造进程 / 视图 / 日志并整体替换默认菜单，而 macOS 的 ⌘C / ⌘V / ⌘X / ⌘A 由应用菜单 key equivalent 经 responder chain 转发给 WKWebView，缺失即导致**所有输入框无法用键盘复制粘贴**。现加「编辑」子菜单（undo / redo / cut / copy / paste / select_all，走系统预定义项，不产生菜单事件、不进入冻结托盘动作域）。 |
| 2026-09-23 | 就地补充 `FZ-23` / `FZ-46` 的**详情展示**说明（用户决策，契约值未变）：扩展列表条目「名称 + 描述」可打开**只读详情**（完整描述 + `SKILL.md` Markdown 渲染、源目录与体量；MCP 类型 / 命令 / 参数 / 落点），**掩码规则不变**；单条「检查 / 更新」只作用于该条目、批量检查为独立入口。同步新增 `REQ-28`（需求基线）、`AC-11` 扩展条款（DMD Revision 6）与 `CAP-11.5`（能力地图）；方案见 `docs/02-项目核心/扩展管理详情方案.md`。**原型已实现并通过原型级验证（jsdom 130 / 无头 Chromium 98，均 0 失败）；生产读取器、目录统计、MCP 回读与详情页尚未实施、未验收。** |
| 2026-09-23 | 落地日志两分类生产实现（用户决策）：日志页新增「应用日志 / 调用日志」分类与「面板请求日志」入口；日志与 doctor 输出改纯文本渲染。**修复 `FZ-14` 相关的真实缺陷**——审计写入器写在活跃数据根根目录的 `audit.log`，读取器却按 `<数据根>/logs/` 拼接，导致「调用日志」永远为空；现按活跃数据根解析，并在分类缺失时不回退应用日志（`agent.log` 除外）。门禁：`cargo fmt` / `clippy -D warnings` / `cargo test`（416 通过 / 0 失败 / 3 忽略）、`vue-tsc` / `vitest`（41 文件 176 例）/ `vite build` 全通过。**桌面端真机验收已于 2026-09-24 完成，证据 `.adg/work/evidence/imp02-accept/`。** |
| 2026-09-23 | 落地 `CAP-11.5` 扩展条目详情生产实现（用户决策）：新增 `read_skill_detail`（单条按需读取 `SKILL.md`，超 256 KB 截断；目录统计上限 2000 条，越界如实报「未知」）与 `read_mcp_detail`（回读命令 / 参数 / 落点，`env` 只给键名、`args` / `headers` 掩码）；前端新增详情弹窗（Markdown 先转义再渲染、客户端状态格可切换并写回列表、单层滚动条）。同步落地扩展页真实分页与客户端筛选 chip（原为静态「第 1 / 1 页」与不可点 chip），并修原型搜索 / 分类只改计数不隐藏行的缺陷。**桌面端真机验收已于 2026-09-24 完成，证据 `.adg/work/evidence/imp02-accept/`。** |
| 2026-09-23 | 收敛 5 项无法兑现为行为的设置（用户决策，见 `quality-remediation/05-打包验收与交付.md`）：`backup_integrity` 固定 SHA-256（`FZ-21` 只记 `sha256`）、`mcp_conflict_policy` 固定「每次询问」、`launch_with_codex` 改为只读展示（官方 shim 域，管理器不写入官方配置）、`backup_include_skills` / `backup_include_mcp` 明确「不含」。界面不再提供假开关；扩备份契约与同名冲突策略保留为待办。 |
| 2026-09-23 | 定义并落地「首次落地页」与「关闭窗口后的 Dock 行为」（用户决策）：① 无显式入口时按 `auto_panel` 决定落地页（开启→面板，关闭→概览），判定前移到应用挂载之前以消除「先概览再跳面板」的闪烁，规则只保留一处实现；② 窗口隐藏（关闭窗口或启动即不显示主窗口）时 macOS 激活策略切到 `Accessory`——**Dock 图标与菜单栏消失、托盘保留**，从托盘再打开窗口时切回 `Regular`。规范见 `docs/02-项目核心/UI规范.md` §1.3（关联 `FZ-10` 生命周期）。门禁：Rust 418 通过 / 0 失败 / 3 忽略；前端 42 文件 183 例通过。**Dock 与菜单栏的真实外观已于 2026-09-24 真机验收**：窗口隐藏后 `lsappinfo` 由 `Foreground` 变 `UIElement`（Dock 图标消失）、托盘状态项保留，托盘「打开主界面」后回到 `Foreground`。 |
| 2026-09-24 | **桌面端真机验收（隔离 HOME + 打包制品，契约值未变）**：按 `codex/implementation-base` 的 `b99d3148` 制品在 `HOME` 覆盖的隔离环境逐项复核，`lsappinfo` / 无障碍树 / OCR 取证（证据 `.adg/work/evidence/imp02-accept/`）。结论：① 首次落地页两分支均按 `auto_panel` 生效（默认落面板、关闭落概览）；② 窗口隐藏（关闭窗口或 `launch_main=false` 启动）后激活策略为 `UIElement`——Dock 图标消失、托盘状态项保留，托盘「打开主界面」回到 `Foreground` 且窗口恢复；③ 扩展详情弹窗结构按原型（标题动作为图标按钮、描述只在正文面板、信息区标签定宽对齐、`SKILL.md` frontmatter 已剥离、MCP 环境变量独占一行、`env` 掩码、更新时间只给日期、目标格可点击且状态可读）；④ 设置「扩展管理」区路径独占一行、无「测试同步」、`选择自定义目录` 打开的是 macOS 原生目录选择器（非跳转拓展页）。**发现 1 项首装缺陷（未修）**：`extension-config.json` 不存在时，UI 的逐条同步（`link_skill`）与 MCP 写入（`write_mcp`）返回 `NotConfigured`，前端一律提示「Skill 同步失败；目标冲突或文件不可写未覆盖。」；同输入的直调后端在配置存在后成功，已确认根因为缺失配置下的写入门控。复现与根因见 `.adg/work/IMP-OPENCODEX-DESKTOP-02.checkpoint.md`。 |
| 2026-09-24 | **首次落地页改为「按面板当前状态」判定（用户澄清，契约值未变）**：原规则按 `auto_panel` 偏好决定落地页，用户澄清「启动面板时进面板、没启动就还是概览」，即判定输入应是**面板是否已启动**。现规则：无显式入口时，面板已启动（`runtime === running`、`port` 非空、`health !== unhealthy`，与面板内嵌判定同一口径）→ 面板；否则 → 概览；显式入口（深链 hash / 托盘动作）仍不覆盖。`auto_panel` 不再有消费方，设置页「启动后自动打开面板」开关收敛为只读事实「首次落地页 · 随面板状态」（字段仍保留在 `preferences.json` 契约中）；同步改原型同一行。落地页仍在**挂载前**决定：启动入口先取偏好，再等第一份「运行时已定档」的快照（`loading`/`starting`/`stopping` 视为未定档，上限 1500 ms，取不到按概览落地）。门禁：`vue-tsc` / `vitest`（42 文件 192 例，0 失败）/ `vite build` / `tauri build` 全通过；原型 QA 135 / 69 / 100 均 0 失败。**真机验收（重编制品 + 隔离 HOME，证据 `.adg/work/evidence/imp02-landing/`）**：面板运行中（假 `ocx` 报 running + 本地 7317 桩）→ 首屏即面板并成功嵌入桩页；无 `ocx`（未启动）→ 首屏概览；设置页为只读事实。 |
| 2026-09-24 | **修复「点一个客户端图标却全部平台亮起」（用户报告 + 用户口径，契约值未变）**：根因是写入指令**没有客户端维度**——`link_skill{name}` 只把 Skill 记进统一清单，投影 `apply_skill_links` 再按「启用矩阵」（默认六个客户端全 true）写全部客户端，于是点 Claude 会连出六个链接；且关闭分支误用 `uninstall_skill`（整条卸载）而不是断链。按用户口径修正语义：**源头 Skill 只读，客户端目录里只是链接，图标开关 = 只连/断被点的那一个客户端**。实现：`ExtensionConfig` 新增 `skill_targets`（fingerprint schema 升到 3，旧配置缺键时按「所有启用客户端」读取，首次逐项开关以**实际已落地**为基线，避免误伤）；写入指令改为 `link_skill{name, client}` / `unlink_skill{name, client, confirm}` / `write_mcp{name, client, confirm}` / `remove_mcp{name, client\|null, confirm}`；投影改为**逐客户端对账**（要的就连、不要的就断），断开只删管理器建立的软链接，同名真实目录与源目录一律不动；`skills_linked` 取值统一为 `<client>/<name>`（原先带 JSON 引号）。前端行内图标与详情弹窗目标格统一走逐客户端连/断，行内「卸载」仍为整条卸载（独立动作）。门禁：`cargo fmt` / `clippy -D warnings` / `cargo test --workspace`（**420 通过 / 0 失败 / 3 忽略**，含新增逐客户端与旧配置兼容用例）、`vue-tsc` / `vitest`（42 文件 195 例）/ `vite build` / `tauri build` 全通过。**真机验收（重编制品 + 隔离 HOME，证据 `.adg/work/evidence/imp02-perclient/`）**：点 Claude → 只有 `.claude/skills/design-studio` 落地、其余五个客户端无变化、本源不动、`skill_targets={'design-studio':['claude']}`；再点 Claude → toast「已断开 … 到 Claude 的链接」且只断这一个；再点 Codex → 只连 Codex、Claude 不被连带。 |
| 2026-09-24 | **修复「MCP 多端点击会失败；MCP 写入失败；同名冲突或配置不可写未覆盖」（用户报告，隐性数据破坏，契约值未变）**：根因是 `projection.rs` 的 `RemoveMcp` 分支写回时用 `serde_json::to_vec(&payload)` **未按目标格式序列化**——对 Codex（TOML）会把 `~/.codex/config.toml` 整篇写成 JSON，该文件从此不再是合法 TOML；由于 `source_server_definition` 遍历各客户端配置、任一不可解析即返回 `Corrupted`，**该客户端之后的所有 MCP 读 / 写 / 断开（含点其它客户端）全部失败**，界面只给出「同名冲突或配置不可写未覆盖」，与真因无关。修复：① `RemoveMcp` 改用既有的 `serialize_target_payload(target, &payload)`，按目标格式（TOML / JSON / YAML）落盘（新增回归用例 `mcp_remove_keeps_toml_target_parseable`，已验证去掉修复即失败）；② 新增 `src-ui/src/extensionErrors.ts`（`describeExtensionFailure` / `extensionFailureText`），把后端 `{ code, message }` 归一化为中文原因并附原始描述，扩展页失败 toast 改为「动作：真实原因（原始描述）」，不再一律说「同名冲突或配置不可写未覆盖」（规范见 `docs/02-项目核心/UI规范.md` §14，用例 `src-ui/tests/extension-errors.test.ts`）。门禁：`cargo fmt` / `clippy -D warnings` / `cargo test --workspace`（**421 通过 / 0 失败 / 3 忽略**）、`vue-tsc` / `vitest`（**43 文件 200 例**）/ `tauri build` 全通过。**真机验收（旧 / 新制品对照 + 隔离 HOME，证据 `.adg/work/evidence/imp02-mcp/`）**：旧制品点 Codex 断开 → `.codex/config.toml` 变 JSON（首字符 `{`）、再点 Gemini 出「扩展发现失败」、再点 Codex 报「MCP 写入失败；同名冲突或配置不可写未覆盖。」（与上报逐字一致）；新制品同一流程 → `.codex/config.toml` 仍是合法 TOML（`tomllib` 解析通过、`node_repl` 已移除、同文件 `existing` 保留）、其它客户端无连带、失败提示变为「MCP 写入失败：目标配置或统一配置解析失败…（原始报文）」，修好被外部改坏的文件后可自行恢复。 |
| 2026-09-24 | **首装写入门控按用户决策 ② 放开（用户决策，契约值未变）**：此前若 `manager-state/extension-config.json` 不存在，`LinkSkill` / `WriteMcp` / `RemoveMcp` / `ImportSkillArchive` / `RestoreSkill` / `AddMcp` / `EditMcp` / `ResyncSkills` 一律返回 `NotConfigured`，**全新安装第一次点图标同步或写 MCP 必然失败**（界面提示「Skill 同步失败；…」，磁盘无任何落地）。用户决策 ②：**缺配置时按默认配置执行，并在本次写入落盘时把统一配置建出来**。实现：`projection.rs::execute` 去掉该门控，文件不存在时改用 `ExtensionConfig::default()`（源目录 `source_store = null`，即默认 `<主目录>/.agents/skills`，用户之后可在设置页改为自定义目录；`sync_method = symlink`；启用矩阵六端全开）；首装第一次点选的逐客户端意图以「实际已落地」（此时为空）为基线，因此**只连被点的那一个客户端**。前端 `extensionErrors` 中 `not configured yet` 的中文原因改为「扩展配置不可用；数据目录无效或不可访问。」（该分支现在只在数据目录无效时命中）。门禁：`cargo fmt` / `clippy -D warnings` / `cargo test --workspace`（**423 通过 / 0 失败 / 3 忽略**，新增 `first_write_without_projection_config_uses_defaults_and_creates_the_config`、`first_mcp_write_without_projection_config_creates_the_config`，已验证把门控加回即失败）、`vue-tsc` / `vitest`（43 文件 200 例）/ `tauri build` 全通过。**真机验收（旧 / 新制品对照 + 隔离 HOME，证据 `.adg/work/evidence/imp02-firstrun/`）**：旧制品全新 HOME 点 `design-studio → Claude` → 「Skill 同步失败：扩展配置尚未建立…（application is not configured yet）」、无落地；新制品同流程 → 「已同步 design-studio 到 Claude。」、只建 `.claude/skills/design-studio` 软链接、其余五端未创建、统一配置建出且为默认值；新制品另一个全新 HOME 首次写 MCP → 「已写入 node_repl 到 Gemini。」并建出配置；设置页「Skills 源目录 / 选择自定义目录」入口与默认路径展示正常。 |
| 2026-09-24 | **托盘不再承载风险结论与状态说明句（用户报告，契约值未变）**：托盘菜单头部此前把状态说明句拼进去（`{运行标签} · {说明句}`），`at_risk` 变成「存在风险 · 代理未在运行，启动保护未开启，重启后不会自动恢复，官方版本偏差 2.50.0；可点『启动 OpenCodex』直接启动或查看建议。」——**原生菜单项被长文案撑宽**，而托盘只是进程控制入口。按用户口径修正：**托盘只讲进程事实**。实现：① `modules/tray::runtime_label` 中 `AtRisk` → **「未运行」**（「存在风险」这个结论不写进托盘）；② 删除 `TrayState.notice`、`status_notice` / `at_risk_notice`，托盘头部固定为 `运行标签 · 地址`，tooltip 同短式；③ `TrayDto` 去掉 `notice` 字段并同步前端 `commands/tray.ts`；④ 新增前端同源映射 `labels.ts::trayRuntimeLabel`（`at_risk` → 未运行），托盘预览页改用它，菜单栏 / 任务栏的状态文字按原型继续 `display:none`（不占位置）。风险结论与说明句仍在应用内展示（概览状态说明区 / 通知中心 / 日志），托盘入口动作（运行 Doctor / 打开日志 / 打开主界面）把人带回应用。门禁：`cargo fmt` / `clippy -D warnings` / `cargo test --workspace`（423 通过 / 0 失败 / 3 忽略）、`vue-tsc` / `vitest`（43 文件 201 例）、`tauri build` 全通过；原型新增 `qa/tray.test.mjs`（jsdom，15 项通过 / 0 失败 / 0 JS 错误）。**真机验收（旧 / 新制品对照 + 假 at-risk `ocx` + 隔离 HOME，证据 `.adg/work/evidence/imp02-tray/`）**：修复前托盘菜单头部逐字复现用户截图那段长文案；修复后头部只剩「未运行」，托盘内不含「存在风险」与任何说明句，动作项齐全；同实例应用内概览仍显示「存在风险」+ 建议说明 + 启动 / 查看建议 / 刷新三个动作。规格：`docs/02-项目核心/UI规范.md` §15、`docs/02-项目核心/数据与状态.md`（2026-09-24 修订）。 |
| 2026-09-24 | **「运行环境诊断」改为诊断中心第一个 Tab（用户要求，契约值未变）**：用户要求「做成一个 tab，tab 标题四个字，放到第一个 tab」。此前 Doctor 是日志页下方一张独立卡片（与日志历史 / 通知历史两张卡并列），现改为第一层 Tab：**环境诊断｜日志历史｜通知历史**，三个 Tab 共处**一张内容卡**，不再有独立 Doctor 卡片。实现：`LogsRoute.vue` 顶部 Tab 行新增 `环境诊断` 并置于第一位、doctor 内容改为该卡内的一个面板（`v-if="diagnosticsTab === 'doctor'"`），删除原第二张卡；`stores/routes.ts` 的 `diagnosticsTab` 由二值扩为 `doctor / logs / notifications` 并新增 `normalizeDiagnosticsTab`（非法 / 缺失回落到**日志历史**）；`useAppController.ts` 的托盘 `run_doctor` 与 `TrayRoute.vue` 的托盘预览项改为先 `go('logs', { tab: 'doctor' })` 再运行，避免结果不可见。**默认落地页仍是日志历史**（环境诊断未运行前只有空态，不适合当默认）。门禁：`vue-tsc` 通过、`vitest`（43 文件 203 例，新增 2 例断言 Tab 顺序 / 四字标题 / 同卡 / 归一化）；原型 `qa/log-categories.test.mjs`（jsdom）**91 项通过 / 0 失败 / 0 JS 错误**（新增 15 项）、`skill-detail` 135 / `tray` 16、浏览器脚本 25 项全绿；`tauri build` 通过。规格：`docs/02-项目核心/日志分类展示方案.md` §3 / §7、`UI规范.md` §11.1。 |
| 2026-09-24 | **诊断中心命名统一：导航短名「诊断」/ 全名「诊断中心」（用户同意，契约值未变）**：用户问「这里要什么名字更合适」并同意分析结论。此前同一处有四个名字：左侧导航 `日志`、页面标题 `诊断中心`、概览 / 设置入口 `打开诊断中心`、托盘与 macOS 应用菜单 `打开日志`——而内容已含环境诊断 + 日志历史 + 通知历史，「日志」既不准（只占 1/3）又与内部 Tab「日志历史」撞名。落地：① 导航短名 `日志` → **`诊断`**（保持 2 字，与概览 / 面板 / 拓展 / 设置 同宽，`title` 同步）；② 托盘托盘菜单入口项与 macOS 应用菜单「诊断」子菜单里的同项 → **`打开诊断中心`**（全名，与概览 / 设置入口一致；菜单 id 未改）；③ 术语表新增「诊断 / 诊断中心」短名-全名两行并补齐「诊断中心」释义（原写「运行环境诊断 + 通知历史」，漏了日志历史）；④ `UI规范.md` 新增 §11.2 命名规则，§11.1 与数据与状态、日志分类展示方案同步。**不跟着改**：设置里的分区名「日志与通知」与清理动作（讲的是日志 / 通知的保留策略）、概览动作 chip「查看日志」（落点就是日志历史）、通知场景表里表示「日志型记录」承载面的「日志」、路由 id `logs` 与深链 `#logs?tab=…`（改名只涉及可见文案，不影响既有入口与书签）。门禁：`vue-tsc` 通过、`vitest` 43 文件 203 例；原型 jsdom `log-categories` **91**（新增 6 项命名断言）/ `tray` **16**（新增 1 项）/ `skill-detail` 135、浏览器脚本 25 全绿；`tauri build` 通过。 |
| 2026-09-24 | **表格列宽分配与短内容不换行（用户报告，契约值未变）**：设置 → 安装配置 →「数据目录与 OPENCODEX_HOME」表格里「打开」按钮被拆成两行——根因是表格没有列宽策略，长文本列（路径）与短列（用途 / 操作）按内容平均分宽，窗口变窄时短列宽度被压到只剩一个字的宽度。修复（软件与原型同一套口径，规范见 `docs/02-项目核心/UI规范.md` §16）：① 长文本列 `width: 100%`（吃掉剩余宽度、可 `min-width` 兜底），短列 `width: 1%; white-space: nowrap`——**列宽不够时先截断路径，不是把「打开」拆成两行**；② 全局底线 `.btn { white-space: nowrap; }`（`src-ui/src/styles/base.css` 与原型 `index.html` 同款），`.icon-btn` / `.tag` / `.state-pill` / `.cli-badge` 等短控件同样 `nowrap`；③ 数据目录两张表（`#card-install-paths` / `#card-install-partitions` 的 `data-root-table`）第 1 列路径 `width: 100%; min-width: 180px`、第 2 列用途与末列操作 `width: 1%; white-space: nowrap`，操作列右对齐；原型体检表末列（结果 / 状态）同款。**原型新增生成型探针 `05-原型/原型/qa/tables.browser.mjs`**（无头 Chromium）：逐张表断言「不横向溢出 / 表头单行 / 按钮与状态控件单行且高度 ≤34px / 短内容列单行」，覆盖设置两张表（默认窗口 + 最小窗口 900×600）、CLI 说明弹窗两张表、调用日志表、同步体检弹窗表，截图输出 `文档/截图/原型-表格列宽-20260924/`；环境缺失以退出码 2 报 BLOCKED。门禁：`cargo fmt` / `clippy -D warnings` / `cargo test --workspace`（**423 通过 / 0 失败 / 3 忽略**）、`vue-tsc` 通过、`vitest`（43 文件 203 例）；原型 jsdom `log-categories` 91 / `tray` 16 / `skill-detail` 135、浏览器 `tables` **154 项 0 失败 0 报错**、`log-categories` 25 / `skill-detail` 100 全绿；`tauri build` 通过。**真机验收（重编制品 + 真实 HOME，窗口置 960×1050，证据 `.adg/work/evidence/imp02-tables/`）**：`数据目录与 OPENCODEX_HOME` 表在应用最小宽度 960px 下「打开」按钮为 `42×29`（单行），路径列 `372px` 吃掉剩余宽度、用途列 `20~170px`、按钮右缘 1112 < 窗口右缘 1160 无溢出；截图的 Vision OCR 把「打开」识别为单个 11~12px 高的文本块（换行时会裂成两块）。规格：`docs/02-项目核心/UI规范.md` §16。 |
| 2026-09-24 | **卡片内按钮行补上间距（用户报告，契约值未变）**：设置 → 安装配置里「使用外部路径 / 回到数据目录内」「校验并初始化 / 仅切换引用 / 迁移数据」直接贴在输入框下沿（实测 −1px）。根因是卡片内每种堆叠块都有 `margin-top:14px`（`.field-grid` / `.setting-list` / `.cli-preview` / `.doctor-out`），**只有 `.controls` 没有**；原型侧 3 处卡片级按钮行是逐处 inline `style=\"margin-top:14px\"` 补的，软件侧一处都没补。修复：两侧各加系统规则 `.card > .controls { margin-top: 14px; }`，并删掉原型 3 处 inline 补丁（`.metric` 内的 8px 是另一种嵌套，保留）。改动仅 2 行 CSS / 3 处 inline 清理。门禁：`vue-tsc` 通过、`vitest` 43 文件 203 例；原型新增探针 `qa/spacing.browser.mjs`（8 条路由逐卡断言上间距 ≥12px）**11 项通过 / 0 失败 / 0 报错**（3 处候选均 14px），`tables` 154 / jsdom 91 / 16 / 135 全绿；`tauri build` 通过。**真机验收（重编制品 + 真实 HOME，窗口 1181×1173，证据 `.adg/work/evidence/imp02-spacing/`）**：两处输入框 → 按钮行 −1px → **13px**；另三处设置列表 → 按钮行 ≈2px → **16px**，按钮 38px 单行、无横向溢出。规格：`docs/02-项目核心/UI规范.md` §17。 |
| 2026-09-24 | **修复「窗口不能拖拽」（用户报告，契约值未变）**：macOS 用 `titleBarStyle: Overlay`，没有原生标题栏可拖，窗口移动必须由 `.titlebar` 条带承担；而拖拽要三件套齐，当时三条全缺——① `App.vue` 的头部没有 `data-tauri-drag-region`；② `.titlebar { pointer-events: none }` 让元素不可命中（属性即使加上也无效）；③ `capabilities/default.json` 只给了 `core:default`，而 **`core:window:default` 不含 `allow-start-dragging`**，注入脚本的 `invoke('plugin:window|start_dragging')` 被 Tauri v2 ACL 静默拒绝。另外 `.titlebar` 只占图标列（244px），即使能拖也只有窗口左上角一小块。修复：加属性、`pointer-events: auto`、capability 补 `core:window:allow-start-dragging`、条带改 `grid-column: 1 / -1` 铺满窗宽。门禁：`vue-tsc` 通过、`vitest` **44 文件 207 例**（新增 `window-drag.test.ts` 4 条：属性存在 / 不可 pointer-events:none / 授权齐 / 高度口径）；原型 `spacing` 11 / jsdom 91 / 16 / 135 全绿；`tauri build` 通过。**真机验收（合成 HID 事件 + 无障碍树位置，证据 `.adg/work/evidence/imp02-drag/`）**：修复前拖顶栏纹丝不动；只补 DOM+CSS 后 AX 命中测试已能落到 `AXGroup[窗口控制区]` 但仍不动 ⇒ 定位到 ACL；补授权后左半顶栏 `200,120 → 80,180`、右半顶栏 `200,120 → 100,180`（位移与拖拽量一致）、双击顶栏缩放 `→ 0,30 3321×1174` 再双击复原；导航点击未被条带遮挡（同一工具拖 Finder 正常，排除工具因素）。规格：`docs/02-项目核心/UI规范.md` §1.4。 |
| 2026-09-25 | **原型确认：托管安装/运行来源 + 概览说明句分流（用户确认，进入实现规划）**：用户确认原型后要求「检查文档回写 → 把没有开发的内容开发完」，并先出计划。本轮**只补计划与文档**：① 文档回写补齐 `DMD Revision 8`（安装进度可见、联网可选 HTTP/SOCKS5 临时代理、概览状态卡不承载说明句、§8 托管安装 P1 行修正为「默认数据根内、可显式修改」）、`能力地图 CAP-1.4`（代理 + 进度）、`数据与状态`（进度瞬态不持久化、代理不落盘）、`UI规范` §19 与 `产品定义 PR-06`；② 发现**未决门禁**——`01-需求分析/04-需求分析产物/安全评审输入.md` 没有「托管安装（联网安装 + 离线包导入）」域，而 `DMD §8 / §10` 要求该能力**实施前必须完成安全评审**，故托管安装实现被该评审阻塞；③ 概览说明句分流不在安全评审范围内，可独立开发。实施计划见本文件 §「待实施计划」。**本轮未新增提交、未改动代码。** |
| 2026-09-25 | **落地 Track A：概览状态说明句分流（用户确认原型后开发）**：概览状态卡删除 `.ovb-note`（`OverviewRoute.vue` / `base.css`），`runtimeScenarios` 去掉 `note` 字段并移除 `statusScenario` 里的运行时来源拼接（该信息仍由「运行详情」展示，无信息丢失）；说明句按性质分流——**稳定事实**（`stopped` / `running`）在状态迁移时播报**一次** info Toast（`stores/app.ts::announceRuntimeTransition`；首份已定档快照只用于起锚、启停流程不重复播报），**需关注**（`not_found` / `starting_failed` / `unreachable` / `at_risk` / `external_takeover`）由**后端**在状态轮询观测到变化时发布持久通知（新增 `commands::runtime_state_notification` + `publish_runtime_state_notification`；`run-start-failed` 与启停失败共用去重键 `run:start-failed`；「启停结果通知」偏好只约束启停结果类，环境与配置所有权类不受影响），**瞬时进度**不写；「启动后首次观测」与「状态未变化」都不发布。**顺带修复 3 个真机暴露的缺陷**：① 后端写入通知后前端不刷新（新增 `notifications-changed` 事件 + 前端监听与排队重拉）；② 通知动作取值 kebab-case ↔ snake_case 不一致——后端冻结为 `settings_installation`，前端按 `settings-installation` 查表，多词动作的通知详情只剩「关闭」，前端统一 snake_case 并抽出 `notificationActions.ts`；③ 通知时间直出原始 RFC 3339，新增 `formatNotificationTime` 按本地 `HH:mm` 展示。门禁：`cargo fmt --check` / `clippy -D warnings` / `cargo test --workspace --features integration-test`（**430 通过 / 0 失败**，新增 7 条）、`vue-tsc` / `vitest`（**48 文件 219 例**，新增 12 例）、`tauri build` 全通过。**真机验收（隔离 HOME + 只读假 `ocx`，证据 `.adg/work/evidence/imp02-overview-status/`）**：概览无说明句；启动不播报也不写通知；未运行 → 运行中 出一次 info Toast（约 1.9s）且不写通知；切 `at_risk` / `not_found` 各写一条未读未解决通知且无 Toast；往返不堆叠并把 `read` 恢复为 false；通知面板在打开状态下随后端写入从「全部 1」变为「全部 2 · 系统 1」；详情「前往设置」落到设置 → 安装配置、「前往处理」进 restore 引导。规格：`UI规范.md` §19 / §15、`数据与状态.md` 2026-09-25 修订。 |
| 2026-09-25 | **冻结托管安装契约：`FZ-47` ~ `FZ-51`（用户确认后补登，FZ 总数 46 → 51）**：B0 安全评审通过后按 `安全评审输入.md` §12.4「待冻结契约」逐项落地。新增本文件 **§3.9 托管安装**：`FZ-47` 前缀布局与入口（默认 `<数据根>/runtime/opencodex/`、入口 `runtime/bin/ocx`、`.runtime-manifest.json`、自定义前缀五项校验 + 系统保护目录清单、临时区 + 原子替换、前缀级跨进程锁）、`FZ-48` 运行来源记录与历史（`manager-state/runtime.json`：来源四态、优先级、`history[]`、只在终态写、不写代理与凭据、生效与「需重启」语义、保留 50 条）、`FZ-49` 离线包校验与解包加固（魔数判型、文件名模式、包名与版本白名单、SHA-256、条目 / 体积 / 深度上限、逐条路径规范化且拒绝符号链接与硬链接、整体拒绝不做部分展开）、`FZ-50` 安装执行策略（包名硬编码 + 装后回读校验、npm 用绝对路径、默认 `--ignore-scripts` + 缺 bin 时二次确认重跑、受控环境、**含凭据代理改写临时 `--userconfig` 0600 使凭据不进 argv**、代理不落盘、掩码与明细规则、取消语义）、`FZ-51` 托管卸载判定（L1 只删登记前缀 + 前置校验 + 备份 + 来源回退，L2 只引导官方 `ocx uninstall`、应用不代删、冲突拒绝）。同步更新 §1 / §2 计数与索引、Track B 计划（B0 标记已通过并引用 `FZ-47` ~ `FZ-51`）；回写 `IMP输入清单.md` §3.10 / §4、`契约字段.md` §6、`领域模型.md` §18 / §19、`数据与状态.md` §1.1。**本轮未改动代码；B1 及之后仍未开始。** |
| 2026-09-25 | **落地 B1：运行来源解析层（`FZ-48`）**。新增 `src-tauri/src/modules/runtime/`：`RuntimeSourceKind`（`explicit` / `managed` / `discovered` / `unresolved`）、`RuntimeSourceRecord` + `RuntimeStore`（`manager-state/runtime.json`，原子写 `0600`，缺失 / 损坏 / 未知 schema 一律降级为未解析且不写回）、`RuntimeResolver`（优先级 用户显式 > 托管 `<数据根>/runtime/bin/ocx` > 自动发现候选；**不读 PATH**；显式路径失效时静默回落而不是钉死在未解析）、`RuntimeHandle`（`Arc` 共享句柄：`initialize` 冷启动即落盘、`refresh` 重解析、`set_explicit`、`record_resolved_version`、`record_history`，历史上限 50）。新增 `paths` 模块并把**读 / 写两套校验分开**：`validate_executable`（跟随符号链接、不限系统目录——Homebrew 与 npm 全局前缀下的 `ocx` 常常是链接，按写入口径判会把用户正当安装误伤）、`validate_install_target`（`FZ-47` 写路径：绝对路径、拒符号链接、拒系统保护目录、可写；符号链接检查限定在 `HOME` 范围内，否则 macOS 的 `/var` → `private/var` 会把所有临时区路径判成逃逸）。新增 infrastructure `runtime_executable::RuntimeExecutableProvider` trait（+ `FixedRuntimeExecutable` fixture），`RuntimeHandle` 实现之；`OfficialStatusSource` / `OfficialDoctorSource` / `OfficialCliVersionSource` 与 `state::ProcessContext` 全部改为**每次调用现取**当前来源，安装 / 卸载 `refresh` 一次即对所有消费者生效。`lib.rs` 用 `RuntimeHandle::initialize` 替换一次性 `discovery_paths.ocx`（发现候选 = 受控默认目录 + `/opt/homebrew/bin/ocx` + `/usr/local/bin/ocx`，逐个校验、缺失即跳过）。门禁：`cargo fmt --check` / `clippy -D warnings` 通过；`cargo test --workspace --features integration-test` **448 通过 / 0 失败**（较上轮 430 增加 18 条，全部在 `modules::runtime`）。**B2 及之后仍未开始。** |
| 2026-09-26 | **落地 Track B · B2–B5 并完成 B6 真实链路验收（真机 GUI 部分未通过）**：新增 `modules/runtime/archive.rs`（`FZ-49` 离线包校验与解包加固）、`modules/runtime/install.rs`（`FZ-47` / `FZ-50` 安装内核：写路径校验 + 前缀锁 + 临时区 → 回读校验 → `.runtime-manifest.json` → 原子替换 → 稳定入口；受控 npm；含凭据代理只写临时 `--userconfig` 0600；`VersionProbe` 以「真的跑一次入口读版本」落实 `FZ-47` 的「版本可读」）、`modules/runtime/uninstall.rs`（`FZ-51` 两级卸载）、`commands/runtime.rs` + `types/runtime.rs`（B3 进度事件流与命令面）、前端 B5（`commands/runtime.ts`、`runtimePresentation.ts`、设置页运行来源卡片、安装向导四步、两级卸载弹窗、`EnvironmentGate` 未安装引导改造去掉 PATH 假设）。**真实链路验收**用真实 npm + 真实网络 + 真实官方包（`tests/runtime_managed_real.rs`，`OCX_TEST_RUNTIME_REAL=1`）跑通联网安装 / 入口打印 `opencodex 2.66.0` / 来源切 `managed` / 一级卸载 / 离线导入 / 拒绝类 / 临时区零残渣，热缓存与冷缓存均通过；并据此修掉三个真实缺陷（稳定入口曾指向临时区、离线导入缺依赖、冷缓存 `ENOTEMPTY`）与两条按事实修正的结论（官方包在 `--ignore-scripts` 下即可运行；官方 tarball 不含 `node_modules`）。门禁：`cargo fmt --check` / `clippy -D warnings` / `cargo test --workspace --features integration-test`（**502 通过 / 0 失败**）、前端 `vue-tsc` / `vite build` / `vitest`（**51 文件 239 通过**）、`tauri build` 通过。**B6 的真机 GUI 部分未通过**（GUI 点完四步后是否真正落地并切换来源、卸载弹窗两级、来源切换后启停与状态采集跟随均未验证），证据与复现步骤见 `.adg/work/evidence/imp02-runtime/`；另记录一条测量结论：本环境 exec 沙箱的 `/tmp` 视图与 GUI 进程不一致，不能用 `/tmp` 下的可见性判定安装失败。 |
| 2026-09-26 | **完成 Track B · B6 打包制品真机 GUI 验收，并修掉真机暴露的两个缺陷**。真机 GUI 路径（隔离 `HOME` = `~/.ocx-accept/gui9-013123`，`HOME` 内 `~/.local/bin/{node,npm}` 指向本机 Node）走通：①四步向导联网安装 → `安装完成 · 2.66.0`，`runtime.json` = `managed`、`resolved_version=2.66.0`、稳定入口落地；②来源切换后启动 / 停止与状态采集跟随（`运行中 端口 10100 版本 2.66.0`，进程为托管前缀内 `bun.exe`）；③一级卸载 → 来源回退 `unresolved`、前缀与入口删除、备份保留、`resolved_version` 清空；④二级卸载 → 停代理 + 备份 + 逐字 `uninstall` 二次确认 + 只生成官方 `ocx uninstall` 引导、**不代删**。据此修两个**真实缺陷**：(a) **前端运行来源读取缺「在途去重」**——安装完成后端广播 `runtime-source-changed`（事件监听先发起一次读取），安装流程紧接着自己也读一次；旧实现里后来者因 `runtimeSourceLoading` 直接返回 `false`，于是拿着切换前的旧来源把一次成功安装误报成「来源没有切换到托管安装」（真机上磁盘已是 `managed`、界面却报失败）。现改为并发调用共享同一份在途结果，并加回归用例（在旧实现上必失败）。(b) **`RuntimeHandle::record_resolved_version` 从未被回填**——`FZ-48` 的 `resolved_version` 一直是 `null`，卡片版本恒显「未知」；现安装终态回填版本，并在 `persist` 里于**来源路径变化**时清空旧版本（卸载后不会把上一来源的版本挂到 `unresolved` 上），补单元用例 `resolved_version_follows_the_current_source_path`。门禁：`cargo fmt --check` / `clippy -D warnings` 通过；`cargo test --workspace --features integration-test` **503 通过 / 0 失败**；前端 `vue-tsc` / `vite build` / `vitest` **241 通过**；`tauri build --target aarch64-apple-darwin` 通过（重新产出的 `.app` / `.dmg` 即真机验收所用制品）。真实链路（`OCX_TEST_RUNTIME_REAL=1`）热缓存与冷缓存均通过。证据与 OCR 文本归档 `.adg/work/evidence/imp02-runtime/`（`B6-真机2-*`）。 |
| 2026-09-26 | **B6 之外的收尾：门禁口径收口 + 契约回写 + 真机补充覆盖**。①**概览环境门禁不再误报**（用户授权新增口径，已回写 `UI规范` §18.5）：`missing_ocx` 只说明自动发现候选为空，托管安装（或指定 / 发现来源）已解析时门禁改为 `ready_via_source`——OpenCodex 一行显示「运行来源 · 通过」、不出现安装型动作、不再阻断设置页安装区；`missing_node` / `missing_npm` 仍按原门禁阻断。②**契约回写**：`FZ-48` 增补「版本归属」行（`resolved_version` 描述当前来源，来源路径变化即清空，安装终态回填）；`FZ-51` 的 L1 范围明确「按安装记录登记的 `target` 定位，且跳过已被成功卸载过的前缀」；`契约字段.md` §6.1 同步。③**真机补充覆盖**（隔离 `HOME` = `~/.ocx-accept/gui10-075023`，`cliclick` + Vision OCR 驱动）：自定义前缀（系统保护目录「已拒绝」；原生目录选择器选定合法自定义目录并落点「数据根外」；自定义前缀安装后**稳定入口仍在数据根**、包体在自定义目录、`history.target` 记录自定义前缀；L1 卸载按记录删对目录）；离线包导入（真实 `npm pack` tarball，预览 SHA-256、离线无命令行、`安装完成 · 2.66.0 · 离线导入`）；更换运行来源（用户指定 · 数据根外）/ 恢复自动发现；含凭据代理（界面掩码 `http://***@host:port`、临时 `--userconfig` `0600` 且用完即删、口令不进 argv 有单测护栏）；取消安装（终止 npm、清临时区、来源与版本不变、history 记 `install/failed/安装已取消`）；更新（官方 `ocx update`）**只展示不执行**。④据此再修两个真机缺陷：(a) **卸载 / 重装的前缀取自安装记录**（旧实现固定用数据根默认前缀，自定义前缀安装后卸载弹窗指向错误目录、卸载后安装弹窗又预填回已删除目录；`RuntimeSourceDto` 现区分 `managedPrefix`＝登记前缀与 `defaultPrefix`＝默认落点，「恢复默认」用后者）；(b) **版本事实回填**（`official_project_facts` 读成功后回填 `resolved_version`，来源变化后前端「读版本 → 重读来源」，卡片版本不再恒显「未知」）。门禁：`cargo fmt --check` / `clippy -D warnings` 通过；`cargo test --workspace --features integration-test` **505 通过 / 0 失败**；前端 `vue-tsc` / `vite build` / `vitest` **51 文件 246 通过**；`tauri build --target aarch64-apple-darwin` 通过。证据：`.adg/work/evidence/imp02-runtime/`（`B6-真机3-*`）。 |
| 2026-09-26 | **修掉「官方版本事实在安装 / 卸载终态后不刷新」**（真机观察到的残留）：版本升级卡与关于页读的是 `officialProject`，它只在进入页面或手动「检查更新」时重读，于是托管安装完成后该卡仍显示装之前的「未发现」（真机：装完 2.66.0，升级卡仍是未发现）。现 `installRuntime` 成功收口与 `uninstallRuntime` 收口后各补一次 `loadOfficialProject()`——安装后按新来源回填版本，卸载后读不到时按 `FZ-48` 保留上次结果并标记错误，不再把已移除来源的版本当作当前版本。真机复验（隔离 `HOME` = `~/.ocx-accept2/gui11-*`，打包制品四步向导联网安装）：装前升级卡「当前版本 未发现」→ 装后「当前版本 v2.66.0」。门禁：前端 `vue-tsc` / `vitest` **51 文件 246 通过** / `vite build` 通过；`cargo fmt --check` / `clippy -D warnings` / `cargo test --workspace --features integration-test` **505 通过 / 0 失败**；`tauri build --target aarch64-apple-darwin` 通过。证据：`.adg/work/evidence/imp02-runtime/`（`B6-真机4-*`）。 |
| 2026-09-26 | **治理收口：补建本次交付的 TASK / RCP / CERT，并同步已漂移的治理对象**。此前 Track A 与 Track B 只写了 IMP 变更记录、检查点与证据，`.adg` 里没有对应的治理对象。本轮用 Skill 自带的确定性内核（`kernel.py`）建对象、跑校验、跑投影：`TASK-OPENCODEX-DESKTOP-68`（Track A：概览说明句分流与关注态持久通知，绑定 `AC-03` / `AC-05`）、`TASK-69`（Track B：托管安装与运行来源 B0–B6，绑定 `AC-01` / `AC-08`）、`TASK-70`（安装 / 卸载终态后刷新官方版本事实，绑定 `AC-01`），各自配 `RCP` / `CERT` 并以 `succeeded` 关闭；五项 `review:*` 必过闸门都附具体证据（含真机证据目录）。**同时发现并修复对象漂移**：`project-check` 报 DMD 与 IMP-01 的 `source_digest` 漂移（DMD 对象仍带 Revision 7 之前的 AC-01 旧文——「不执行 npm 安装、卸载或依赖管理」——而其 Markdown 权威早已按用户决定改为「可装进自有数据根内私有前缀、不做全局 npm 前缀管理」），本轮用 Skill 的 `project-refresh` 按 Markdown front-matter 重建 DMD / IMP 投影，并把 `IMP-OPENCODEX-DESKTOP-01` 的 `task_ids` 补齐为 01–70（原先只到 58）。收口后 `project-check` = `valid`（索引 fresh、看板 valid、source binding 无漂移）。**对象内 before/after 摘要口径**：文件字节 `sha256`，`before` 取该任务基线 commit 快照、`after` 取收口时工作区（历史对象由已退役的 Skill 生成，口径不同，不追溯改写）。**仍未收口**：`TASK-OPENCODEX-DESKTOP-67`（IMP-02）仍是 `in_progress`，且 DMD 对象的 `implementation_ids` 为空、IMP-01 的 `summary` 文案仍停在 TASK-37 时代——属既存漂移，未在本轮一并改写。 |

## 待实施计划：托管安装与运行来源 + 概览说明句分流（2026-09-25）

来源：`DMD Revision 7 / 8`、`能力地图.md` `CAP-1.4` ~ `CAP-1.7`、`UI规范.md` §18 / §19、`数据与状态.md` 两处同日修订。原型已按上述契约实现并通过原型级验证。

### 缺口盘点（原型有、代码没有）

| 能力 | 现状 | 结论 |
|---|---|---|
| 运行来源解析 | `lib.rs` 启动时一次性把 `~/.local/bin/ocx` 拷进 status / doctor / version / IPC / `ProcessContext`；无 `runtime.json`；不读 PATH 但也没有「用户指定 > 托管 > 自动发现」优先级 | **未开发** |
| 托管安装（联网 / 离线） | 无；设置页仍只展示 `npm install -g @bitkyc08/opencodex` 作为引导命令 | **未开发** |
| 安装进度 + 代理 | 无 | **未开发** |
| 卸载（两级）/ 更新引导 | 无（只有通用升级前备份与官方 update 引导） | **未开发** |
| 概览状态卡说明句 | `OverviewRoute.vue:89` 仍渲染 `{{ scenario.note }}` | **未开发** |
| 关注态持久通知 | 后端只为启停失败发通知（`commands/mod.rs::publish_run_failure`），`at_risk` / `external_takeover` / `unreachable` / `not_found` 无来源 | **未开发** |

### Track A —— 概览说明句分流（**已完成**，2026-09-25，见上方同名修订行）

| 步 | 内容 | 落点 |
|---|---|---|
| A1 | 去掉状态卡说明句：删 `.ovb-note` 节点与样式、`runtimeScenarios[…]` 不再作为渲染文案 | `src-ui/src/routes/OverviewRoute.vue`、`src-ui/src/styles/base.css`、`src-ui/src/data/mock.ts` |
| A2 | 稳定事实播报：`runtimeState` 迁移到 `stopped` / `running` 时给**一次 info Toast**；启停 / 重启已有结果 Toast 时不再叠加 | `src-ui/src/composables/useAppController.ts` |
| A3 | 关注态持久通知：为 `not_found` / `starting_failed` / `at_risk` / `external_takeover` / `unreachable` 发布通知（level + dedupe + action，未读未解决；同问题只更新一条） | 新 `src-tauri/src/modules/runtime/…` 或 `modules/status` 收敛处 + 复用 `NotificationPublisher` |
| A4 | 门禁 + 真机验收：`cargo fmt/clippy/test`、`vue-tsc` / `vitest`、`tauri build`；真机核对概览卡无说明句、Toast 一次、通知去重 | `.adg/work/evidence/` |

风险：状态抖动会刷通知 → 用 `dedupe_key` + **仅在状态真正变化时发布**，并把「应用刚启动的首次收敛」排除在外。

### Track B —— 托管安装与运行来源

| 阶段 | 内容 | 门禁 |
|---|---|---|
| **B0 安全评审（前置闸门）** | **已完成（2026-09-25）**：`安全评审输入.md` §12.4 新增「托管安装」域（威胁 T-I1 ~ T-I10：供应链与 registry 劫持、安装生命周期脚本执行、凭据经命令行与日志泄漏、任意路径写入、tgz 解包穿越与链接逃逸 / 解包炸弹、代理被滥用为全局出口、前缀与运行来源不一致、卸载越界删除、并发写入者、失败残留），§16.2 阻塞项 B-6 ~ B-11，§16.6 处理结果；`DMD` 已回写 **Revision 9**（门禁解除）、`集成与安全边界.md` 已补「已确认控制」六条 | ~~必须先行~~ **已通过** |
| B1 运行来源解析层 | **已完成（2026-09-25）**：新增 `modules/runtime`（`RuntimeSourceKind` 四态 + `RuntimeSourceRecord` / `RuntimeStore` / `RuntimeResolver` / `RuntimeHandle` + `paths` 两类校验）；`lib.rs` 一次性 `discovery_paths.ocx` 改为 `RuntimeHandle` 共享句柄；status / doctor / version / 进程动作 / IPC 全部改为**每次调用现取**（新增 infrastructure `RuntimeExecutableProvider` trait，句柄实现它）。冷启动即把首次解析结果写 `manager-state/runtime.json` | ~~B0 通过~~ **已完成** |
| B2 托管安装 | **已完成（2026-09-25）**：`modules/runtime/archive.rs`（`FZ-49` 离线包加固）+ `modules/runtime/install.rs`（写路径校验 + `<前缀>.lock` + 临时区 → 回读校验 → 清单 → 原子替换 → 稳定入口；受控 npm；含凭据代理走临时 `--userconfig`） | ~~B0 通过~~ **已完成** |
| B3 进度事件流 | **已完成（2026-09-25）**：`commands/runtime.rs` 经 `runtime-install-progress` 推送 `{phase, percent, line?}`；`runtime-source-changed` 广播；取消标志终止 npm 并清理临时区；**终态才写历史** | ~~B2~~ **已完成** |
| B4 卸载两级 / 更新引导 | **已完成（2026-09-25）**：`modules/runtime/uninstall.rs` + 命令层。L1 四道前置校验 + 备份（不备份包体）+ 删前缀与入口 + 来源回退；L2 停代理 + 备份 + 逐字 `uninstall` 二次确认 + 返回官方命令与对象清单（**不执行、不代删**）+ 只读结果观测 | ~~B2~~ **已完成** |
| B5 前端 | **已完成（2026-09-25）**：`commands/runtime.ts`、`runtimePresentation.ts`、`stores/app.ts` 运行来源与进度订阅、设置页运行来源卡片、安装弹窗（步骤条 1~4 + 代理 + 进度）、卸载弹窗（两级 + 清单 + 二次确认）、`EnvironmentGate` 未安装引导改造（去掉 PATH 假设） | ~~B1~B4~~ **已完成** |
| B6 验收 | **已完成（2026-09-26）**：①**真实链路**全部通过（联网安装 / 一级卸载往返 / 离线导入 / 系统保护目录拒绝 / 坏包拒绝且不改来源 / 外部改写拒绝 / 临时区零残渣；热缓存与冷缓存两种条件）；②**打包制品真机 GUI 验收**（隔离 `HOME` = `~/.ocx-accept/gui9-013123`）：四步向导联网安装真实落地并切换来源（`安装完成 · 2.66.0`、`runtime.json` = `managed` + `resolved_version=2.66.0`）、来源切换后启动 / 停止与状态采集跟随（`运行中 端口 10100 版本 2.66.0`，进程为托管前缀内 `bun.exe`）、一级卸载（来源回退 `unresolved`、前缀与入口删除、备份保留、`resolved_version` 清空）、二级卸载（停代理 + 备份 + 逐字 `uninstall` 二次确认 + 只引导、不代删）。本轮并据此修掉两个真机缺陷（前端「在途去重」缺失导致成功安装被误报为未切换来源；`resolved_version` 从未回填导致卡片版本显示「未知」）。证据归档 `.adg/work/evidence/imp02-runtime/` | ~~B1~B5~~ **已完成** |

**契约已冻结（2026-09-25，本文件 §3.9）**：`FZ-47` 前缀布局与入口、`FZ-48` 运行来源记录与历史、`FZ-49` 离线包校验与解包加固、`FZ-50` 安装执行策略（脚本 / 代理 / 环境 / 脱敏）、`FZ-51` 托管卸载判定。B1 ~ B6 均以这 5 条为实施基线，不再另立口径。

> 与安全评审的一处**收紧**：评审只要求「代理凭据掩码后进界面与日志」，而 DMD §10 明确凭据**不得进入命令行参数**，因此 `FZ-50` 进一步冻结为「含凭据的代理写临时 `--userconfig`（`0600`，用完即删），凭据不进 argv」。

### 执行顺序与提交粒度

1. ~~Track A（A1~A4）一笔提交 + 真机验收。~~ **已完成（2026-09-25，未提交）**，证据 `.adg/work/evidence/imp02-overview-status/`。
2. B0 安全评审单独一笔（文档）。
3. B1~B2 一笔（解析层 + 安装内核），B3~B4 一笔（进度 + 卸载/更新），B5 一笔（前端），各自门禁通过后再进下一笔。
4. B6 真机验收证据归档到 `.adg/work/evidence/imp02-runtime/`。

### 明确不做的

不写系统目录与全局 npm 前缀；不替代官方 `ocx update` 事务；不自行删除官方 service / shim / config；不持久化代理与任何凭据；不在安全评审未完成时开始 B1 之后的工作。
| 2026-09-27 | **内容页表面材质定为「半透明磨砂」，落地 `UI规范` §21（用户确认档位）**：概览页批注「页面能否做成半透明背景」经原型四档打样（实底 / 半透明磨砂 / 中透 / 更透）后，用户选定**「中透」**。口径：**软件底层（窗口底面 / 菜单 / 标题栏）保持不透明**，透明的是**内容页**这一层；页面自己的颜色盖在「窗口底面 + 环境光」上，透上来的是**软件自己的底**，与桌面、壁纸无关（原型里一度加的「垫壁纸」开关是错误道具，已删除）。落地：`tokens.css` 新增 `--page-fill`（亮 `rgba(255,255,255,.40)` / 暗 `rgba(38,38,38,.40)`）；`base.css` 的 `.main` 用它，外壳零改动；卡片 / 按钮 / 表格仍是实底，页面只在留白处透出底色。**真机实测挖出的坑**：第一版给 `.main` 加了 `backdrop-filter`，把窗口在两个位置间移动时外壳取色不变、**页面颜色跟着变** —— WKWebView 里它会去采样窗口背后的桌面，那就不是「软件自己的底」了；去掉后同样两点取色逐位相同；页面透出的柔和感由 `.app-window::before` 那层已 `filter: blur()` 的环境光提供。规范：`docs/02-项目核心/UI规范.md` §21（§1「内容为圆角页」一行改指 `--page-fill`）。门禁：`vue-tsc` / `vite build` 通过；`vitest` **52 文件 252 例**（新增 `page-surface-material.test.ts` 5 条）；`cargo fmt --check` / `clippy -D warnings` / `cargo test` 全过；`tauri build --target aarch64-apple-darwin` 通过。**真机验收（打包制品 + 隔离 HOME）**：亮色 页面 `242,232,230` / 卡片 `253,251,251` / 外壳 `249,249,249`；暗色 页面 `63,45,44` / 卡片 `41,41,41` / 外壳 `21,21,21`；窗口移位后各层取色逐位相同。遗留：暗色下页面偏暖（与原型的近中性不同，两边底色构成不同），是否调环境光待定；真机只覆盖了启动落地路由（合成点击落不进该窗口）。证据 `.adg/work/evidence/imp-page-material/`、`.adg/work/evidence/proto-page-material/`。 |
| 2026-09-27 | **设置页两条批注落地：删「首次落地页」只读行；「随 Codex 启动 OpenCodex」恢复为真开关（写入通道跟随官方）**。① 批注「这个没必要显示了」——删除通用设置里那条只读事实「首次落地页 · 随面板状态」（生产与原型两侧同步；`auto_panel` 字段仍在契约中，只是不再有任何消费方），`UI规范` §1.3 就地修订。② 批注「原型有按钮，开发软件却没做到」——原先按 2026-09-23「界面不撒谎」把它收敛成只读结论「官方共享域」，本轮改为**可交互开关**，但**不放松 AC-11**：配置源与写入通道都跟随官方，读走 `ocx codex-shim status`、写走 `ocx codex-shim install|uninstall`，管理器只做投影与转发（不做旁路写入）；新增 `modules/codex_shim`（契约 / 状态解析：先判「未安装」、不可识别按 `unreachable` 不猜）、`infrastructure/codex_shim_source`（受控 spawn：显式路径、冻结环境、30s 超时、写后以官方 `status` 复读为准）、`commands/codex_shim`（`codex_shim_status` / `set_codex_shim`）、`types/codex_shim`；前端 `commands/codexShim.ts` + 设置页开关（官方 CLI 未就绪时退回只读事实「官方 CLI 未就绪」，不给假开关）。门禁：`cargo fmt --check` / `clippy -D warnings` 通过；`cargo test --workspace --features integration-test` **511 通过 / 0 失败 / 3 忽略**（新增 `tests/codex_shim.rs` 真实子进程往返 2 条）；`vue-tsc` / `vite build` 通过；`vitest` **53 文件 254 例**（新增 `settings-codex-shim.test.ts` 3 条，`settings-readonly.test.ts` 移除旧只读断言）；`tauri build --target aarch64-apple-darwin` 通过。**真机验收（打包制品 + 隔离 `HOME` ~/.ocx-accept-shim/home + 桩 `ocx`，`cliclick` + Vision OCR）**：进页面即调 `codex-shim status`；点开关「关→开」调 `codex-shim install` 并复读、旋钮右移；再点「开→关」调 `codex-shim uninstall` 并复读、标记清除。另用**真实官方 `ocx`** 跑 `status → install → status → uninstall → status` 全 `rc=0` 且**原样还原**（`codex.opencodex-real` 备份消失、真实 `codex` 二进制完好）——注意官方 shim 实际落在 `/Applications/ChatGPT.app/Contents/Resources/codex`，**不随 HOME 隔离**，属高风险写入，故真机 GUI 用桩、只经官方 CLI。证据 `.adg/work/evidence/imp-codex-shim/`；规范 `docs/02-项目核心/UI规范.md` §22。 |
| 2026-09-27 | **修「随 Codex 启动 OpenCodex」开关点不动（用户报「我发现无法打开」）**。真机复现定位到两个缺陷：① **子进程 PATH 里没有 `codex`**——官方 `ocx codex-shim install` 只在 PATH 上找 `codex`，找不到时打印「⚠️ Could not find a codex executable on PATH.」但**退出码仍是 0**（等于什么都没做），而管理器沿用的 `status_environment.path` 只有 node 所在目录（`FZ-13` 的受控发现结果），于是「codex 由 ChatGPT.app 内置」这类 macOS 常见安装永远装不上；用应用真实环境复现：`env -i HOME=… PATH=~/.local/bin ocx codex-shim install` → 该警告 + `rc=0`。② **补上 PATH 后 macOS 拒绝本应用改动别的应用包**——官方 `rename` 报 `EPERM: operation not permitted`（终端有「App 管理」授权所以手工能过，打包应用没有），而写入失败既不按目标状态校验、也不采集 stderr，界面只好静默或不给原因。修法（均不放宽 AC-11，写入仍只经官方 CLI）：`infrastructure/codex_shim_source` 按**固定候选**（`<HOME>/.local/bin`、`/Applications/ChatGPT.app/Contents/Resources`、`/opt/homebrew/bin`、`/usr/local/bin`）补齐 PATH 且**只接确实存在 `codex` 的目录**；写入改为**按目标状态复读校验**（退出 0 但没生效即失败）；写入路径采集 stderr 并逐行脱敏截断，前端把两种已知形态映射成可执行中文说明（找不到 codex → 提示先装 codex；`EPERM` → 提示开启「App 管理」权限或改用终端执行 `ocx codex-shim install`）。门禁：`cargo fmt --check` / `clippy --all-targets -D warnings` 通过；`cargo test --workspace --features integration-test` **516 通过 / 0 失败 / 3 忽略**（`tests/codex_shim.rs` 5 条）；前端 `vue-tsc` / `vite build` 通过；`vitest` **53 文件 256 通过**；`tauri build --target aarch64-apple-darwin` 通过。**真机复验（隔离 HOME + 记录调用的 `ocx` 包装，包装只 `exec` 真官方 CLI）**：子进程 PATH 已含 `:/Applications/ChatGPT.app/Contents/Resources`；点击开关依次触发 `codex-shim status → install → status`；失败时界面给出「官方 shim 未生效：macOS 拒绝本应用改动 Codex 所在的应用包；请在系统设置里给「OpenCodeX Desktop」开启「App 管理」权限，或改用终端执行 `ocx codex-shim install`。」；验收后台无半成品（无 `codex.opencodex-real`、`ocx codex-shim status` = 未安装）。**结论**：本机这种「codex 内置在 ChatGPT.app」的形态下，开关要真正生效需要用户授予「App 管理」权限或改用终端——应用现在会明确说出这一点。证据 `.adg/work/evidence/imp-codex-shim/`（`run/14-fix-eperm-toast.png`、`run/15-fix-*.log`）；规范 `UI规范` §22.1。 |
| 2026-09-27 | **官方共享开关补「进行态」与「行内说明」（用户批准）**。真机实测：官方 shim 安装会先做探针，**成功路径要 ~20 秒**；原来只有「控件禁用 + 1.9 秒 toast」，用户看到的等价于「点了没反应」，失败原因也读不完。现补：① **进行态**——写入期间在控件位显示「正在安装…」/「正在关闭…」（`data-testid="codex-shim-busy"`），不再只置灰；② **行内说明**——失败原因与「未就绪」原因写成该行的持久说明（`.setting-note`，`data-testid="codex-shim-notice"`），toast 只留「原因见该行说明」做提示；未就绪按稳定原因码映射中文（`unresolved` / `timeout` / `locked` / 其他），DTO 新增 `reason` 字段；③ 先回读真实状态、再落本次失败说明，避免被回读的 `codexShimError = ''` 覆盖。门禁：`cargo fmt --check` / `clippy --all-targets -D warnings` 通过；`cargo test --workspace --features integration-test` **517 通过 / 0 失败 / 3 忽略**；前端 `vue-tsc` / `vite build` 通过；`vitest` **53 文件 257 通过**（`settings-codex-shim.test.ts` 6 条，新增进行态与行内说明断言）；`tauri build --target aarch64-apple-darwin` 通过。**真机复验（打包制品 + 隔离 HOME，`cliclick` + Vision OCR）**：无 `ocx` 时行内读到「未解析到 ocx 可执行文件；请先在「安装配置」里指定运行来源。」；点击后写入窗口内读到「正在安装…」；失败时行内保留完整中文说明（含「系统设置 → 隐私与安全性 → App 管理」的处置指引）。**顺带确证一条真机事实**：11:09 授权生效、安装成功；**重新构建后同一操作又回到 `EPERM`** —— 被测制品是 ad-hoc 签名（无 `TeamIdentifier`），macOS「App 管理」按 cdhash 记账，**重建即换 cdhash、授权随之失效**；用户「授权后仍无效」要先确认没有换过构建产物。证据 `.adg/work/evidence/imp-codex-shim/`（`run/17-notready-inline.png`、`run/18-busy-inline.png`、`run/18-calls.log`）；规范 `UI规范` §22.2。 |
| 2026-09-27 | **官方共享开关写入进行态改为「行内面板」，并区分开启/关闭（用户批准原型 → 落生产）**。承接上一笔的进行态，用户看原型后三轮纠偏：① 进行态不该挂在行内文字列里，应在该行**内部底部独占一行**，默认那行（标题/说明/开关）保持同一行；② 原型默认关闭，便于点开看效果；③ 关闭（`uninstall`）是短、单一动作，不该复用开启那根长耗时不确定进度条，且要写清「对应指令做了什么」。落地：`base.css` 新增 `.shim-panel`（`grid-column:1/-1`，行内底部小框）及不确定进度条/结果说明/指令清单样式；`SettingsRoute.vue` 新增进行态状态机（`codexShimPhase`/`codexShimElapsed`/`codexShimResult`）与 `CODEX_SHIM_PLANS`（安装/关闭两套标题、提示、指令、结果），开启→转圈＋不确定进度条＋真实「已用 Ns」（**不编百分比**）、关闭→`.is-close` 不给进度条、完成→绿底结果说明（安装=替换 wrapper/备份 `codex.opencodex-real`；关闭=移除 wrapper/还原原可执行文件/清理备份）停留 ~1.4–2s、失败→红底说明（同 `.setting-note` 文案）；框内「查看执行的具体指令」可展开真实 CLI 清单并注明「严格经官方 CLI 转发」。进行态由**真实命令耗时**驱动，**不引入人工假延迟**——原型里的「准备」延迟只是为忠实还原真机等待（官方 install 先做探针、真机 ~20s）。写入仍只经官方 CLI，AC-11 不放宽。门禁：`vue-tsc --noEmit` 通过；`vite build` 通过；`vitest --run` **53 文件 259 通过**（`settings-codex-shim.test.ts` 增至 8 条：进行态小框独占行/开关可见但禁用/开启不编百分比/为安装结果说明、关闭不给进度条且结果点明「还原」、指令下拉含真实命令）；`cargo test --test codex_shim --features integration-test` **5 通过**；`tauri build --target aarch64-apple-darwin` 出 `.app` + `.dmg`。**真实渲染验收（生产构建产物真机无头 Chromium，IPC 打桩）**：默认态 `row.h=70`（一行）、无面板；点击后 `row.h=192`、面板 `x=298 w=1089` 与行内宽一致（**独占一行**）、标题「正在安装官方 shim」、`aria-valuenow=null`（不编百分比）、开关禁用；完成结果说明为「已安装官方 shim：把 Codex 启动器替换为官方 wrapper…」；关闭态 `class="shim-panel is-close"`、**无进度条**、结果说明含「还原」。原型侧：`qa/shim-progress.test.mjs` **36/36** + 真浏览器截图。证据 `.adg/work/evidence/imp-codex-shim/`（`proto/`、`prod/`、README §九）；规范 `UI规范` §22.3（§22.2 第 1 条标注被取代）。 |
| 2026-09-27 | **三方审计后续处置（DEC-DOC-05 / DEF-01 / R-17）**。① **DEC-DOC-05 回写需求侧**：「随 Codex 启动 OpenCodex」官方共享开关此前只有代码 / IMP / 原型，现补需求口——DMD 新增 `REQ-30` / `AC-14`（**Revision 10**）与 §2.3 / §8 / §10 条款（删除「不接管 codex-shim，如未来需要再单独建需求」的过期措辞，改为「不自行接管；共享开关经官方 `ocx codex-shim` 读写、不做旁路写入」），`02-项目核心/能力地图.md` 新增 `CAP-3.4`（`DOM-03`，能力项 3→4），`需求基线.md` 新增 `REQ-30`，`场景与验收矩阵.md` 新增 `AC-14` 段与表行。② **DEF-01 修复**：`.github/workflows/ci.yml` 的 `backend` 作业此前在仓库根执行 cargo（根无 `Cargo.toml`，必然失败），现加 `defaults.run.working-directory: src-tauri`，并把 `cargo test --workspace` 对齐为 `--features integration-test`（与本地门禁一致；真实设备用例由 `OCX_TEST_*` 环境变量自跳过）。③ **R-17 实现层闭合**：生产官方 Web 面板承载确定为运行时子 WebView（`http://127.0.0.1:<port>/web`，不打包官方素材），剩余为发布前随发布计划单独确认（`系统架构.md` §1、`风险与开放问题.md` R-17）。门禁：前端 typecheck / vitest（53 文件 259 例）/ build、后端 `cargo fmt --check` / `clippy -D warnings` / `test --workspace --features integration-test`（517 通过 / 0 失败 / 3 忽略）在 `src-tauri/` 下通过；`ci.yml` YAML 结构校验通过。证据 `.adg/work/three-way-audit/`。 |
| 2026-09-28 | **诊断中心交互调整（用户批注，原型 → 生产同步）**。① **默认落地页改为「环境诊断」**（取代 2026-09-24 的「日志历史」口径）：`navigation.normalizeDiagnosticsTab` 缺失 / 非法取值回落 `doctor`；`#logs?tab=logs` 仍可直达日志历史。② **Doctor 动作按钮改中文名「环境诊断」**（原「运行 Doctor」，运行中显示「诊断中…」）。③ **新增「界面诊断」自检**：环境诊断面板内提供可复现触发入口，故意抛出受控异常走**真实错误边界**（Vue `errorHandler` → `reportUiFault`）置位顶部兜底条 `.ui-fault`，用于验证「关闭提示 / 重新加载界面」恢复动作；只在本次会话生效，不写配置、不改数据，可重复执行——此前 ④ 层「兜底条原生外观」因**制品内无合法触发入口**而无法验收，本节即为该入口的设计与实现。④ 日志历史「面板请求日志」入口改用**标准按钮规格**（`.btn ghost`，与同排「清理日志 / 刷新」一致），原型侧此前是小号胶囊样式。落地：`navigation/index.ts`、`routes/LogsRoute.vue`、新增 `features/diagnostics/selfCheck.ts`、`styles/base.css`（`.selfcheck`）；原型 `index.html`（`.diag-tabs` 默认 `active`、按钮中文化、日志工具栏内标准按钮）与 `qa/*.mjs` 同步。门禁：`vue-tsc` / `vite build` 通过；`vitest` **65 文件 295 例**（`navigation.test.ts` / `log-categories.test.ts` 更新，新增自检走错误边界的用例）；浏览器探针 `audit:browser` **204 通过 / 0 未捕获错误**（新增「界面诊断自检（制品内触发兜底条）」用例，覆盖触发 / 两个恢复动作 / 可重复触发 / 重载后干净）；原型 jsdom **332**、浏览器 **415** 全绿。规范：`UI规范.md` §11.1、`日志分类展示方案.md` §3 / §7。 |
| 2026-09-28 | **界面异常提示改为弹窗（用户澄清）**。用户指出「界面诊断」触发的异常提示**带恢复动作**，应为弹窗；同时澄清**通知中心不在此列**——上一轮误改的通知中心（铃铛下拉 → 居中弹窗）已 `git checkout` 全部回退、未提交，保持既有浮层形态。落地：`App.vue` 的 `.ui-fault` 改为 `.ui-fault-modal`（居中弹窗 + 遮罩，`role="alertdialog"` + `aria-modal`，标题「界面出现异常」，正文可换行），`base.css` 定义容器 / 遮罩 / 卡片并取 `z-index = calc(var(--z-modal) + 1)`（故障提示压在应用弹窗之上）；关闭路径 = 关闭提示 / 点遮罩 / Esc，均只清提示。原型同步（`index.html` 内联预览 → 窗口内弹窗 `#uiFaultModal`），`qa/log-categories.test.mjs` 更新。门禁：`vue-tsc` 通过、`vitest` 65 文件 296 例、`vite build` 通过、浏览器探针 **209/209**、原型 jsdom 336 / 浏览器 415；后端未改（沿用 517）。**生产 bundle 校验**：`dist/` + 无头 Chromium 投递真实 `unhandledrejection` → 弹窗居中 720,450 / 1440×900、宽 480、`role=alertdialog`、有遮罩、两个动作齐备、0 页面错误。重打包 `.app` exe `a3e6c504…` / `.dmg` `1acbb0a6…`；**制品屏上真机复核因复核期间显示器布局变化 + 屏幕锁定而未完成**，待解锁补做。规范：`UI规范` §24（改标题为「运行期异常提示（弹窗）」）/ §11.1。 |
| 2026-09-28 | **卸载线：安装版本管理 → 覆盖任意允许来源、卸载干净、备份只提醒（原型先行）**。用户四轮批注：① 卸载入口原来只在「托管安装」下出现（本机 `discovered` 时永远看不到）→ 改为**只要运行来源已解析就出现 `卸载…`**；② 卸载要**干净**；③ **备份只检测与提醒**，无备份**不阻断**；④ 进度要**动效**，并核实「OpenCodex 装了哪些文件、怎样算干净卸载」。落地（原型 `…/05-原型/原型/index.html` + 其 QA）：动作行按来源常显卸载；卸载弹窗改为**统一方案**（事实 + 分层「将移除」清单 + `完整卸载` / `仅移除包体与入口` 两级 + 备份检测 + 确认 + 进度 + **只读残留核验**）；**文案口径按官方安装包源码核验**（`cli/index.ts handleUninstall` / `cli/uninstall-plan.ts` / `lib/config-ownership.ts` / `service.ts` / `codex/shim.ts`）：包体官方不删（提示 `npm uninstall -g`）、service = `~/Library/LaunchAgents/com.opencodex.proxy.plist`、shim 原地替换 codex 并留备份、`OPENCODEX_HOME` 只删 `.opencodex-uninstall.json` 清单内官方自有路径（元数据缺失则 `refused`）、非清单文件与 `~/.codex` 内自有文件属残留；进度动效（转圈 / 进度条流光 / 步骤脉冲 / 完成弹入，`prefers-reduced-motion` 全关）+ 官方输出口径的命令行明细。**缺陷修复**：TASK-152 压缩高度导致确认项落到折线以下 → 主行动灰着且无原因（用户报「卸载按钮点不动」）→ 确认项改 **sticky 常驻条** + 禁用原因提示。**需求/规范回写（Revision 11）**：卸载边界改为覆盖任意已解析来源并由**应用执行**（官方 `ocx uninstall` 应用调用、外部包体应用代执行 `npm uninstall -g …`、`OPENCODEX_HOME` 纳入清理范围），据修订 `DMD` `AC-01` / `AC-08` / §2.2 / §8 / §10 与 `UI规范` §18.1 / §18.4。门禁：原型 jsdom **336/336**；浏览器 QA **439/439**（`runtime-source` 112）× 0 未捕获错误。规范：`UI规范` §18.1 / §18.4。 |
| 2026-09-28 | **统一卸载实现（后端）**。按 DMD Revision 11 与 §19.25 落地：安全评审补 §12.4.1（T-I11 ~ T-I15）；契约修订 `FZ-51r`（范围轴 `body`/`full`，新增方案与结果 DTO、`uninstall_backup_failed` / `uninstall_command_failed` / `bad_scope`）；内核新增 `BodyOwner` / `ExternalRemoval` 归属判定、`plan_uninstall`（只读方案 + 备份检测）、`execute_uninstall`（停代理 → 备份 → 移除包体与入口 → 官方 `ocx uninstall` → 清空 `OPENCODEX_HOME` 非自有残留 → 只读残留核验）、`SystemUninstallCommands`（受控环境执行 `ocx uninstall` 与 `npm uninstall -g @bitkyc08/opencodex`，包名硬编码）；命令层新增 `plan_runtime_uninstall` 并重写 `uninstall_runtime`；删除旧「只引导」实现。门禁：后端 `cargo fmt --check` / `clippy -D warnings` / `cargo test --workspace --features integration-test` **522 通过 / 0 失败 / 3 忽略**。前端弹窗重做与前端门禁留待下一轮。 |
| 2026-09-28 | **统一卸载实现（前端）**。运行来源动作行的「卸载」对**任意已解析来源**常显（无省略号）；弹窗按定稿原型重做：范围两级、将移除清单（含将执行的固定命令）、形态无法确认时禁用并给原因、备份只提醒不阻断、数据清理为完整卸载专属、确认项 sticky 常驻 + 禁用原因、进行中转圈/流光 + 完整后换真实步骤、官方输出进 **132px 固定高度**明细框、完成态只读残留核验卡片。契约同步 `api.ts` / `features/runtime/store.ts` / `stores/app.ts`（新增 `planRuntimeUninstall` 与 `loadUninstallPlan`）。门禁：`vue-tsc` 通过、`vitest` **66 文件 300 例**、`vite build` 通过、`audit:browser` **209/209**；原型 jsdom 336 / 浏览器 443。规范：`UI规范` §18.1 / §18.4。 |
| 2026-09-28 | **统一卸载：浏览器层审计补齐**。为卸载弹窗加浏览器审计夹具与用例：`src/dev/auditHarness.ts` 新增 `?runtime=stub`（只改内存态、不连真实端点、不执行删除，场景固定为自动发现到的 npm 全局安装，返回真实形状的方案与结果）；`qa/audit.browser.mjs` 新增「交互·卸载弹窗（Revision 11）」，覆盖入口常显与文案无省略号、范围两级、将执行固定命令、备份只提醒（不阻断）、确认前禁用与可读原因、**弹窗无滚动条**且确认项常驻、完成态真实步骤、132px 固定明细框、残留核验逐项结论；`.modal.wiz-uninstall` 放宽高度上限以消除滚动条。门禁：`vue-tsc` 通过；`vitest` 66 文件 300 例；`vite build` 通过；`audit:browser` **225/225、0 未捕获错误**。 |
| 2026-09-28 | **统一卸载：制品内原生复核 + 后端格式修复**。`cargo fmt` 修复 `uninstall.rs` 测试模块两处换行偏差（语义不变）；由当前源码 `tauri build` 重建 `.app`（exe `3cfcdabb…`）/`.dmg`（`ded7c418…`），并在隔离 `HOME` 沙箱实例内用 `CGEvent` 点击 + `Vision` OCR 复核统一卸载弹窗（入口常显无省略号、按来源卸载、范围两级、将移除清单、备份只提醒、确认前禁用+原因、勾选后可点、取消可退出），全程不执行破坏性动作并确认真实数据根/钥匙串未受影响。门禁：后端 fmt/clippy/test（522/0/3）通过；`audit:browser` 225/225；原型 jsdom 336/336、浏览器 443/443。证据 `.adg/work/imp03-04-execution/evidence/app-smoke/uninstall-native-158/`。 |
| 2026-09-28 | **缺陷修复：卸载后仍显示「可启动」+ 官方卸载顺序**。用户反馈卸载后其他状态判断不更新、界面仍可直接启动。① 状态陈旧：运行来源 `unresolved`（入口消失）时采集失败被「保留上一份状态」吞掉 → `StatusSource` 增 `resolved()`，`StatusCollector` 对「未解析/入口不存在」置 `not_found`（清空 facts/port/pid），瞬时失败仍保留；前端装/卸终态补跑环境发现。② 顺序缺陷：完整卸载原为「卸包 → 官方 `ocx uninstall`」，npm 全局来源的入口就是 `ocx`，先卸包致官方命令失败（真机 history 出现 `uninstall/failed`）→ 改为官方命令先行；UI 与原型同步。契约 `契约字段.md` §5 与本文 FZ-08 刷新表补规则。门禁：后端 524/0/3；前端 vitest 300、audit:browser 225；原型 jsdom 336 / 浏览器 443；制品内隔离实例原生复核通过。 |
