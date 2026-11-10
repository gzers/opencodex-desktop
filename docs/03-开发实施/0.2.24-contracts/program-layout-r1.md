# 0.2.24共享程序位置合同 r1

2026-11-10，P24-00第一批，代码main 9f1ffe677。由本机单写，Windows W0可只读消费，W1安装器/CLI接线尚未放行。

## 已落代码

- modules/runtime/program_layout.rs的ProgramLocations::resolve(executable,user_home,platform)只读解析：Mac desktop_application是实际.app，program_root是当前用户~/Library/Application Support/OpenCodeX Desktop；Windows desktop_application是实际exe，program_root是实际exe父目录，自定义位置不改写为默认值。
- runtime_root为program_root/runtime，stable_entry为runtime/bin/ocx或ocx.cmd；versions_root为runtime/versions，current_record为runtime/current.json。version(version)产生versions/<官方三段发布版本>/opencodex和dependencies/{node,npm,bun}的明确路径，拒绝遍历/路径字符/不完整版本。
- Layout::installation_root_for_user共用该解析，启动初始目录选择和运行来源初始化传同一个已解析用户HOME。Mac不再以.app父目录作为运行程序写目标，现有数据根/.codex-desktop分区与路径/权限拒绝保护保持。

## 尚未落盘的持久合同

current.json目前只冻结路径名，尚未写入/消费其内容；版本目录供应、安装清单、原子切换、升级回退和实际包校验属于P24-00/01后续提交，不能让Windows按猜测Schema生成清单。此批ProgramLocations类型不是IPC或持久记录结构，也不代表2.99制品已构建或已安装。当前产品其它供应消费者仍有flat runtime路径待切换，阶段代码不得标版本完成/可发行。

Windows当前只继续W0原生事实后端，使用上层显式给定程序/入口/HOME；待本机给实际激活记录及调用提交后接W1。需要跨共享mod.rs/入口/类型的修改由本机应用，保持单写。

## 实测

隔离复制真实库测试二进制，用合成HOME、显式sandbox根及外网/日常配置写入拒绝策略执行：3程序路径＋11布局＋27供应保护＋3启动共44项通过。首轮缺sandbox根的保护拒绝、零匹配错误过滤器保留，不当产品失败或通过。未启动真实应用/服务、未碰日常配置。证据.adg/evidence/0.2.24/shared-program-layout-r1/，测试目录已清理。
