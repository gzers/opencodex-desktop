---
id: REL-OPENCODEX-DESKTOP-09
object_kind: release.candidate
state: verified
title: 0.1.9 Windows 外观维护发布
summary: 用户试用确认后0.1.9已公开并晋升稳定Latest；main完整CI、双平台发布、三个更新包旧公钥验签及匿名端点一致性通过。设备专项未测项保留，未替换日常安装。
source_refs:
  - IMP-OPENCODEX-DESKTOP-19
  - de6862d40389d527527b9db8ef52e186a58b625f
---

# 0.1.9 Windows 外观维护发布

2026-10-09，Asia/Shanghai。用户明确表示“我看了下0.1.9基本没有问题了。可以发布。”，本次据此执行 GitHub 发布和稳定通道晋升。该授权是用户试用后的发布决定，不作为未执行设备检查的通过证据。project-governance 仅保存发布事实，实际操作使用 Git／GitHub Actions。

## 范围与源码

`feature/0.1.9-windows-appearance@689a86e6261c9f48ea6c4f89f0991c6267735540` 快进合入 `main`，根 CHANGELOG 在 main 单独补写；版本标签 `v0.1.9` 固定 `de6862d40389d527527b9db8ef52e186a58b625f`。实现包含此前未公开的 0.1.8 Windows 启动／发现／托管运行适配，以及单行原生标题栏、Win11 Mica／实色回退、移除顶部应用菜单行、托盘重载、用户三档画质与玻璃下拉修复。

交付平台为 macOS Apple Silicon、Windows x64。沿用 0.1.7 更新公钥和稳定 latest.json 端点；应用配置与 0.1.7 的差异只有版本号。操作系统代码签名与 OTA 更新签名分别记录，不将更新签名当作 Apple 公证或 Windows Authenticode。

## 发布门禁

- 固定实现提交的完整 [CI 37796440853](https://github.com/gzers/opencodex-desktop/actions/runs/37796440853) 和 [候选安装门禁 37796440954](https://github.com/gzers/opencodex-desktop/actions/runs/37796440954) 已通过，包括 Windows MSI／NSIS 首次安装、真实版本及 EXE 身份、首屏和卸载。
- 发布源码 [main CI 37871885546](https://github.com/gzers/opencodex-desktop/actions/runs/37871885546) 五项 job 全部成功，包含最终 Windows EXE 首屏与 macOS main 专属构建；[Release 37871899116](https://github.com/gzers/opencodex-desktop/actions/runs/37871899116) 双平台全部成功。
- 现有发布脚本要求清单版本及平台完整、实际上传包大小／SHA-256 一致、使用标签内公钥验证更新签名；公开后检查固定与稳定匿名端点、Latest 身份。任何失败均不记为发布完成。

## 公开结果与更新兼容

[Publish Release 37873865375](https://github.com/gzers/opencodex-desktop/actions/runs/37873865375) 以手动 `stable` 模式执行成功。[v0.1.9](https://github.com/gzers/opencodex-desktop/releases/tag/v0.1.9) Release ID 为 `407422226`，公开时间 `2026-10-09T02:17:59Z`（北京时间 10:17:59），`draft=false`、`prerelease=false`；Latest 从 v0.1.7 晋升至 v0.1.9。集成线独立匿名下载固定与稳定 latest.json，内容相等、版本0.1.9，API确认目标Release为Latest。

集成线使用 v0.1.7 原公钥验证实际下载的三个更新包，均通过真实 minisign 验签；公钥文本 SHA-256 为 `30190687026fdc7b7e223884315baa52f6d42c1c37d447b8631e9597c1ecdfee`。macOS 压缩包内 Info.plist 版本／构建号均为0.1.9，标识为 `com.gzers.opencodex.desktop`。旧公钥兼容不等于旧客户端安装升级已完成实机验收。

| 更新资产 | 字节数 | SHA-256 |
|---|---:|---|
| OpenCodeX.Desktop_aarch64.app.tar.gz | 3919806 | 9795e2a754c2efc262fb88eeb30e49d66cf47c506191da92bbaf0a40f378de24 |
| OpenCodeX.Desktop_0.1.9_x64_en-US.msi | 4030464 | b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a |
| OpenCodeX.Desktop_0.1.9_x64-setup.exe | 2884185 | e2538debc1f8acf5b5ed931f3e68d9b128263742c230fbc151513f2833a38006 |

证据入口：[发布证据清单](../../.adg/evidence/REL-019-20261009/manifest.json)。正式发布资产由标签重新构建，身份与候选包分别保存，未将候选未签名文件直接替代正式更新包。

## 保留覆盖边界与回退

既有双权限 WebView DPR 矩阵与原生 13/14 检查的边界仍有效。Snap 完整操作、Windows 10/11 物理 DPI／多屏、人工读屏、真实代理／托盘生命周期、macOS 原生 UI／安装专项及旧客户端升级往返没有新增通过证据。用户试用确认允许本版本发布，IMP-19 的专项检查继续保留 in_progress；不将发布完成等同于全部设备验收完成。

保留 v0.1.7 历史资产作为回退来源；稳定降级需单独授权，已安装新版不会因 Latest 回退自动降级。本次不安装替换本机日常 0.1.7，不重启用户官方代理，也不提交其它工作中的模型配置原型或证据。文档及机器证据只提交 docs/governance-main。
