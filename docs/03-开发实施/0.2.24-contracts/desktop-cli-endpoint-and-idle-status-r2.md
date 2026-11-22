# CLI 端点与停止后状态：请求前要证伪的两条判断

日期：2026-11-22。代码：main b6785b486。

## 判断

1. 使用正式 .app 时，ocx 能连上 GUI，并按真实状态报告启停。
2. Stop 完成后，状态不会再显示运行中。

## 结果：两条都不成立，均已修

### 端点路径不一致

GUI 把 IPC 端点绑定在活动数据根的 cache/ipc/opencodex.ipc。CLI 却固定查找用户目录下 Library/Caches/com.gzers.opencodex.desktop/ipc/opencodex.ipc，两处从未重合，所以 ocx 一直报 instance offline。这与用户此前反映的「ocx 用不了」一致。

修法：两边共用同一个相对路径常量；CLI 读取启动时登记的 manager-location.json，拼出活动数据根下的端点。CLI 不重跑完整启动准入，也不创建目录；记录缺失或非法时直接报离线。

### 停止后的状态误报

隔离 HOME 没有运行记录时，官方 status 会去探测默认端口 10100，把本机另一个 OpenCodex 实例的 pid 报成本实例。采集器把这个失败当成瞬时错误，保留了上一份 Running，于是 Stop 之后界面仍显示运行中。

修法：管理的运行时一旦没有本实例的运行记录，就作为确定的「未运行」事实；不再调用官方 CLI，也不再沿用旧快照。这是缺少记录的陈述，不授予任何启停权限。外来实例的实时记录同样归为这一类，不会被当成本实例在运行。

## 验证

- 候选 .app（运行包资源映射正确、供应成功）实测：初始 stopped → start → running → stop → stopped。
- 停止后本 HOME 的 ocx.pid 与 runtime-port.json 均不存在，没有残留服务进程。
- 新增回归：删除记录时既不启动官方 CLI，也不保留旧的运行态。
- cargo fmt、严格 clippy 通过；全量 lib 单测 2174 通过 / 5 失败，仍是基线可复现的 5 项。

## 候选产物

- 位置：target/release/bundle/macos/OpenCodeX Desktop.app（268 MB）。
- 可执行文件 SHA256：c53dadb3a0c95446018138ff44247638f32f3313e2c8be45bfb24f32fe4c33fb。
- 签名：adhoc，identifier com.gzers.opencodex.desktop；未公证。
- 构建自 b6785b486；资源映射使用 tauri.runtime-bundle.conf.json，运行包落在 Resources/runtime-bundle。

## 仍未完成

Windows 真机原生启停与取消清理仍需 Windows 侧在固定候选上验证。本机没有 Windows 环境，未执行真实业务同步或模型请求，未改日常配置，未发布。
