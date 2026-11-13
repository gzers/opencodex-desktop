# 0.2.24 macOS真实工程包与隔离窗口

revision: 1；2026-11-11。版本提交main a58f03cb903b2f5e57d4889f2b103f94f6bf4b3e；实际构建在09392b38e基础上加version-source.patch，仅改Tauri/UI/Cargo及锁记录的工程版本。不是整版冻结候选、Windows最终交接或发布验收。

## 工程来源与真实制品

Tauri CLI2.11.4执行build --features custom-protocol --config tauri.runtime-bundle.conf.json --bundles app，Cargo离线依赖模式及固定运行资源强制开启。实际前端production build/优化Rust/密封.app约255.2秒成功，内置Node/npm/Bun/官方2.99资源沿固定024输入不修改上游。五处工程版本现为0.2.24，版本预检通过；此变更只说明开发目标版本，不说明最终范围/候选或README/发布说明已冻结。

实际.app位于main工作树apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app，主程序SHA256 fda508bfbe7ac3e6ee71512c827d1aeb07ecdd0e0280295e8d9034afef26ea20，Info.plist为0.2.24，严格codesign/deep校验通过，ocxd --help返回实际命令。使用adhoc签名，无Apple公证；未签名test发布限制不因此消失。

新单.app包装器实际制作并只读挂载真实产品DMG，核对单.app/Applications/Install.txt、原树与原官方资源SHA，通过。保留工程DMG路径main工作树apps/desktop/tauri/target/engineering024-single-app/OpenCodeX-Desktop-0.2.24-engineering-arm64.dmg，277329903字节，SHA256 39782f66a24b158600810c67653388219150ea94c08f710524fb396276e3d60e。它不是tag/release资产，未上传或分发。

应用内启动记录仍嵌入构建时原base09392b38e；证据保存该base、五处确切源码摘要、补丁原文及补丁SHA和后继版本提交a58f03cb9，不把后提交SHA伪称嵌入binary。最终候选需从固定干净集成树重构，不复用此工程包冒充。

## 原生隔离窗口

CUA连续两次inventory调用超时，未据此猜屏幕锁定。改用项目已有精确PID/executable准入的AX/CG采集器，实际权限观察axTrusted=true、locked=false、onConsole=true、screenCaptureAllowed=true。

首轮直接执行工作树.app，精确OS sandbox拒绝日常用户根读写、只豁免候选.app读取，setup在Tauri resource_dir报unknown path，主窗口未出现，原失败保持。后继不放宽策略：把同一密封签名.app原样复制到/private/tmp隔离Applications后，拒读整个日常用户根，应用主窗口/概览/环境门禁实际建立，AX及该PID窗口截图成功；源binary/副本binary相同、源签名保持。独立C realpath探针在首轮精确policy下对工作树exe和Resources返回EPERM，对隔离目录成功，说明原失败为验证路径/内核隔离冲突，不据此改产品或去签名。

进程环境使用私有HOME/CFFIXED_USER_HOME/TMP/CODEX_HOME和显式OPENCODEX_SANDBOX身份及边界，OS拒外网/日常目录/Keychain。运行只检查真实签名产品首窗，不点击启动/同步/安装或写日常配置。Sandbox按已有合同跳过首次自动供应，环境报告未就绪符合该测试身份；这项不证明正常用户首次供应/生命周期全链，先前独立worker证据不能被拼成原生整链通过。

本轮按保留Popen及唯一进程组只结束自己的GUI测试进程（SIGTERM退出），确认该组为空再删副本.app/私有数据/辅助工具根。所有挂载、包装scratch、自有进程、测试根及重复临时回执清理；只保留一份明确工程.app/DMG在target，不能误称已安装日常软件或删除尚需来源的正式包。

## 证据与后续

.adg/evidence/0.2.24/macos-engineering-build-r1/保存真实构建/签名/包装/命令/AX/截屏、首轮失败与realpath因果诊断、源码摘要和清理回执。accepted=false/frozenCandidate=false不改。没有运行模型、业务WebDAV、真实服务启停或用户配置导入，没有触发GitHub工作流/tag/发布。

WindowsW0 HOME/Profile更正、W1原生PATH/安装器与完整接线、NAME支持出口/采用、实际完整生命周期与客户端还原、其余HTTP代理一致性、官方新发布验证及最终原生候选仍归IMP-34。共享README在最终交付行为稳定后与docs分支同文更新，当前保持0.2.23已发布事实，不能提前声称用户已可安装0.2.24。
