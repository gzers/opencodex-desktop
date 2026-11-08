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
