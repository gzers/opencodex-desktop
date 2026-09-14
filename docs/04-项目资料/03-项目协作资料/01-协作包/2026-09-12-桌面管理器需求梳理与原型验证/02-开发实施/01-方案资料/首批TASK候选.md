---
id: 首批TASK候选
object_kind: collaboration.derived
state: draft
title: OpenCodeX-Desktop 首批 TASK 候选
summary: IMP 确认后的第一批工程底座任务范围、检查、证据和授权边界；C-01 ~ C-05 已确认并转为正式 TASK。
source_refs:
  - IMP-OPENCODEX-DESKTOP-01
  - 工作包追踪与依赖
  - 验证设计与交接
  - 工程准备
---

# 首批 TASK 候选

## 1. 授权链

```text
用户确认 IMP
→ 用户确认首批范围
→ 治理机制创建 TASK
→ TASK 授权执行
→ 工程初始化
```

本文件保留当时的候选口径。C-01 ~ C-05 已于 2026-09-14 转为正式 `TASK-*`。

## 2. 候选清单

### C-01 工程底座（WP-01）

| 字段 | 内容 |
|---|---|
| 目标 | 创建 Tauri + Rust + Vue / TypeScript / Vite / Pinia 骨架 |
| 允许写入 | `src-tauri/`、`src-ui/`、`.gitignore`、基础 README |
| 禁止写入 | `docs/02-项目核心/**`、`docs/03-开发实施/**`、`.adg/**`、用户家目录 |
| 产物 | 可构建空应用、命令目录、公共错误与类型、Token 样式占位 |
| required checks | `review:contract`、`review:tests`、`review:platform` |
| 完成标准 | lint / typecheck / test / build 全过；不包含业务功能 |

### C-02 测试基线最小集（WP-21A）

| 字段 | 内容 |
|---|---|
| 前置 | C-01 |
| 目标 | 前端 / Rust 测试命令、临时目录策略、FIX-01 / FIX-03 最小 fixture 设计 |
| 禁止 | 读取真实配置、调用真实 `ocx`、访问真实 Keychain |
| required checks | `review:tests` |
| 完成标准 | 空状态测试可重复执行，失败可定位 |

### C-03 构建基线（WP-22A）

| 字段 | 内容 |
|---|---|
| 前置 | C-01 |
| 目标 | 本机构建、CI 清单、`aarch64-apple-darwin` 目标占位 |
| 禁止 | 签名、公证、发布、Windows ARM64 构建 |
| required checks | `review:platform` |
| 完成标准 | 无签名构建成功；构建清单可重复 |

### C-04 写入原语（WP-17A）

| 字段 | 内容 |
|---|---|
| 前置 | C-01 |
| 目标 | 锁、临时文件、SHA-256 摘要、原子替换的最小 Rust 模块 |
| 禁止 | 数据根迁移、扩展写入、WebDAV、真实系统 PATH |
| required checks | `review:contract`、`review:tests`、`review:security` |
| 完成标准 | 正常、半写入、替换失败、锁超时用例通过 |

### C-05 实例约束基础（WP-18A）

| 字段 | 内容 |
|---|---|
| 前置 | C-01 |
| 目标 | `app.lock`、单实例行为、错误码 `7` 的基础处理 |
| 禁止 | 创建 IPC socket、注册 PATH、开放 `ocxd` 命令 |
| required checks | `review:contract`、`review:tests`、`review:security` |
| 完成标准 | 第二实例启动被阻断且可验证；进程退出释放锁 |

## 3. 证据要求

每条候选转正式 TASK 后都要产出：

- 被测提交；
- 检查命令与结果；
- fixture 或隔离路径说明；
- 失败 / 取消 / 恢复证据；
- 已知限制；
- 与 WP / FZ / AC 的覆盖说明。

## 4. 用户确认清单

确认前需要用户明确：

1. `IMP-OPENCODEX-DESKTOP-01` 已确认；
2. 首批范围接受 C-01 ~ C-05；
3. 工程前端使用 Vue 3 / TypeScript / Vite / Pinia；
4. 不在本批执行签名、发布、真实环境验证。

## 5. 确认记录

2026-09-14，用户确认 `IMP-OPENCODEX-DESKTOP-01` 与首批 C-01 ~ C-05，并同意进入工程初始化。治理机制已创建以下任务，当前保持 `in_progress`，待阶段证据与回执齐备后收口：

| 候选 | 正式 TASK | 当前状态 |
|---|---|---|
| C-01 工程底座 | `TASK-OPENCODEX-DESKTOP-01` | `in_progress` |
| C-02 测试基线 | `TASK-OPENCODEX-DESKTOP-02` | `in_progress` |
| C-03 构建基线 | `TASK-OPENCODEX-DESKTOP-03` | `in_progress` |
| C-04 写入原语 | `TASK-OPENCODEX-DESKTOP-04` | `in_progress` |
| C-05 实例约束 | `TASK-OPENCODEX-DESKTOP-05` | `in_progress` |
