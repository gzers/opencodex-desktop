# 第20轮：原版就绪终态失败，转只读归属诊断

Run42469644038，source5cba024a2，26过/1失败，accepted:false。272份manifest成员的大小与SHA256全部核对一致；summary SHA256 8bd95b988482d6d507b133898a15305c6aa79c5ea4be9ceb70a92f60a579e160。原目录/private/tmp/ocxd023-sweep-r20-42469644038-download/windows023-sweep-42469644038-1/保持。

01～13、15～27通过。14：真实user1服务自动分配49346、HMAC/PID/port通过，原30秒仍pending。预算失败后的只读readyz多次得到HTTP503、status=failed，身份一致；没有生成.codex目录，.claude只有agents。实际backend验证300ms、状态来源744ms、仍无保护事实。两真实账户/Profile、代际进程、ACL/策略清理通过。未通过双服务/冲突/停止协议，不是发布通过。

采样没有显示持续的icacls/Windows发现子进程，不能宣称先前“CLI遗留导致持续pending”猜想已证实。late服务终态failed排除了只靠延长等待；保持原预算、不关闭客户端集成、不伪造就绪。

原包源码：启动Codex同步先调用admitCodexWrite；它通过inspectNativeCodexOwnership检查服务状态与服务管理器，unknown/foreign均明确拒绝。Windows实际拒绝原因仍未拿到，Mac禁网/禁日常HOME对照的service-home拒绝不能冒充Windows原因。当前只加测试诊断，不修改维护包/产品门禁。

复用原真实ordinary Rust执行器及其SID/Profile/TEMP/无凭据验证，新增显式opt-in只读小测试。父端装配同一个固定names.2完整闭包；小测试在真实普通用户启动实际Bun和原版inspectNativeCodexOwnership。runRaw仅观察并转发原defaultRawProbeRunner，不注入服务结果；记录固定命令类别、状态、耗时、长度/摘要和错误类别，不输出原命令行/令牌。没有代理服务、客户端/配置写入；断言客户端HOME与config.json仍不存在。诊断进程60秒上限；原执行器继续精确代际回收、账户/Profile和ACL还原。任何诊断结果accepted:false/fullSoftwarePassed:false，不替代第21轮或正式候选验收。

源码只在test/和.github/。本机小crate编译、原执行器静态合同、sweep四条回归、YAML检查通过；Mac不执行Windows归属测试。编译target已移出工作树到/private/tmp/ocxd023-r20-ownership-compile-target，日志和证据保留。取得实际原因前不再盲目重复完整sweep；正式候选42469646550继续其原运行。

## 实际只读结果与后继对照

只读运行42472622892、source07b5b746b完成。真实普通SID原版inspectNativeCodexOwnership用时776ms返回owned/no_service_claim；sc.exe实查1060无服务，schtasks目标非0后完整CSV查询0退出，证明无任务。没有创建.codex或config.json，原执行器账户/Profile、进程、ACL清理通过。因此没有证据允许修改或绕过服务归属门禁；前面的服务归属猜测已被该环境反例否定。

后继只增显式普通账户CLI启动对照：同一精确固定Node→原版bin/ocx.mjs start，子进程环境按生产清空重建，纯模拟127.0.0.1:1静态provider，省略port、当前tier格式。保留原版启动stdout/stderr与60秒独立诊断时间线（不是延长原30秒验收），记录实际readyz身份/终态；不关闭客户端集成、不注入ready、不替换原版默认逻辑。原执行器持有全代际进程账本并在测试结束证明全部退出和新账户/Profile移除；诊断不以PID-only taskkill清理。仅此新测试明确启动自己的原版进程；前次只读诊断仍不启动服务，两个模式互斥。小crate编译、固定Bun语法、原ordinary静态合同和YAML核对通过，Mac未执行Windows分支。正式候选继续原运行，不重新启动全量sweep。

## Windows直接启动输出已确认缺少客户端HOME

运行42473371702实际普通账户/Node→names.2原CLI，10.214秒readyz终态failed；stderr原文明确Codex home不存在，须先启动Codex生成HOME再sync。原执行器进程/账户/Profile与ACL清理通过。因而这次失败不是端口冲突或状态读取错误，原测试没满足默认客户端集成的既有前置。

后继只加可显式选择的空客户端目录对照，不预置config/catalog，不关闭集成、不改ready条件。原“无客户端HOME”诊断保持可单独执行、原件保留，不覆盖成通过。两种模式互斥，通过编译时显式flag选择；正式双SID测试尚未修改，待原版实际注入/ready结果确认再补正确场景前置。
