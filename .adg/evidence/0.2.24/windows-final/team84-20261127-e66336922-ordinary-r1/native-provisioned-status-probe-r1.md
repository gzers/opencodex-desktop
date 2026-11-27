# TEAM84 已准备运行根的原生 Status 定向复核

候选源码 e663369226337c9a9cad594ccd3cafd1719c3711。待证伪判断是已准备的 2.99 运行根在没有对应原生 Owner 时，运行身份校验会耗尽 5 秒窗口或误报空闲。

使用已有的 15119 单账户隔离程序根作为只读输入。桌面镜像复算与该轮启动回执一致，SHA-256 为 d29d0e308c9b34fb7ca0710fcca613a32381b69c6244179666e87761062ec1f0。运行镜像元数据复算与 current.json 一致，SHA-256 为 ab9314d3df4475481736245580b24a4d1a6fa52e8030512d9afc9340b6d6c3b6。元数据指向固定官方归档 b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c。复用 MSVC 测试二进制，依赖记录的 OPENCODEX_BUILD_COMMIT 与候选相同，二进制 SHA-256 为 34d9ea7d32f3628cfeca288eb7a5ca6696f20123d97631e6b64631298a35d299。marenben 的私有运行指针在当前令牌下拒绝读取，普通桌面现场保持原样。

| 观察项 | 耗时 | 结果 |
| --- | ---: | --- |
| 解析运行身份 | 226.243 ms | 2.99.0 |
| 原生 Status（无 Owner） | 0.092 ms | Unreachable |
| 完整 Status 来源 | 232.507 ms | Unreachable |
| 轻量 Status 来源 | 231.632 ms | Unreachable |

[原始测试输出及清理回执](raw/native-provisioned-status-probe-r1.log)。断言覆盖 5 秒原窗口、空 HOME 和 windows024_owner_unknown。单项通过，复现了快速拒绝，仍缺真实普通桌面首次 Status 的耗时和首错。未调用正向生命周期或官方 Doctor。新建的探针状态目录已清除，旧测试现场保留。

本次用户截图 SHA-256 为 4907ffea3b783a0eb7d35cf232fb011f3d70ea597cc0094c27e156ba38b4a4d4，与先前归档的 [Doctor 截图](raw/full-reopen-doctor-unavailable-user.png)一致。已准备程序根内的 Windows 运行入口使用固定内置 node.exe 绝对路径，因此用户配置的 PATH Node 不会替换这个入口的 Node。Doctor 实机报告仍不可用，底层首错仍未知。共享接手点见[现有 Doctor 交接](doctor-source-readback-r1.md)，首次 Status 仍需[可关联追踪和可信空闲证明](first-status-trace-readback-r1.md)。

accepted:false。保持 Start 拒绝，正向预算 Start／Status／Stop 为 30／5／10 秒。
