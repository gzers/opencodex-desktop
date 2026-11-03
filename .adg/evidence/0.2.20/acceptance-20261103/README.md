# 0.2.20 验收纠正与弹窗状态复核（2026-11-03）

采用原型仍为 cfbd90c4b + 1da4cabca。安装验收修正为 PR #45/abe20e9f；浏览器夹具和状态验收为 PR #46/52692a54。未改正式产品代码或日常配置。

## 安装器报告的确定性缺陷

Release 41589977450 的源提交为 efbe46e8，工作流虽为 success，但原始 installer-verification.json 只有 MSI/default、MSI/custom、NSIS/custom 三格，缺少 NSIS/default。保留原件为 release-41589977450-three-cells.json，不接受为四格完成。

确定原因是 PowerShell 变量名不区分大小写：foreach 的 $kind 覆盖参数 $Kind，使末尾的all条件不再成立。PR #45 改名为 $installerKind，并在 success 前校验所请求 Kind/Mode 的全部单元格；漏格、重复、失败及超范围结果均被拒绝。没有采用中断前待验证的 ALLUSERS/MSIINSTALLPERUSER 参数方案。

真实 PowerShell 7.5.4（macOS）运行新回归时，改前生产脚本因“Installer loop overwrites a requested matrix parameter”失败。改后 12 项合同检查通过，包括拒绝真实三格假绿报告、9种 Kind/Mode 组合及缺格/重复/失败反例。Windows CI 快检用 PowerShell 7.6.9 也通过同12项，原始回执见 matrix-contract-windows.json。另有55项发布工具/Windows验收回归通过，20个smoke脚本语法通过。完整原生 CI 和正式包安装矩阵仍待完成；CI backend job 不生成 MSI/NSIS，不能据它证明包体矩阵或注册表清理生效。

## 草稿资产与测试包不一致

draft-assets-snapshot.json 保留取回时的8项服务器摘要；其 targetCommitish 为32bb1cb4，tag和被测源码为efbe46e8。更强的字节证据：

| 包 | 工作流实际测试 SHA-256 | 草稿 API SHA-256 |
| --- | --- | --- |
| MSI | f3a278feb35ec15eac88c9cf44c700ba7a93cf051b8f0145561b418bb0477506 | 6e93a20a30beb416013e1322a978e1e7e4d8dd71eb91b22621e665f880eb1f73 |
| NSIS | cefc22e2b4dcad5885cc6e5d6ef98473c71a95053f36f6c69640a416e866414b | a9bb102ce653fb53eb3c020ab7111d877a6779cdd89a9f8a2ebabe0198ed5e6c |

二者均不同。草稿资产不能承接该CI结果。后续须固定最终源码，重建干净草稿，核对最终上载字节/签名，再通过独立四格工作流验证实际下载包。普通账户smoke现在可显式接收包体ExpectedCommit，避免将新验收脚本HEAD误认为旧包源码。未公开stable、未执行真实同步。

此前“NSIS排序污染是已证根因”“CI全绿证明registry清理/四格通过”等表述均证据不足：排序更改后的三格通过只证明custom MSI在那一轮通过；先前AppSearch重定向的完整机制仍未确定，不能倒推出hive来源或安装范围原因。

## 浏览器弹窗适用状态

DEV夹具遗漏discardUnattempted接口，导致受保护导出或取消预览调用不存在的native IPC并出现清理失败toast。旧浏览器检查仍报通过。新增“正常流程不出现清理失败”的断言在原夹具失败，原报告保留于 cleanup-red。PR #46补全显式内存discard适配，已完成导出保留结果用于重试；正式构建不加载这些DEV夹具。

Chromium/WebKit各26项导出/WebDAV断言、26项导入状态断言，共104项通过；40张截图，pageerror为0。每引擎1180×760、900×600，覆盖：默认不加密/加密字段/生成结果、连接字段与测试/取消、原页同步预览、概览导出原页停留/Escape；空文件、受保护文件同窗解密、错误口令不采用、正确解密后预览、默认备份、Escape取消不采用、重开清除文件及解密输入。导出复测宽屏亮色/窄屏暗色；新增导入状态均亮色。三档材质/全部缩放、本机真实文件、备份恢复、真实WebDAV、原生菜单和其余弹窗状态未由这些结果覆盖。

目录中的报告与截图按引擎/流程对应。已目检WebKit宽屏连接窗和窄屏导入预览，标题/动作栏可见，窄屏内容在内部滚动。原生macOS及全部35个历史调用点仍未完整验收；本轮不关闭S20、AC或整体RCP。

## Current callsite inventory

Current source has 37 static Dialog/Modal component references excluding DEV; current-dialog-inventory.json records exact files/lines. Three ModelTransfer windows and the old SyncRoute direct UiDialog are retired, and six new component/host references are included. These are references, not 37 unique runtime modals. The current ledger maps each reference and preserves remaining native/state work.
