# 待授权：已运行服务的官方状态读取（尚未应用）

证据：sweep42450285614（源码27c7d4d7d）24 pass / 1 fail，accepted:false；256份受控文件已核验大小与SHA-256。前序失败目录全部保留。

## 已证明的事实

- 两真实SID首次供应、实际Node/npm/ocx就绪、路径/IPC/隔离及账户清理通过。
- 服务A经管理器公开process_action启动，在指定端口返回200，healthz的HMAC挑战证明、PID和端口与当前runtime-port.json完全一致。
- 管理器仍保留loading/stopped，原get_status_snapshot的5秒CLI采集没有成功。
- 只读对照：固定Node执行原package bin 5秒超时；固定Bun执行src/cli/index.ts 5秒超时；额外15秒Node仍超时。额外观察不能判定原门禁通过。
- 原collectStatus同时执行Codex发现、客户端/服务管理器、账户/路由等广泛诊断；这与已运行代理的实时健康/就绪不是同一时延范围。

## 建议的产品改动及边界

修改infrastructure/status_source及新增同目录小模块。仅对当前Desktop受管安装、当前显式H内受保护runtime-port.json进行已运行服务观察。

1. 保留现有受管来源/激活验证；只读取当前H，不搜索旧HOME或端口，不修改配置。
2. 检查记录无reparse、普通单链接、大小上限、实际owner、PID>0、合法端口和43字节base64url进程密钥。
3. literal 127.0.0.1、无代理/重定向的healthz挑战及readyz请求。匹配service=opencodex、PID、port、version、响应HMAC；每次新挑战。
4. 请求后重新读取同一受保护记录并校验完全相同。拒绝端口复用、记录改变、外来服务、伪造健康、错误密钥和版本不符。
5. 在原5秒总采集窗口内返回受证明的运行/就绪事实。启动/保护事实只使用官方已认证接口；不可用时未知，不制造none/rebootSafe。
6. 完整CLI诊断路径保留；无运行记录、官方接口不支持或任一核验失败，按原失败/完整诊断规则处理，不把错误解释为已停/已就绪。不会因日志成功、文件存在或tasklist判通过。
7. 不改启动/停止/冲突语义、不增加超时，不降低两用户端口协议要求；测试仍要求两服务同时健康、冲突拒绝、B停A存活、两服务终止及真实账户清理。

## 必须通过的验证

- 模拟真实本地HTTP：正确证明/ready通过；错密钥、错PID/端口/版本、重定向、畸形/过大响应、记录切换和超时拒绝；未证明字段保持未知。
- 原状态映射/失败保留/保护事实/身份复用测试、strict Clippy。
- 同源Windows全量sweep完整状态表，accepted:false；两SID真实启动/健康/冲突/重叠/停止/清理不跳过。
- 产品修改后需正式同源候选及后续保留/OTA/回退/发布合同，不用模拟或sweep代替正式验收。

当前仅记录提案，未实施以上产品改动。用户前次仅额外授权Windows根路径和ProcessContext/路径快照问题；本次变更官方状态读取来源，需要单独确认。

## 官方实际接口复核（2026-11-08；实施前必须补齐）

以维护包原始tgz摘要afb72a4c792c1ca70a0fcd9209d212c78249281d2d7bc42f8eee63aa4464b26b读取源码，各源文件摘要见同目录attested-runtime-source-contract.json。没有使用日常安装或真实服务。

- /healthz实现挑战HMAC；/readyz返回service/PID/port/version/status，但**没有挑战签名**。不能把相同PID/端口的未签名ready响应当作独立密码学证明；原提案第3条需要官方就绪签名协议或明确完整会话绑定设计，并在实现前测试攻击/复用情形。
- /api/startup-health的fresh读取会调用完整CLI/Windows服务管理器诊断。响应认证不等于满足5秒预算；不得从超时或stale fallback推断protection=none或rebootSafe。
- 暂不把只读健康取代完整诊断作为已解决方案。若采用新来源，需要同时补齐官方就绪认证与有界状态投影，可能涉及维护运行包修订、固定摘要及双平台bundle重新构建，而非仅改Desktop一个函数。
- 没有修改产品实现，没有新增诊断轮。当前只等待已发出的产品范围确认；若同意，仍先按上述合同细化和验证，不执行缺认证的捷径。

## 用户采用与实际实现（2026-11-08）

用户已明确允许按以上边界修复并验证。采用同一条固定loopback TCP会话：先healthz随机挑战验签→readyz身份/版本/状态读取→再次healthz新挑战验签，整段连接不可重连，不依赖独立未签名ready响应或端口名字；原记录字节前后完全相同，激活证明前后完整重验。连接断开或HTTP framing不受支持即拒绝。无需修改维护运行包。

此方式没有给readyz添加不存在的独立签名，而是把它绑定到已认证、前后确认身份连续的同一OS TCP流。只接受literal loopback，无代理/重定向。缺少启动保护事实保持null，不发admin凭据或服务管理器请求。完整CLI路径仍保留在没有运行记录/非managed来源的原消费者中；不把其超时当已停止。

原5秒总采集窗口保持。新增真实HTTP与实际status consumer正负回归；Windows sweep第26项独立回执，失败继续其它关卡，整轮仍accepted:false。
