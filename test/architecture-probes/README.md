# 对话策略可行性探针

研究程序，不是生产模块；真实 Codex / OpenCodex 使用临时配置、伪凭据和回环模型替身。脚本自行创建并清理所属临时目录和进程，不安装插件到日常实例，不重启当前 App，不执行付费模型请求。

20261027-opencodex-extension-contract.mjs 是无网络源码契约检查。其他程序验证真实进程和公开协议；ws_json_client.py 仅供本地测试的有界 WebSocket JSON 传输。

复跑时传入实际二进制 / 包位置，不用会改路由的 shell shim 替代真实 Codex 可执行文件。--native-helper 复用现有 test/codex-collaboration/probe_native.py 中的 RPC / SSE helper，当前存在于同项目 ocx-0110-main 工作树。版本与源码指纹随 JSON 结果记录。

~~~sh
python3 probe_protocol_schema.py --codex /absolute/path/to/codex --output /tmp/protocol-schema.json
python3 probe_owner_connection.py --codex /absolute/path/to/codex --native-helper /absolute/path/to/probe_native.py --output /tmp/owner-connection.json
python3 probe_native_children.py --codex /absolute/path/to/codex --native-helper /absolute/path/to/probe_native.py --output /tmp/native-children.json
python3 probe_opencodex_service.py --package /absolute/path/to/opencodex --codex /absolute/path/to/codex --native-helper /absolute/path/to/probe_native.py --output /tmp/opencodex-service.json
node 20261027-opencodex-extension-contract.mjs /absolute/path/to/opencodex
~~~

脚本运行目录为本目录，或为各脚本提供完整路径。OpenCodex 探针适配当前已读的 2.84.0 包和包内 Bun；升级后先核对路径、配置 Schema 与临时服务隔离行为。Windows 插件自动加载在此版本禁用，不通过修改原版绕开。

结果范围：

- currentAppAttachment 描述当前观察到的启动方式，不能将独立 owning server 通过写成当前 App 已连接。
- Native children 的项目 / 对话差异通过 start/config 显式投影产生，不证明 manager 的分层存储或自动解析已经实现。
- 搜索 reasoning 保存读回、候选拒绝与真正搜索执行是三项不同能力。
- 客户端收到 compaction 项不能证明上游 native；同时检查实际摘要请求和压缩后续聊。
- 后台开关被解析不证明后台调度；没有事件和请求证据时保持 inconclusive。
- Passed 包含负例与限制断言，不代表每个产品功能可用。
