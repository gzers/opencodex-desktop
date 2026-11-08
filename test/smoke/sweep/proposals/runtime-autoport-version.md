# Desktop维护运行包默认自动端口及轻量版本查询提案

原始证据为第15轮42455704560（源码2de66b9d9）：24 pass / 2 fail、accepted:false。已先展示全表，250个证据文件核对完整。第14轮证明服务可正常监听并签名返回；不是底层端口无法绑定。

实际维护包2.97.0-desktop023.names.1（afb72a4c792c1ca70a0fcd9209d212c78249281d2d7bc42f8eee63aa4464b26b）的schema未填写端口默认10100，而chooseListenPort对同端口已有OpenCodex采取拒绝重复策略。底层findAvailablePort支持0由系统分配。当前测试之前先选端口，不能证明默认自动策略，现已改为省略端口、由运行记录读回，仍要求两个真实SID/独立HOME、不同实际端口、固定端口冲突不影响A、六秒重叠、B停止A存活及完整清理。

第15轮运行包供应后，Node约106ms、npm约3302ms、ocx版本查询10132ms失败。不是用配置版本代替执行就能判通过；需要让实际官方入口自身的只读版本请求不启动完整Bun图。

## 可审查补丁

同目录runtime-autoport-version.patch仅修改维护包两个源文件，尚未应用到产品或构建配方。

- bin/ocx.mjs：仅精确三个版本调用（--version、-v、version，无额外参数）直接读取自身package版本，校验语法并输出原有opencodex <version>。不启动Bun、不读HOME、不修改配置；其它命令/附参保持原dispatch。
- src/config/schema/config-schema.ts：仅Desktop专用维护包的缺省port从10100改为0，表示交由系统分配。已明确配置的正整数及--port固定值保持，0也不会重写成静态值。原同HOME已有服务防重复策略继续保留。
- 不修改/覆盖已采用names.1原tgz。需新维护修订（建议2.97.0-desktop023.names.2），两次独立重建相同摘要后更新真实sha/size/package-lock完整性、双平台bundle及正式候选来源。不能更换原诊断artifact中的资源假装同源。

## 已做与未做

本机采用原始tgz受限解包到全新临时目录，应用版本补丁，Node真实执行三个参数，stdout与原有格式精确一致，耗时35.5/35.8/34.8ms；非法版本明确非0退出且无stdout。预加载守卫阻断child_process、网络、文件写入，HOME/OPENCODEX_HOME/CODEX_HOME临时路径均未创建。证据/private/tmp/ocxd023-autoport-version-proof-20261108.json；临时解包树已删除，证据保留。该验证只是版本入口提案，不是Windows就绪或默认双用户端口通过。

尚需：原同HOME重复/固定端口/跨用户自动选择真实回归、维护修订两次重建及所有摘要、两端完整供应、Windows完整sweep及后继正式包/保留/OTA/发布。预算不增加、门不跳过、失败目录不覆盖。

本批测试改动在既有test/授权范围内。新维护包默认值/版本入口是单独产品语义，尚待明确采用；未新跑诊断，未发布。
