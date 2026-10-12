# Windows npm 路径复验与脚本二次确认

2026-10-12，Asia/Shanghai。关联IMP-20 / TASK-182，续接[轮换与npm路径故障](../20261012-Windows轮换与npm路径故障/README.md)。原TASK仍缺失，无正式RCP；证据为 `.adg/evidence/OCX-0110-20261012-WIN-NPM-RETEST/`。

## 路径修复与真实复验

`ee65f1158c4f0bb7c283ff09e24120a7884472d0` 的特性分支CI38208548177及候选38208582603均completed/success；fmt、Clippy、Windows全回归、最终EXE与安装器smoke均通过。已快进并推送main，main集成CI38210214739仍在运行，按不同run单独记录。

候选ZIP11729071479摘要 `0a8cc1035123662e968f67344e5e56615d9531ebf8721a218ce92fc33f0dcad5`；证据ZIP11728837252摘要 `c76c6ee831579cae3c7791c9857021ce5411f16b9685edc20e11cfccad085fa7`；与GitHub digest一致。MSI摘要 `17f05f9aee3ddb6e827874f8cb6fc3535bc70327140417412afda1c42255b5ec`；只读提取主EXE摘要 `bb255ce04ecb1582483bb3982aebbbdf5b6070b17fa408e1336e99b3f6e79731` / 0.1.10，与CI安装EXE一致。本机未执行管理器安装器。

旧4f测试进程由其supervisor结束后，复用原容器启动唯一ee实例PID21592，实际普通令牌false/type3、源码日志匹配。生产install_runtime再次请求官方2.82.0，allowScripts=false。05:54:31.143Z开始，05:54:47.758Z npm输出“added 102 packages in 14s”及exit0；随后版本探针失败，返回scripts_required，05:54:48Z来源仍unresolved且保留失败历史。原生事件2245条、npm原日志与监听解除证据保留。

这证明实际npm安装不再出现旧canonical路径栈溢出；不证明完整运行时安装成功、旧失败事件已恢复、版本可读或真实官方升级完成。阶段/日志派生percent不当作可信字节进度。

## 待确认动作的具体对象

原冻结[IMP-01 FZ-50](../../../../../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-01.md)要求：默认忽略脚本，若入口缺失或不可执行，用户在二次确认框显式开启后才允许带脚本重跑一次。本轮未设置allowScripts=true，不能把“完成Windows门禁”的总体授权自动代替该明确二次确认。

只读检查精确官方包及依赖后，待确认对象为：

- `@bitkyc08/opencodex@2.82.0`，registry integrity核验一致，包SHA-256 `3d2e06f075a91529f30ad71c02442886e0ba4e14497080008d0b0e97fb2e7ff7`。
- 使用`--package-lock-only --ignore-scripts`生成审阅锁，125项中标记hasInstallScript的是`bun@1.4.2`，postinstall=`node install.js`。这次锁计划未执行任何安装脚本。
- Bun包integrity已核验，审阅install.js摘要 `c317425810b3b70862713d51b9640e1e02ca9061a9c2f7bbdfe61e8e622ef55d`。脚本在依赖目录准备bun.exe/bunx.exe、探测版本；可从registry下载平台包，临时目录沿用显式TEMP。原脚本保留在忽略的test/software供审阅，不作为项目源码复制入库。
- 准备在普通权限、原隔离HOME/TEMP和目标 `test/runtime/windows0110/data/runtime/opencodex` 内重跑一次allowScripts=true。它会执行第三方代码；数据根/环境隔离不是OS文件访问沙箱，不能声称脚本绝无访问普通账户其他路径的能力。
- 批准只针对这次精确2.82.0安装重跑，不自动包含后续别的版本脚本、真实用户迁移、日常安装替换或公开发布。

待确认计划见`install-script-confirmation-plan.json`，approvalProvided=false。脚本尚未执行，保持原scripts_required门禁。

## 工作目录合同补齐

复验npm原日志还显示cwd继承manager EXE目录，与FZ-50目标前缀合同不一致。修复`e1a1aaa54ccc56530d5da507d0666b6ec8d2c138`已提交推送feature/0.1.10-windows-npm-cwd：pack进入下载临时区，install进入已验证临时前缀；Windows真实Node测试核对普通/canonical输入的cwd归属。保留源码身份与安全简化，不开启脚本。

本地fmt通过，普通CI38210513009及新候选已启动，仍待精确结果和实际安装重跑；尚未并入main。脚本开启前先验证此补丁，避免在不符合原cwd合同的制品上执行代码。

原TASK、桌面连接、管理员/其余物理DPI、实际安装/卸载、自定义根成功、真实官方更新/进度/回读、全事件、30min退避、同机配对性能与真实周期仍待验。旧性能失败、预算、八项门禁不变；日常文件摘要保留，未迁移真实数据、未关闭或发布。先前junction夹具自动审批拒绝仍保留not_run，未绕过。
