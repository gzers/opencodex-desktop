---
id: IMP-OPENCODEX-DESKTOP-01
object_kind: implementation.change
state: planned
title: OpenCodex 桌面管理器实施契约
summary: 冻结项目核心 46 项 FZ 与 9 类契约输入，建立数据根、状态、进程、日志、加密、扩展、同步、CLI 和验收的可实现基线；本文件确认前不创建 TASK。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: []
completion_summary: null
source_refs: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
---

# IMP-OPENCODEX-DESKTOP-01 OpenCodex 桌面管理器实施契约

## Metadata

| Field | Value |
|---|---|
| 状态 | `planned` |
| 授权链 | `DMD-OPENCODEX-DESKTOP-MANAGER` revision 3 → 项目核心 2026-09-14 定稿 → 本 IMP |
| 任务授权 | **0 个**；必须先确认本 IMP，再拆分 `TASK-*` |
| 冻结范围 | 9 类契约 + `FZ-01` ~ `FZ-46` + 建议 `S-2` ~ `S-7` |
| 时间基准 | ISO 8601 / RFC 3339，UTC 存储，界面本地化展示 |
| 编码与路径 | UTF-8；内部路径用绝对路径或以数据根为基准的相对路径，不解释 `~` 为存储值 |
| 冲突原则 | 与 DMD 或项目核心冲突时，先回写权威事实，再修订本文件 |

## 1. 冻结结论

本 IMP 把项目核心里标 ⏳ 的 46 项变成实施基线。重点是：物理目录名用 kebab-case；三维状态内部与 `--json` 都用 `runtime` / `connection` / `operation`；加密用 Argon2id + AES-256-GCM，manifest 用独立 AEAD 包保护；钥匙串服务名固定为 `OpenCodex Desktop`；CLI 入口叫 `ocxd`；允许写入的六个客户端首期全部默认开启，但用户可以在设置里逐个关闭。

- 状态字段内部名与 `--json` 字段名一致，减少二次映射。
- manifest 用「清单 AEAD 包 + AAD 覆盖设备身份与协议版本」的方案，不把口令派生密钥同时用于两类包。
- 扩展投影白名单只做四类机械改写，且每次重写都要展示确认；不做业务级迁移。
- 项目级 `ClientTarget` 只保留模型和落点校验，首期不开放编辑。

## 2. FZ 冻结索引

46 项全部已在 §3 冻结，编号连续无缺号。

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
| 首选 | `<数据根>/manager-state/skills-store/` |
| 兼容读 | `~/.agents/skills/` 只作只读兼容源；写入以管理器源为准 |
| 切换 | 设置里手动切换；每次切换前备份当前源登记与源目录 |
| 迁移 | 只复制登记的 Skill 内容；不搬运客户端目录；失败回滚 |
| 版本 | 源存储加 `store-version=1` |

### 3.2 状态与展示协议

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

展示规则：`danger` 不自动消失；清理按实体既有文案；通知正文与 `action_ref` 不带 S3。

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

#### FZ-14 / FZ-15 日志

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

## 4. 建议项处置

| 项 | 处置 |
|---|---|
| S-1 | 已在项目核心冻结：容器 `0600`、不写符号链接目标 |
| S-2 | §3.5 / §3.7 上限已冻结 |
| S-3 | §3.5 风险口径已冻结 |
| S-4 | §3.1 备份保留已冻结 |
| S-5 | §3.6 来源登记已冻结 |
| S-6 | §3.8 审计与 `--json` 已冻结 |
| S-7 | §3.6 fingerprint 已冻结 |

## 5. 验收环境与 mock 数据

- macOS 首期；不连接真实 CLI / 文件系统 / WebDAV / 钥匙串。
- 所有 fixture 在仓库内使用临时路径，不引用真实家目录内容。
- 覆盖三维状态矩阵、前置缺失、WebDAV 连接与同步、加密导出容器、冲突、通知。
- 用场景驱动替代真实外设；同一 fixture 可重建。
- 回归必须覆盖 AC-01 ~ AC-13 的正常 / 失败 / 取消 / 恢复路径。

## 6. 派生引用回写

本节只记录本轮同步的派生产物，不作为事实源：

| 对象 | 调整 |
|---|---|
| `IMP输入清单.md` | 开头指向本 IMP，后续稳定契约以本文件为实施权威 |
| `docs/README.md` | 实施行改为「IMP 已创建，`planned`」 |
| 协作包 `02-开发实施/README.md` | 前置说明改为 `FZ-01` ~ `FZ-46` |
| 协作包确认与评审 | 下一步改为「确认 IMP → 拆 TASK」 |

## 7. 变更记录

| 日期 | 变更 |
|---|---|
| 2026-09-14 | 创建 IMP，冻结 46 项技术契约。 |
