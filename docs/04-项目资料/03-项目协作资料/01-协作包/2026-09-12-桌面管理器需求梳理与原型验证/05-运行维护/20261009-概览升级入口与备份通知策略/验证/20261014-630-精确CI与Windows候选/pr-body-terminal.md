面板详情以前固定显示缺省说明，管理器网页地址直接由版本拼接。现在两个入口共用精确候选的说明组件：面板先核对同版本 npm 仓库元数据，再读取匹配的非草稿 Release；管理器优先 updater 正文，只有核验过的 Release 才提供网页。说明有加载、可读、缺失和独立失败重试，失败不改变版本检查、备份或安装结果。

按活动根/对象/通道/版本缓存，命中不联网；总截止30s、响应256KiB/正文64KiB、默认TLS与既有代理策略，拒绝重定向和来源错配。关闭观察者、切候选/通道/根及重试控件移除时保留结果归属和焦点。

依赖草稿 PR #1，base为feature/0.1.10-update-center-focus@be7395a，仅审阅本主题17文件；head为6305747f035945f3563296b42d21edcf5c816464，未合入main。

验证：同新增黑盒flow测试在完整be夹具3项断言失败，新实现通过；最终前端97文件/501通过，Rust34组/884通过/6忽略，fmt、strict Clippy、类型与构建通过。首轮编译、夹具和Clippy失败证据保留于docs/governance-main@5aa3f72f的IMP-20及“20261014-精确版本更新说明与独立重试”。

精确630的push CI38490716335、PR CI38491351619和Windows候选38492240481均completed/success。CI与候选WindowsRust各34组/612通过/7忽略；NSIS/MSI隔离runner新装、最终EXE首屏与卸载通过。候选artifact11863515199 ZIP7658531字节/SHA2568861851c191bc03d57dd6d594a037a6e93264268903c4896e1699fd641e37616及证据ZIP独立核验API摘要/CRC/安全条目，两份安装回执字节一致。MSI4591616字节/SHA6725f7f7…、NSIS3275286字节/SHA3d85d834…与回执一致。

保持草稿：runner实际应用令牌均elevated/type1、mode any，沙箱根override、DPR和自动收尾不替代真机双权限/物理DPI/默认安装根/数据保留或正常退出。说明四态/长正文/链接/重试/缓存真实计数仍须新SHA候选实测；旧be制品不可证明新代码。标题栏真实还原、安装数据保留、完整更新/备份/事件、真实周期及官方0.1.9配对性能门禁仍开放；macOS既有CPU/RSS失败不解除。未自动替换当前Windows be候选，不执行未许可FZ50脚本，不合并、签名、发布或晋升通道。

Windows候选下载：https://github.com/gzers/opencodex-desktop/actions/runs/38492240481/artifacts/11863515199
