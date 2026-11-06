> 2026-11-06：本定向任务已完成并由集成线核验接收。代码63f9dbe26、证据98625f5a1；当前入口 ../windows-candidate2-targeted-20261105/HANDOFF.md。以下为原验收合同，保留历史，不再要求重复执行。

# Windows候选2定向验收

代码codex/post021-sync-fixes，ca46bc1f3c35988d206e1a2949c708d4f23aab50；文档docs/governance-main最新IMP-30及本文件。新CI42056342409待终态，不用旧b82/bfcf二进制冒充本轮。只补剩余项，不重跑旧材质/同步全矩阵。

1. 取本轮同源MSVC候选，核对run、head、apps树、EXE摘要/内嵌commit与实际启动。若尚无制品先做源码/构建器核对，不把旧包当新包。
2. 在隔离的上游dev0a30c551副本，先应用test/smoke/post021-windows-final/patches/runtime-owner-dev-0a30c551.patch，再应用runtime-provider-reload-baseline.patch。若复用已应用第一份的测试副本，逐前像/后像摘要核对后只应用第二份。不得在管理器仓库或日常已装包上应用。保留未知归属、路径、SID、PID代际、祖先及原10s/8s限制。
3. 用现有build-installable-runtime-candidate.py及同目录patches/runtime-candidate2-review.json，基于原npm2.97归档和元数据生成新的空输出目录。原归档完整性及四个runtime后像必须匹配；版本应为2.97.0-post021.ownerquery.2，15,406,072字节，SHA256 bba03ba0cf02c7bcd1895f63c670cb4a2f4b5272657145e45701b20d5e0f1957。GUI及其他包文件保持，候选不冒充上游正式版本。
4. 用真实管理器离线预检和托管安装该候选，读回完整预发布版本、托管来源和实际入口；故意失败时旧来源/入口/包体必须保留。可选复现本机真实安装测试：显式OCX_TEST_PRERELEASE_ARCHIVE、OCX_TEST_RUNTIME_NODE、OCX_TEST_RUNTIME_NPM后执行cargo test --locked --test runtime_prerelease_real -- --ignored --test-threads=1 --nocapture；仅供隔离根，公共registry下载只为测试依赖，不改日常源。
5. 在可丢弃账号/独立实例从全新渠道改名开始，真实输入/保存后首次确认重启；核对knownModels/recentArrivals基线在reload/目录刷新期间不发生无意义时间重建、config字节保护仍有效，新引用出现/旧引用消失、pending清除、正式桌面旧代退出新代启动；补错误/重试/取消、其他账号代际和日常配置摘要保护。旧恢复pending通过不替代全新首次。源码测试140路由已先红后绿，管理器仅时间变化仍拒绝；不得为通过删除字段或放宽守卫。

沿现有Windows贡献分支回交，必要时适配原隔离驱动的本轮制品身份与fresh-rename准入，不能只改结果标记或扩大清理范围。停止每个测试进程前重新核对路径、PID、创建时间、SID/session；不复用历史固定PID。保留原失败及新回执，回交SHA和未完。

不公开上游PR/fork、不发布、不合并main、不采用候选为日常生产来源、不运行真实WebDAV同步或推理、不重启日常Codex。生产正式运行包交付/维护责任和最终发行安装升级/签名/回退仍由集成线收口。
