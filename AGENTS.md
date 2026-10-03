<!-- project-governance:managed-start -->
## 项目治理

本项目已绑定全局 `project-governance` Skill。

- 项目 ID：`opencodex-desktop`
- 中文名称：`OpenCodex 桌面管理器`
- 人工长期事实写入 `docs/`；TASK、回执、证书和事务写入 `.adg/`。
- 普通编码、翻译、摘要和机械编辑不自动进入项目治理。
- 高风险迁移、发布、恢复和全局同步必须单独确认。
<!-- project-governance:managed-end -->

## 分支使用

本项目按产物类型分离分支：文档治理产物只在 `docs/` 开头的分支上产生与提交，实现产物只在 `codex/` 开头的分支上产生与提交。

- 文档治理分支（当前 `docs/governance-main`）：承载需求、治理、协作包、报告与项目事实，即 `docs/**` 与 `.adg/**` 的人工长期事实与治理对象。
- 实现分支（如 `codex/implementation-base`）：承载产品源码、测试、构建配置与 CI，从文档分支的当前状态派生。
- `docs/**` 分支不提交 `src-tauri/**`、`src-ui/**`、`.github/workflows/**` 等实现产物；实现分支不承载第二份文档事实源，文档记录实现状态时只引用实现分支与真实证据，不复制源码。
