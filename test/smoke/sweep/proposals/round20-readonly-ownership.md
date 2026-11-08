# 第20轮：原版就绪终态失败，转只读归属诊断

Run42469644038，source5cba024a2，26过/1失败，accepted:false。272份manifest成员的大小与SHA256全部核对一致；summary SHA256 8bd95b988482d6d507b133898a15305c6aa79c5ea4be9ceb70a92f60a579e160。原目录/private/tmp/ocxd023-sweep-r20-42469644038-download/windows023-sweep-42469644038-1/保持。

01～13、15～27通过。14：真实user1服务自动分配49346、HMAC/PID/port通过，原30秒仍pending。预算失败后的只读readyz多次得到HTTP503、status=failed，身份一致；没有生成.codex目录，.claude只有agents。实际backend验证300ms、状态来源744ms、仍无保护事实。两真实账户/Profile、代际进程、ACL/策略清理通过。未通过双服务/冲突/停止协议，不是发布通过。

采样没有显示持续的icacls/Windows发现子进程，不能宣称先前“CLI遗留导致持续pending”猜想已证实。late服务终态failed排除了只靠延长等待；保持原预算、不关闭客户端集成、不伪造就绪。

原包源码：启动Codex同步先调用admitCodexWrite；它通过inspectNativeCodexOwnership检查服务状态与服务管理器，unknown/foreign均明确拒绝。Windows实际拒绝原因仍未拿到，Mac禁网/禁日常HOME对照的service-home拒绝不能冒充Windows原因。当前只加测试诊断，不修改维护包/产品门禁。

复用原真实ordinary Rust执行器及其SID/Profile/TEMP/无凭据验证，新增显式opt-in只读小测试。父端装配同一个固定names.2完整闭包；小测试在真实普通用户启动实际Bun和原版inspectNativeCodexOwnership。runRaw仅观察并转发原defaultRawProbeRunner，不注入服务结果；记录固定命令类别、状态、耗时、长度/摘要和错误类别，不输出原命令行/令牌。没有代理服务、客户端/配置写入；断言客户端HOME与config.json仍不存在。诊断进程60秒上限；原执行器继续精确代际回收、账户/Profile和ACL还原。任何诊断结果accepted:false/fullSoftwarePassed:false，不替代第21轮或正式候选验收。

源码只在test/和.github/。本机小crate编译、原执行器静态合同、sweep四条回归、YAML检查通过；Mac不执行Windows归属测试。编译target已移出工作树到/private/tmp/ocxd023-r20-ownership-compile-target，日志和证据保留。取得实际原因前不再盲目重复完整sweep；正式候选42469646550继续其原运行。
