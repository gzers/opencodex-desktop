# 03e0 完整 CI 与同 SHA Windows 候选启动

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182，源码为 `03e0ad942c16fe2ef75473eb6693012b3ae684ac`。本轮不修改源码、不启动额外测试实例、不操作日常安装。

`ci-success.stdout` 是 [CI 38120149821](https://github.com/gzers/opencodex-desktop/actions/runs/38120149821) 的原始 gh JSON：completed / success，五个 job 全成功，包括 Windows 后端和 macOS build。此前原生证据目录中的 07:32 UTC 运行中快照保留不变。

`main-ref.stdout` 验证远端 main 身份；`candidate-before-dispatch.stdout` 查重未发现同 SHA 候选。`dispatch.stdout` / `.stderr` 留存单次工作流触发结果。`candidate-after-dispatch.stdout` 和 `run-identity.json` 验证新 [Windows 候选 38122059291](https://github.com/gzers/opencodex-desktop/actions/runs/38122059291)，2026-10-11 15:38（UTC+8）创建，event 为 workflow_dispatch，headSha 与 CI 一致，当前 in_progress。

`commands.json` 记录 argv、采集时间、exit code、原始 stdout / stderr 字节数和 SHA256。空 stderr 文件如实保留；`checksums.json` 覆盖本目录除自身以外的文件。

候选完成后需下载安装器及 runner 证据，独立核验哈希与源码身份，再做 Windows 双权限、物理 DPI、安装目录数据根、原生外观与 IPC 验收。macOS 原生 / 安装、性能 / 真实周期、真实更新 / 事件和正式签名仍未闭合。当前没有版本标签、公开 Release 或 stable 晋升，IMP / TASK 保持 in_progress，release_authorized / release_accepted 为 false。
