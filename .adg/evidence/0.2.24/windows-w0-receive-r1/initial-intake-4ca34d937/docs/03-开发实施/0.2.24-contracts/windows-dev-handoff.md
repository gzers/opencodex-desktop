# Windows 0.2.24 W0开发回交

接令依据：docs/governance-main 3d5c1b302301ad6cc97f8f4ed5b649575892a72b，TEAM-0.2.24 revision1及IMP-34。代码共同基线origin/main 0828a3955615df0a2a3e52fb96ab8249f2e04d1b；未合并旧原型分支。只写新增windows_official_runtime024/**和test/windows024-native/**；父mod、共享Schema/入口/状态/预算、CLI及安装器待Mac接口提交。

## 第一批：身份/私有记录/路径

- 代码分支codex/024-windows-native，源码提交779d820159403f1edad25ef0ad7fc4a62a3bac59；清理编译缓存后实际核验HEAD 12c69e27e615fc3d27dc3d341a9f66d3e0e72903。首次提交误含测试target产物，已在下一提交移除并增加test/windows024-native/.gitignore；未改共享根.gitignore，未改历史。接收以12c69e27e版本树/净diff为准。
- 提供真实Win32 TokenUser/session/elevation，显式当前TokenUser owner的User/SYSTEM私有目录/记录创建，现有ACL只读拒绝不修补，逐层父路径pin、文件ID/单硬链/非reparse核验、有界不可变记录。尚未接产品，类型仅进程本地Rust合同。
- 独立Cargo宿主直接path引用apps中新模块，无复制源码、无Tauri或父入口改动。4项真实原生测试通过：当前TokenUser，原记录/目录分享锁/身份，旧宽权限与硬链接拒绝且不修补，设备/ADS/dot等路径及长度边界拒绝。cargo fmt --check、cargo test --locked --offline、cargo clippy --all-targets -D warnings均exit0。
- 首次开发编译发现OpenProcessToken从错误windows-sys模块导入，修正为System::Threading后通过；属于本批实现缺陷，不冒充原官方包故障。原执行错误由对话工具输出观察，未伪造丢失的原始stderr文件。
- 实际命令、工具链/Token身份、来源文件摘要见.adg/evidence/0.2.24/windows-dev/w0-batch01-20261110/result.json及同目录日志。测试为新临时小目录，RAII释放句柄并删除精确新目录；未操作日常HOME/官方运行包/账户/Profile，无服务启动、最终全量或发布。
- 下一批：原始创建句柄、无先运行窗口的Job分配/Resume、成员/原退出和IPv4/IPv6监听事实。产品整链/正式恢复仍未接线；不重复r5单用户可行性链。

最小共享接线建议：仅在Mac明确模块/类型接口提交后，由Mac注册父模块，并将已解析程序/HOME和共享deadline传入平台原语；记录序列化、官方HMAC/ready准入、状态机与总30/5/10预算仍由共享层持有。W1未放行。
