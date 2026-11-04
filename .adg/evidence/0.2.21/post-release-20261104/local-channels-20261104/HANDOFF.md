# 本机L2交接

代码codex/post021-sync-fixes / 0e604cb2b6be706af767d9937532fe056cac7ca6，沿草稿PR #69接续；候选仍为0.2.21 debug，不是新发行。

S21-POST-06：新增/编辑只用渠道名称，名称大小写归一后检查唯一；可使用中文、空格、保留词。内部运行键自动分配，已有保存名称恢复时保留原键，名称冲突独立在后端拒绝。已有name字段和新渠道name随真实保存更新，模型原始ID/当前引用和其它渠道保留。原生已验证中文新建、获取、加入草稿、保存，改名openai后保存重开。

S21-POST-07/08：移除渠道发现/节点检测的地址类别准入和UI私网开关，旧allowPrivateNetwork只作兼容读取、不控制请求；DNS空结果归DNS错误。所有相关真实loopback读取合同以旧值false验证，包括协议/路径/分页/认证/重定向/取消；未放宽HTTP响应、凭据同源和TLS等其它规则。仍不承诺用户Windows代理/Fake-IP现场已复现。

证据source-final.json、validation-summary.json及native子目录。软件双引擎各12项、适用原型各4项与59条受影响原型回归；Vitest1280项。原生实际WKWebView/AX候选12检查通过，自有端口2次带合成认证的GET模型清单，零生成请求，检测前后保存配置一致。没有修改日常配置/Key或运行日常同步。

Windows材质/重启/面板及整版集成发布仍待。合并后Windows原生测试中旧“运行渠道标识”字段和provider/model预览断言应按当前名称唯一合同适配，不能恢复已退役输入来满足旧探针；test/smoke/windows-021-probes.mjs、windows-model-ui.mjs与旧ocx021-macos-native.py的rename_save/readback原探针仍属于历史合同，本L2用专属原生驱动验证新合同。W2实际重启路径不可仅凭本机保存通过关闭。

最终本机回归：Rust全库1211通过/0失败/4原条件忽略，Clippy -D warnings及构建通过；Vitest1280通过。远端CI41845962425当前前端/发布工具/Windows冒烟已通过，双后端待结束，不计通过。两个本机L2任务按自身范围关闭，DMD/IMP继续in_progress，不能发布。
