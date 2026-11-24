# 0.2.24 双平台候选与发布校验

2026-11-24。整版 `accepted:false`；候选构建仅上传 Actions 制品。真实业务同步与日常配置未涉及。

Windows 来源为 `f1948a6cd007856d3b41339c1feb88ba09cb488f`，集成批次结束于 `8cde2fd35a640824bf8de2998dff2dcdba3c5344`。[代码](https://github.com/gzers/opencodex-desktop/commit/f1948a6cd007856d3b41339c1feb88ba09cb488f)

复核的 Windows 回交位于 `1667d14bd54225dc9ab92c4bb5458cb9f21eeaa7`。GAPS.json 共 2,948 字节，SHA-256 为 `5e041cbb31b8270a51dd0090ac372c59a1e6712cc68501cea86ce347597eda34`。普通用户交互式 NSIS 已过。临时管理员环境固定 MSI 正常卸载通过，注入退出 7 时 MSI 返回 1603 且保留登记，策略已恢复。组件引证还覆盖撤销与卸载的顺序。实际登记的终端入口撤销、普通用户交互式固定 MSI、双 SID 和账户及 Profile 清理仍待真机验证。[交接](https://github.com/gzers/opencodex-desktop/blob/1667d14bd54225dc9ab92c4bb5458cb9f21eeaa7/docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md) · [缺口表](https://github.com/gzers/opencodex-desktop/blob/1667d14bd54225dc9ab92c4bb5458cb9f21eeaa7/.adg/evidence/0.2.24/windows-dev/wl2-r8-remaining-gaps-20261124-r1/GAPS.json)

官方 `@bitkyc08/opencodex@2.99.0` npm 归档 SHA-256 为 `b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c`。发布检查核对制品实际大小与摘要、更新 Minisign 签名、macOS DMG 与已签 updater 的应用内容、Windows 安装器内的官方运行包。35 项发布校验和 40 项 0.2.24 定向测试通过。发布测试覆盖正确签名、摘要不符及重新计算摘要后仍被原签名拒绝的改包。[发布校验提交](https://github.com/gzers/opencodex-desktop/commit/cec780871a82a66c96b5c4f664d242c1a8812f56)

构建失败记录单独保留。第一轮 Windows 在读取 Cargo 清单时失败，Mac 缺少打包入口的 Tauri 运行包配置。第二轮 Windows 仍因打包入口缺少该配置而失败，Mac 在队列中等待期间主线已推进，启动时由候选提交校验拒绝。后续修复按 UTF-8 读取，并把生成配置放到实际打包入口。[第一轮](https://github.com/gzers/opencodex-desktop/actions/runs/45359191238) · [第二轮](https://github.com/gzers/opencodex-desktop/actions/runs/45359482990)

第三轮两端均完成打包和更新签名。Mac 的 tar 头包含 Unix 文件类型位，旧验证器误报权限位。Windows 在读取 Cargo 产物元数据时被单硬链接断言拒绝，旧日志未记录链接数。后续校验保持对文件种类、权限、路径与打开句柄身份、读前后完整性和包内内容的核对。Windows 主 EXE 比对使用 Tauri Bundler 2.9.4 对 MSI／NSIS 各自的一次标记替换，其余成员逐字节比较。31 项打包校验与 35 项发布校验通过。[第三轮](https://github.com/gzers/opencodex-desktop/actions/runs/45359860905) · [校验修复](https://github.com/gzers/opencodex-desktop/commit/9a0fa6145b72e34638cdda7ef0241af7c74faae4)

## 同源候选冻结

Mac ARM64 与 Windows x64 均从远端 main `f4d3108363375a974328a2a009553fd8629e8bb4` 完成构建。Windows MSI、NSIS 包内资源核验通过。Mac 核对三个封装可执行文件的构建来源，并确认 DMG 为标准单 .app 布局。两端候选可用于定向验收，整版 `accepted:false`。[构建](https://github.com/gzers/opencodex-desktop/actions/runs/45375451583) · [Mac 制品](https://github.com/gzers/opencodex-desktop/actions/runs/45375451583/artifacts/14913470373) · [Windows 制品](https://github.com/gzers/opencodex-desktop/actions/runs/45375451583/artifacts/14912527342)

| 制品 | SHA-256 |
| --- | --- |
| OpenCodeX Desktop_0.2.24_aarch64.dmg | `67692f88bd3a1eb78bf16db777a1bee1398d3ab4cb611957e87135979a1e4107` |
| OpenCodeX Desktop.app.tar.gz | `e360c657ec8995a33a342f2a64289b3f961ef86caf651afeb91ceb9f263fab76` |
| OpenCodeX Desktop_0.2.24_x64_en-US.msi | `780055175cf325f5a4f452806cb0042552a216506be558c9216e371af02b6e78` |
| OpenCodeX Desktop_0.2.24_x64-setup.exe | `988a55ecbbe662ce46260238401e007e9ec9ad266b69f9b375b443852ebaf65b` |

下载后独立复算四包及三份签名文件摘要，与 runner 账本一致。以同提交的发布脚本和更新公钥离线核验，四个原包通过。Mac DMG 与已签 updater 的 .app 一致，两端包内资源为固定的官方 OpenCodex 2.99.0。另复制 MSI、翻转一字节并重算摘要，摘要门通过后原 Minisign 签名以退出码 1 拒绝该副本。临时篡改副本随验证目录清理。[机器回执](verification.json) · [原始验证输出](verification.log)

Mac updater、Windows MSI 与 NSIS 的独立 Minisign 更新签名通过。DMG 由已签 updater 的应用内容一致性覆盖。Mac .app 的 ad hoc 签名通过严格校验，缺 Developer ID 签名及公证证明。Windows MSI／NSIS 的 Authenticode 均为 NotSigned。平台签名缺口随候选记录，正式发布判定仍待处理。[Mac 封装证据](https://github.com/gzers/opencodex-desktop/actions/runs/45375451583/artifacts/14913261294) · [Windows 安装器证据](https://github.com/gzers/opencodex-desktop/actions/runs/45375451583/artifacts/14912387548)

W2 由原 Windows 对话按 TEAM79 运行。GitHub 未创建 v0.2.24 tag 或 Release。
