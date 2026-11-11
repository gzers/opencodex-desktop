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


## 2026-11-11版本选择与首装API检查点（后继源码）

main4524c84fe已增加program_selection::read及CurrentSelection/ImageManifest/SelectedProgram；后继main ef4c46767完善实际Bun入口绑定和pending保护，并新增bundled::versioned::provision初次供应API。current.json含schemaVersion/version/platform/imageSha256，版本根.runtime-image.json绑定官方包/version/platform/原始tgz摘要、bin及关键程序文件摘要；所有路径由同一VersionPaths派生，不接受记录内随意绝对路径。实际opencodex依赖的node_modules/bun/bin/bun.exe必须与独立Bun文件字节摘要一致，稳定入口必须与所选Node/bin生成文本完全一致。hash是既有安装来源的漂移检查，不能冒充上游签名或首次来源准入。

versioned首装复用完整descriptor/tar/链接/大小/数量/文件树校验，再将程序和依赖落于versions/<version>；单独npm副本也保留总字节/数量上限。版本目录发布不覆盖现存目标；current与entry采用创建式写入，pending存在时普通读取拒绝。完整读回一致才清pending并记本机历史；中断及未知内容保留，不自动迁移/接管flat树。初次供应尚不承担已装版本升级切换/整文件回退/中断显式恢复，这些是后续必须接线项。

31供应＋6选择保护＋1标准app路径检查通过，证据program-selection-r1。此阶段未构建真正官方2.99新离线包，startup_bundle和其它安装/更新/来源消费者仍未全部切换；不能将独立API完成当生产已可用。Windows W1仍未放行，W0回交已收并由main2d68a28bc注册模块；后续以实际生产接线提交更新合同，不让贡献端猜旧路径。


program_selection::read是程序准入/激活完整核验接口，会读取并哈希关键程序文件，不应每次塞入5秒高频状态查询。后续状态仍由受保护实例代次、原保留句柄/Job及官方health/ready证明；真实版本观测与程序漂移检查在对应来源/动作入口处理。此时未声称新产品状态/启停预算已通过。

## 2026-11-11 可恢复版本激活检查点

main60e9fc640 将首次供应接入 [program_activation共享事务](program-activation-r1.md)，后继代码以该合同为准：稳定内部入口、current.json和当前D的runtime.json不再分别写后清pending；三者按原件/后像校验、保留回执后才完成。候选镜像仍须来自已准入供应器，read_image不是另一条上游信任策略或任意版本发现入口。当前源码不会恢复旧flat树，也不会在启动时隐式完成pending。

共享Rust API可显式继续/回退，保留版本镜像；但常规更新源装配、产品恢复命令/UI、真实生命周期准入及CLI尚未全接，W1/W2继续等待。此前检查点关于“无显式恢复API”的描述为当时事实，不再描述此后共享核心；不能将新增核心API当整版安装/更新完成。
