# 03e0 macOS 备份树表与更新详情局部验收

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182。源码为 `03e0ad942c16fe2ef75473eb6693012b3ae684ac`，仅沿用隔离测试 PID42681；日常 PID75764 未操作。测试副本及原包签名边界见上一轮 `OCX-0110-20261011-03E0-MACOS-APPLICATION-MENU`，本目录不代表完整原生、安装升级或发布验收通过。

沙箱根为 `/private/tmp/ocx-0110-03e0-candidate-3wpphyvk/test-data`。01–15 各组含原始 JPEG、AX 及带采集时间的 metadata；截图未裁切或改写，AX 有 full / diff 区别，以 metadata 为准。

## 已取得的局部证据

| 证据 | 实际观察 | 结论边界 |
|---|---|---|
| 01–03 | 空列表生成偏好备份，根 / 年默认展开，月份收起 | 通过本沙箱初始层级及生成操作 |
| 04–07 | AX cell 点击未展开；实际箭头点击后逐层展开至两个真实文件 | 04 文件名含 expanded，但该帧未展开；不得按文件名认定通过 |
| 07、09 | 无复选框场景箭头、文字与叶占位视觉对齐；键盘焦点移至 backup-manifest.json | 09 有 AX 焦点及蓝色焦点环；08 首次尝试未证明焦点。复选框 / 半选 / 禁用 / DPI 尚待验 |
| 10 | 刷新后原展开层级保留 | 点击刷新改变焦点，未独立验证行被移除时的焦点恢复，不记该项通过 |
| 11–14 | 设置进入共享更新中心及管理器详情；当前 0.1.10，可用版本为 —，显示已是最新，缺更新说明，安装禁用 | 不证明真实更新端点、说明来源或安装通过；重新检查无明显变化 |
| 15 | 面板详情当前未知，可用 2.83.0，版本查询失败，缺说明回退及官方网页入口可见，安装禁用 | 只证明未就绪状态不允许安装及详情可达；未点击网页或执行安装 |

实际备份 `bk_20261011072259_f9e7a1ac` 在 2026-10-11 07:22:59 UTC 生成，用途 `manual-preferences`，仅有 `preferences.json` 与 `backup-manifest.json`。载荷 1371 bytes，SHA256 `58e1c8cc33f43243cd1dee8db4a45da712106e44c29c1ad6a0f97288843e3c85`；独立只读重算与清单一致，见 `backup-integrity.json`。没有独立 checksum 文件；未导出偏好正文，未执行恢复、删除、固定或策略修改。

## CI 快照

2026-10-11 07:32 UTC 的原始 gh JSON 保存为 `03e0-ci.json`、`3d856-windows-candidate.json`；argv、时间和原始字节摘要见 `ci-provenance.json`。

- 最新 [CI 38120149821](https://github.com/gzers/opencodex-desktop/actions/runs/38120149821)：headSha 与 03e0 一致；frontend / release-tools / backend / backend-windows 成功，macOS build 仍运行，整体未完成。
- 旧 [Windows 候选 38119648935](https://github.com/gzers/opencodex-desktop/actions/runs/38119648935)：completed / success，NSIS / MSI 构建、runner 安装验证和上传步骤成功，源码为 3d856。未下载或独立校验这些安装器，不能替代最新 03e0 候选身份与真机验收。
- 本轮未重复 dispatch；最新候选须等完整 CI 成功后按同 SHA 查重再生成。

## 剩余验收

长篇说明内部滚动、网页实际跳转、真实阶段进度、备份失败阻断、重启版本回读、焦点恢复边界、异常精确解除与通道隔离、配对性能 / 真实周期、Windows 双权限 / 物理 DPI / 默认数据根 / IPC 及正式签名仍待验。当前沙箱未发现 OpenCodex，准备态不能替代概览就绪态更新入口验收。

IMP / TASK 保持 in_progress；release_authorized / release_accepted 均为 false。本轮没有迁移真实数据、替换日常安装、创建标签、公开发布或 stable 晋升。`checksums.json` 覆盖本目录除自身之外的所有文件。
