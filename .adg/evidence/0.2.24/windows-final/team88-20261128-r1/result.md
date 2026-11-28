# TEAM88 Windows 冷启动定位

accepted: false

- 清场 pass。旧候选 9a452c37d 已作废，PID 108100 已退出，原 SID/session 3 回执无剩余进程、监听，两把锁均可独占打开。正常 Stop 与 Closed 均为 false。原日志保留。
- 待证伪一：固定 1dac7c065 的首次 Status 在解析安装来源后，因当前进程无 owner 槽而返回未知。用 OCXD024_STATUS_TRACE=1 的首个拒绝阶段判断。
- 待证伪二：补齐隔离 cwd 后，Doctor 已进入子进程等待。需分别取得来源解析、入队、创建、退出、stdout EOF 和超时时刻。历史 7.5 秒只证明端到端失败，尚不能归因。
- 待证伪三：仅有空槽、记录缺席与可用端口无法证明冷启动空闲。Windows 在 Status 和 Start 共用原生证明，维持 5/30 秒上限；未知时不得写端口或创建进程。Stop 保持 10 秒。

## 给 Mac 集成线

固定 1dac7c065 只有 Status 原生 trace，Doctor 未分离 spawn、exit 和 stdout EOF。请在共享 Doctor 写集中提供阶段观察和定向验证，保留 5 秒等待，默认只读本地诊断。Windows 单轮固定二进制取证需要能独立观察进程和管道事件，若缺该观察能力，记录为 blocked，避免用代理现象给历史失败归因。受控下载供应继续由 Mac 单写。
