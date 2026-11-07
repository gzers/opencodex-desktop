# L4：环境发现后继的Windows补验范围

2026-11-07。只读观察集成52a84e233c4960e30077667d9990d12705cfb0fc，含f73c23bbc2d3b055f3b163534ed815af089fa4e6 discovery修复及52a界面重查。任务文档仍docs/governance-main@323b75df971658908cd6b23915bb880ce41296b0、明确基线7730073e96999382697fe2ad8dd8f877352c75a3。本批未采用未派发的新基线，贡献HEAD05d14a41d及干净代码树保持。

实际源码现默认只接当前RuntimeHandle的Managed根/激活证明、固定Node/npm与包内bin；Windows报告ocx.cmd但使用同一Node执行实际JS探测，不依赖系统PATH。探测进程清空环境、隔离工作目录，接既有OwnedQueryJob暂停创建、assign_and_resume及terminate_and_wait。此为源码接线事实，不是Windows原生结果。

新增discovery_v2_tests.rs整份只在cfg(unix)模块内，包括完整bundle测试；因此该批Mac/Unix测试通过或Windows编译通过，均不能证明Windows新默认发现路径实际Ready。现run42323530379的Windows job128253406896已实证in_progress/Windows backend regression，fmt和strictClippy过；保持同一job观察，不重复派发。其新源码不包含前批回交的model_receive/model_transaction/data_root/uninstall/runtime/archive/普通执行器修复，这6个文件与773 diff为空，不能从环境发现修复推定96失败或退出回执缺口已修。

待L0明确后继源码及可取的完整Windows候选后，最小原生验收需求：

- 用完整本版I/runtime/node/node.exe、node/node_modules/npm/bin/npm-cli.js及runtime/bin/ocx.cmd，默认null发现实际报告Node26.8.1/npm11.19.0/ocx2.97.0-desktop023.names.1和本I路径。合成Profile/Unicode空格I，系统Node/PATH旧候选不得参与；RuntimeStore字节及日常配置不变。
- 版本探测在Windows实际通过固定Node执行包内JS，不能以存在文件或仅cmd入口文本代替；保留原进程/参数、实际版本、真实退出及Job精确清理回执。
- 供应未完成到ready后的实际界面重查需观察，不复用供应完成前的旧Missing报告。该体验由后继完整应用证明，不把Unix测试或源码监听当原生通过。
- 缺失/篡改/版本不符/探测失败应明确拒绝，不能回退系统安装指引；Job暂停归属、超时/超长输出与子进程清理保持原预算/真实进程权威，不误停其它用户。

所需公共代码/测试仍归L0/L1及L3，L4不并发修改。平台正式候选未取得前本批只增加独立证据文档；安装器、相邻升级/卸载保留、真实双SID/端口仍待，第二账户受控通道用户稍后提供。不改日常配置、不真实同步/推理、不清理或发布。
