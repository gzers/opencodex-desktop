# 0.2.24程序供应与消费者合同 r2

2026-11-11，P24-00/01阶段提交：构建main3d3c90943，消费者main9125f7134。承接program-layout-r1，以下为后继准确入口；旧检查点按当时源码保留，不把阶段代码当整版完成。

## 当前产品入口

- 启动通过RuntimeHandle::initialize_versioned，仅取实际program_root/runtime/current.json。固定程序根保持：Mac当前用户Library/Application Support/OpenCodeX Desktop；Windows实际安装exe父目录。用户配置/.codex-desktop不从程序目录猜测。
- 当前内部程序入口为runtime/bin/ocx-runtime（Windows ocx-runtime.cmd），用于受控启动/查询和所选版本绑定。runtime/bin/ocx（Windows ocx.cmd）保留给用户显式启用的终端转交，后者必须绑定实际OPENCODEX_HOME，不因安装完成自动暴露PATH。此分工不新增产品设置；它防止内部shell入口误导用户以为终端已绑定相同配置。
- ProgramLocations包含stable_entry（内部）与cli_entry（用户终端）；SelectedProgram/ImageManifest/current.json字段及版本子目录以实际代码为准。Windows贡献不自行把内部入口当用户命令，也不从管理历史找旧flat程序。
- startup_bundle已按versioned构造器调用bundled::versioned::provision。当前完整已装程序可以只读接纳，不需要写安装锁/重新解包；缺失才进入创建式首装。未知非目标文件不读取或自动迁移，保留其字节；新版本目标/内部入口/current等保留位置发生冲突仍拒绝覆盖。

## 实际版本和状态

runtime_source改为后台工作线程上的实际来源核验，source_dto的version/managedPrefix直接取当前经过校验的版本镜像，不再只用runtime.json历史值。只读observe_current不改安装记录。metadata损坏、pending激活或入口变动明确报不可确认，状态层使用Unreachable/Unknown并清除旧健康/端口/PID，不记成NotFound；真正没有current才是缺少选择。

来源查询可观察新当前选择，不能凭历史伪造新鲜版本。历史记录即使损坏也保留原字节，不擦掉或拿它回退执行。程序文件被官方面板原地替换且不能与激活证明一致时会拒绝旧完整性结论；“展示外部变化后的实际版本与重新核对/更新动作”仍需POST-07后续UI/动作流程，不因本批拒绝旧结论就声称全专题完成。

RuntimeExecutableProvider增加managed_program_identity，由版本化实现返回同一所选镜像，旧明确测试构造器保留原行为。认证health/ready和官方Stop的前后完整程序检查使用该方法，不删除原哈希/程序/记录/签名/超时要求；小型选择/入口新鲜度检查不替代完整准入。原生实际5秒状态与10秒完整Stop预算尚未在新产品整链验证。

## 官方包与构建

官方npm2.99.0归档原样，SHA256 b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c。.github/runtime-platforms-024.json保留与旧包完全一致的传递依赖记录，只变更本项目与官方根包身份/归档integrity/bin。未制作维护修订；npm ci --ignore-scripts、所有目标依赖及真实原生头检查保持。

新stage_runtime_bundle_024.py直接供应官方包，恢复资源名official-runtime.tgz，build.rs及固定资源解析器同步。不再调用旧维护包生成器；旧0.2.23脚本只保留历史，本批未改发布工作流，最终release必须切新装配入口，不能将旧工作流打出的包当新候选。

Mac真实构建：11,467个文件/633,363,932未压缩字节，bundle SHA256 5b10c1a888000d54d14783622d9d8330c33f4e1b14a5f163d007337d0424ec7f；descriptor SHA256 e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3。两次规范重打包相同摘要，完整树检查通过；仅Mac平台已实际构建，Windows包未构建/验收。

## 实测与残留

本批相关库测试按最终适用结果去重为93项，另2项显式真实包检查：实际供应/重开/原HOME原字节保持，及原样Node26.8.1/npm11.19.0/OpenCodex2.99.0版本探测。后者实际探测约2295ms；含完整解包/验证/同步落盘的首次供应约68秒，不宣称启动服务预算已满足。7项额外只读包检查（版本、原生模块加载、配置validate）全部通过，外网和日常配置/Keychain拒绝，无真实模型/服务/客户端重启。

构建装配单元边界12项通过，1项外部制品参数测试明确未运行，由本轮真实Mac制品证据单列，不抹去skip。原失败/测试输入修正保留。临时测试/构建/readonly副本已清理约1.8GB；只保留一份固定Node构建依赖与当前构建资源。工作区原先0.2.23输入目录先按摘要移到target/retained-build-inputs保全，没有删除旧制品或已安装软件。

证据：.adg/evidence/0.2.24/program-consumers-r1/及official-runtime-input-r1/。

## 未完及Windows放行

版本更新切换/取消/中断恢复、卸载、用户CLI启用/禁用与UI/原包仍待接；WindowsW0产品启停/持久归属/完整恢复、W1安装器/PATH及W2最终候选尚未完成。本批不发布W1完整接线放行，不用单Mac包证明双平台。

另Windows接口必须在产品接入前区分user_profile（HOME/USERPROFILE）和opencodex_home（OPENCODEX_HOME）；详同目录windows-w0-integration-findings-r1.md。该改动可由Windows在独占模块内先处理，不等待本机安装器/更新实现，也不需要重跑旧r5服务链。
