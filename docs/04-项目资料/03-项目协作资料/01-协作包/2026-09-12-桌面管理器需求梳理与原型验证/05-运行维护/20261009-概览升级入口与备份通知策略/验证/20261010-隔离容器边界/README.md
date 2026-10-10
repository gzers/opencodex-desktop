# 显式隔离容器边界验证

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182，源码 feature/0.1.10-maintenance@6da3cc28。macOS 27.0.1 / arm64，本记录仅验证源码，不替代原生、安装器或 Windows 验收。

- full.log：Rust all-targets --no-fail-fast，32 组 / 737 通过 / 0 失败 / 4 忽略；执行时已有边界检查和最终同候选防覆盖保护，但尚未加入容器迁移重启补测及初始化入口检查。
- migration.log：最终源码迁移命令定向 8/8；包含容器内同级目标迁移、目标锁保留、重启绑定与内置 / 外部 HOME 内容核对。
- clippy.log：最终源码 all-targets -D warnings 通过；fmt 与 git diff --check 通过。日志原件及 SHA256SUMS 保留。

## 原生验证启动合同

仅使用本次新建的隔离容器，例如 /tmp/ocx-0110-migration-<运行标识>/：

```text
OPENCODEX_SANDBOX=1
OPENCODEX_SANDBOX_ROOT=<隔离容器>/source
OPENCODEX_SANDBOX_BOUNDARY=<隔离容器>
迁移目标=<隔离容器>/target
外部 HOME（如测）=<隔离容器>/external-home
```

BOUNDARY 必须是已存在目录、严格包围 anchor，不允许文件系统根；省略时仍仅限 anchor，不推断父目录。启动解析一次并捕获到应用状态；初始化、切换引用 / 迁移及外部 HOME 设置按同一边界拒绝越界，链接 / 悬空链接不能隐藏逃逸。普通身份不受这项测试边界限制。

本合同不授予日常目录迁移权。不得把真实应用数据、用户 HOME 或安装目录包含在隔离容器内；后续原生操作须单独记录制品摘要、隔离路径、截图、实际文件核对和重启结果。
