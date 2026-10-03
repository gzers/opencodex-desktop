<!-- project-governance:managed-start -->
## 项目治理

本项目已绑定全局 `project-governance` Skill。

- 项目 ID：`opencodex-desktop`
- 中文名称：`OpenCodex 桌面管理器`
- 人工长期事实写入 `docs/`；TASK、回执、证书和事务写入 `.adg/`。
- 普通编码、翻译、摘要和机械编辑不自动进入项目治理。
- 高风险迁移、发布、恢复和全局同步必须单独确认。
<!-- project-governance:managed-end -->

## 分支与文件归属

本仓库按产物类型分离分支。**下表是权威归属**；新增或改动任何文件前，先确认它属于哪一类。

| 类别 | 归属分支 | 内容 |
|---|---|---|
| 代码产物 | `main` | `apps/**`、`.github/**`、`test/**` 及仓库根构建配置。开发期间可在 `codex/*` 特性分支上进行，最终并入 `main` |
| 文档与治理产物 | 以 `docs/` 开头的分支（当前 `docs/governance-main`） | `docs/**`、`.adg/**` |
| **全分支共享文件** | **每个分支都必须存在，且内容保持一致** | `AGENTS.md`、`README.md`、`README.en.md`、`LICENSE`、`.gitignore` |
| 仅 `main` | `main` | `CHANGELOG.md`（用户可见发布说明的事实源） |

规则：

- 文档治理产物只在 `docs/` 开头的分支产生与提交；代码产物只在 `main` 产生与提交（特性分支完成后并入 `main`）。
- **禁止**在 `docs/` 分支提交 `apps/**`、`.github/workflows/**` 等实现产物；**禁止**在 `main` 提交 `docs/**`、`.adg/**`。
- **全分支共享文件**（`AGENTS.md`、`README.md`、`README.en.md`、`LICENSE`、`.gitignore`）不属于任何单一分支：在任一分支修改后，必须同步到其它分支并保持内容一致，任何分支的使用者读到的都是同一份规则与说明。
- 用户可见的 README 以 `main`（默认分支）为准呈现；文档分支保留同一份副本，不允许两处内容漂移。
- README 引用的截图等文档素材存放在 `docs/` 分支（`docs/04-项目资料/05-README素材/`）。由于 `main` 不含 `docs/`，README 使用指向 `docs/governance-main` 的**绝对链接**引用素材，使两个分支都能正确渲染图片。
- 文档记录实现状态时只引用实现分支与真实证据，不复制源码，不建第二份事实源。
