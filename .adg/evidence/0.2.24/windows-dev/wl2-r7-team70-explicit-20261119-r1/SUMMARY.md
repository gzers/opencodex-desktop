# r7 同一原宿主的 Explicit 首次失败

按 TEAM71、IMP214 和 r7 接收 range-r1 单次 Explicit。六份原始回执逐字节归档，成功验收核验器退出 1。合成核验器结果另存，范围仅为离线拒绝样本。原回执 Explicit-result.json SHA-256 为 f06070f4f474b94289323872db63f73657f7a39b1eccc571d07d80326967648b。

2026-11-19 04:26:32 UTC，本次管理器 begin-canonical-abort 经过 9,999 ms 返回 TimedOut，raw=null。测试线程在 partial_capture_tests.rs:205 的 Result::unwrap() 触发 panic，外层测试退出 raw101，launcher 第 32 行记录 Manager transport or refusal authority differs.，HRESULT -2146233087。Explicit 未收到经过验证的原生答复，回执没有 recoveryActionId。未见 DiagnosticAfter 产物。此前 Host 与 DiagnosticBefore 的已通过证据沿用，本次对 Host stdout、stderr 和结果做只读摘要比对，与已提交快照相同。系统进程清单仍可见 PID 25144、session 7，该清单只证明 PID 可见。

待证伪假设为，若请求已准入，原线程的准备期三次完整扫描及其前后围栏、两次原件引用读取、附件写入和传输回传可能耗尽同一 10 秒。固定源码显示 before-resume-private-control-v2-events.jsonl 的 intent 在 dispatch 前刷盘，result 在最终身份围栏与写回前刷盘。若同一请求缺 intent、已有非超时 result 或记录了未进入等待的拒绝，优先核对准入、服务端回传或客户端复验。若日志有 intent 及超时 result，仍须结合原线程计量区分准备扫描与其它等待。Host 首启单次完整扫描的观测区间为 2.096–2.509 秒，该样本不能替代本次恢复计量。

当前账户打开私有原生事件日志被拒绝，raw5。原生请求是否准入、是否到达原 worker、实际扫描覆盖和 V2 Intent/Pending 是否落盘均为 Unknown。现有公开 Host 日志没有新增可归因于该动作的计量。供集成线交接的最小输入是 marenben 同一 SID/session 对现有私有事件日志的只读快照，并结合既有公开 Host 日志核对原申请和结果。若日志覆盖不足，需集成线决定新隔离配对及分段证据，仍用原期限和完整扫描。此次单次尝试已消费，未补跑 DiagnosticAfter，未改变留置进程、文件、锁或 ACL。旧官方 r2 独立保全。
