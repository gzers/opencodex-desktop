# 0.2.24 双平台候选与发布校验

2026-11-24。整版 `accepted:false`；候选构建仅上传 Actions 制品。W2 等待两份候选固定，真实业务同步与日常配置未涉及。

Windows 来源为 `f1948a6cd007856d3b41339c1feb88ba09cb488f`，集成批次结束于 `8cde2fd35a640824bf8de2998dff2dcdba3c5344`。[代码](https://github.com/gzers/opencodex-desktop/commit/f1948a6cd007856d3b41339c1feb88ba09cb488f)

复核的 Windows 回交位于 `1667d14bd54225dc9ab92c4bb5458cb9f21eeaa7`。GAPS.json 共 2,948 字节，SHA-256 为 `5e041cbb31b8270a51dd0090ac372c59a1e6712cc68501cea86ce347597eda34`。普通用户交互式 NSIS 已过。临时管理员环境固定 MSI 正常卸载通过，注入退出 7 时 MSI 返回 1603 且保留登记，策略已恢复。组件引证还覆盖撤销与卸载的顺序。实际登记的终端入口撤销、普通用户交互式固定 MSI、双 SID 和账户及 Profile 清理仍待真机验证。[交接](https://github.com/gzers/opencodex-desktop/blob/1667d14bd54225dc9ab92c4bb5458cb9f21eeaa7/docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md) · [缺口表](https://github.com/gzers/opencodex-desktop/blob/1667d14bd54225dc9ab92c4bb5458cb9f21eeaa7/.adg/evidence/0.2.24/windows-dev/wl2-r8-remaining-gaps-20261124-r1/GAPS.json)

官方 `@bitkyc08/opencodex@2.99.0` npm 归档 SHA-256 为 `b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c`。发布检查核对制品实际大小与摘要、更新 Minisign 签名、macOS DMG 与已签 updater 的应用内容、Windows 安装器内的官方运行包。35 项发布校验和 40 项 0.2.24 定向测试通过。发布测试覆盖正确签名、摘要不符及重新计算摘要后仍被原签名拒绝的改包。[发布校验提交](https://github.com/gzers/opencodex-desktop/commit/cec780871a82a66c96b5c4f664d242c1a8812f56)

构建失败记录单独保留。第一轮 Windows 在读取 Cargo 清单时失败，Mac 缺少打包入口的 Tauri 运行包配置。第二轮 Windows 仍因打包入口缺少该配置而失败，Mac 在队列中等待期间主线已推进，启动时由候选提交校验拒绝。后续修复按 UTF-8 读取，并把生成配置放到实际打包入口。[第一轮](https://github.com/gzers/opencodex-desktop/actions/runs/45359191238) · [第二轮](https://github.com/gzers/opencodex-desktop/actions/runs/45359482990)

第三轮两端均完成打包和更新签名。Mac 的 tar 头包含 Unix 文件类型位，旧验证器误报权限位。Windows 在读取 Cargo 产物元数据时被单硬链接断言拒绝，旧日志未记录链接数。后续校验保持对文件种类、权限、路径与打开句柄身份、读前后完整性和包内内容的核对。Windows 主 EXE 比对使用 Tauri Bundler 2.9.4 对 MSI／NSIS 各自的一次标记替换，其余成员逐字节比较。31 项打包校验与 35 项发布校验通过。[第三轮](https://github.com/gzers/opencodex-desktop/actions/runs/45359860905) · [校验修复](https://github.com/gzers/opencodex-desktop/commit/9a0fa6145b72e34638cdda7ef0241af7c74faae4)

当前候选绑定远端 main f4d3108363375a974328a2a009553fd8629e8bb4。该轮先完成实际打包和独立制品校验，再记录候选冻结结论。[候选构建](https://github.com/gzers/opencodex-desktop/actions/runs/45375451583)
