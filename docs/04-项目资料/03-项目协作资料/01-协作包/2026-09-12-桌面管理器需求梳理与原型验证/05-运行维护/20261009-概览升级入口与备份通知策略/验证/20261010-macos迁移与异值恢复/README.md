# macOS 隔离迁移、异值恢复与重启

代码：feature/0.1.10-maintenance@986299bf73b9f26af354f5ea9ecb9bbb9b940db1；macOS 27.0.1 arm64。版本仍为 0.1.9 开发态，原生 custom-protocol 构建、独立 bundle ID、ad-hoc 签名。不是分发制品。

隔离容器 /tmp/ocx-0110-migration-20261010-6da3cc28，HOME=tool-home，OPENCODEX_SANDBOX=1，anchor=source，明确 boundary=容器根，复制目标=target。未操作日常 /Applications/OpenCodeX Desktop.app。

已观察：复制迁移完成、保存新位置；仅重启隔离应用后 startup_inventory_verified，目标中断标记消失，旧源保留。之后从迁移后目标恢复已有深色备份，使偏好从 system 改为 dark；恢复前生成 system 保护备份，关联精确被恢复备份，标记 committed、pinned=false。再次重启隔离应用，深色仍保留；源偏好 system 未改变。

artifact.json 记录实际二进制 SHA；startup-inventory.json 记录启动核对；disk-readback.json 为有界磁盘取证，包含 payload 实际摘要与 manifest；ax-observation.md 为人工转录。重启 stdout/stderr 保留原件。恢复前保护 payload SHA 与 manifest 一致。旧备份中的原 target_path 保留为历史，不重写历史原件。

启动：env HOME=<容器>/tool-home OPENCODEX_SANDBOX=1 OPENCODEX_SANDBOX_ROOT=<容器>/source OPENCODEX_SANDBOX_BOUNDARY=<容器> <容器>/OpenCodeX0110MigrationSandbox.app/Contents/MacOS/opencodex-desktop。仅停止已核对路径的 sandbox PID，不操作日常 PID。

限制：单个隔离用例通过，不覆盖 Windows / 双权限 / 物理 DPR、安装器、真实面板或管理器更新、所有迁移失败分支、恢复中断与回滚、配对性能或真实周期。不据此通过 native-0110/storage-0110 整体门禁或关闭 TASK。
