# TEAM73 原会话只读事件投影

主线 cfb3c8aa79de85e5a4ecdbf613acc668945ed0d2，治理 fb7d0b3cbe455a1d8c1e31071035de547df453da，TEAM73／IMP216／r7。Windows 源码分支 9a49ea219e8f34dfa1867ecebfc6050fa6dcb756，固定现场源码 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7，测试 EXE SHA-256 1b482ce541885a141dbd672497643ed65d16fee28644b81d36f0d5478fd427b8。采集器 SHA-256 ff4cc38475476a60b98fe981c09a40ecfc7447d46bfbeec1a155839b7b5c5e82。原 Explicit 回执 SHA-256 f06070f4f474b94289323872db63f73657f7a39b1eccc571d07d80326967648b。

2026-11-19 06:42 UTC，marenben 普通令牌、原 session 7、WinSta0/Default。原 PID 25144／FILETIME 134395302283231479 前后存活。五次文件身份和两次 DACL 观察与首轮逐项一致。两个独立句柄读取的全部范围恰为 [0,5622)，摘要 4faebbaf1807aa1aeb9ef7524892d681645974172ab3d81ff129bf733aaf7819。内存中解析六条完整记录，无残缺尾部，无私有副本。首轮 private-copy 失败保留于 832378a92959783c11d99dcad4d4aef1a2722d2a。

唯一意图用原 Explicit peer PID 123212／FILETIME 134395359826678405、普通 SID/session 及原 broker 代次绑定。按 request ID 3f608aff-fc0a-44c2-bed3-d6cf807b6c2a 绑定唯一后续结果，意图携带 transport action ID cb64251f-d365-4cbd-80ff-bb2b2a0cbdc7。最末阶段 v2-original-request-result，服务端生成 refused，TimedOut、raw=null。固定 server.rs 中该记录位于 dispatch 返回之后、最终重验、写 reply 和等 ACK 之前。已接收的 ThreadId(51) 来自前置 DiagnosticBefore，本轮未调用 worker。

原客户端 9,999 ms TimedOut、测试 raw101 和 launcher 首错不变。回复送达、dispatch 内实际到达点、原 worker 是否接手本次恢复、扫描阶段、V2 Intent/Pending 持久状态均为 Unknown，未见 recoveryActionId 或 DiagnosticAfter。控制侧独立核对投影的字节、白名单、固定输入和首轮文件观察，私有原日志保留在原隔离目录。原宿主和旧现场留置。
