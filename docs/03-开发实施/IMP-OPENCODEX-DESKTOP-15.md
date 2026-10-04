---
id: IMP-OPENCODEX-DESKTOP-15
object_kind: implementation.change
state: planned
title: "0.1.2 更新通道与配置分层（A/B 最小闭环 + 沙箱隔离）"
summary: "承接 MNT-OPENCODEX-DESKTOP-20261004-02。0.1.2 冻结为 A/B 最小闭环：更新通道解耦并接通消费（U-05/U-07）、固化默认配置与类型收敛（U-06a/U-08）、`appearance.theme` 入偏好（U-06b）、通用配置迁移服务骨架（U-09a）、测试沙箱隔离、随实现更新中英文 README（U-10a）。真实自更新端点/签名（U-01/U-02）与官方版本卡片远端查询/代跑（U-03/U-04）待用户裁决，本版不做，登记为延期项。磁盘保持扁平，C 阶段存储迁移延期。"
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: []
source_refs: ["MNT-OPENCODEX-DESKTOP-20261004-02", "IMP-OPENCODEX-DESKTOP-01", "REL-OPENCODEX-DESKTOP-02"]
---

# IMP-OPENCODEX-DESKTOP-15 0.1.2 更新通道与配置分层

2026-10-04。承接运行维护记录 [MNT-OPENCODEX-DESKTOP-20261004-02](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261004-更新通道与配置分层)。

## 1. 范围

0.1.2 冻结为 [范围冻结与待裁决](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261004-更新通道与配置分层/分析/06-0.1.2-范围冻结与待裁决.md) 的 A/B 最小闭环。核心目标：让「更新通道」从占位变为可收敛、可消费；让「配置」从散落默认收敛为单一来源并建立通用迁移引擎；测试先隔离，禁止改写日常配置。

| 编号 | 实施内容 |
|---|---|
| U-05 | 删除重复的 `UpdateChannel::endpoint()` 假地址；检查/安装/调度共用同一通道解析 |
| U-07 | `app_update_channel` 拆为 channel / auto_check / interval；消除 `SettingsRoute.vue` 第二个内存通道真相 |
| U-06a | `preferences.defaults.json` / `runtime.defaults.json` 固化默认，构建校验并嵌入，后端类型化读取 |
| U-08 | 读取与保存共用同一校验；类型契约与默认值单一来源 |
| U-06b | `appearance.theme` 落入偏好，localStorage 仅作首屏缓存 |
| U-09a | 通用配置迁移服务骨架：识别/schema/转换链/未来版本拒绝/锁内备份原子提交 |
| — | 测试沙箱：开发/测试身份与日常身份分离，含哨兵校验 |
| U-10a | 随 A/B 更新根 README 中英文配置说明 |

## 2. 分支与版本

- 从最新 `main`（`aec115d0`）新建 `feature/0.1.2-update-channel-config-layers`。
- 版本号 0.1.1 → 0.1.2：`tauri.conf.json`、`Cargo.toml`、`ui/package.json`、`ui/package-lock.json` 同步；`CHANGELOG.md` 新增 `[0.1.2]`。

## 3. 延期与待裁决

见冻结记录 §2.1 与 §3。U-01/U-02（真实自更新端点、签名公钥、CI 签名制品）与 U-03/U-04（官方卡片远端查询、代跑官方更新）依赖用户裁决，本版不做。C 阶段（磁盘嵌套、容器/同步 schema 迁移、中断恢复）延期。

## 4. 验收（与冻结记录 §4 一致）

前后端门禁全绿；配置读取即校验、非法/过新不覆盖原件、当前 schema 不重复写盘、legacy v0 迁移保留用户选择；所选更新通道驱动真实检查与安装且检查/安装/调度共用策略；沙箱哨兵文件内容与 mtime 不变；macOS/Windows 构建通过、隔离 HOME 冒烟 `startup version=0.1.2`；根 README 中英文一致；结果回写 `REL-*` / `RUN-*`。

## 5. 执行结果

（待实施后回写；未实施前不写结论。）
