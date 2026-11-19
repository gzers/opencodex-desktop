# TEAM72 原会话事件日志部分取证

主线 cfb3c8aa79de85e5a4ecdbf613acc668945ed0d2，治理 8d9a27d5a03aa3a650345b0f3017029bfc1e28cc，TEAM72／IMP215／r7。固定现场源码 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7，测试 EXE SHA-256 1b482ce541885a141dbd672497643ed65d16fee28644b81d36f0d5478fd427b8。采集器 SHA-256 07d592bd0fd496e122ac0c7372e92c0b05fe918ba158efc1b7feb933b3a88690。

原始回执为 passed:false。普通 SID 后缀 -1003、session 7、WinSta0/Default，原 PID 25144／FILETIME 134395302283231479 前后存活。五次路径和句柄观察一致，唯一普通文件 ID 0ebd4e59:00010000001a4e61，创建 FILETIME 134395302700540299，受保护的用户和 SYSTEM DACL 摘要一致。独立句柄重复读取固定 [0,5622) 字节，SHA-256 4faebbaf1807aa1aeb9ef7524892d681645974172ab3d81ff129bf733aaf7819。固定单次 Explicit 回执和已通过的 ThreadId(51) 观察有锚点，本轮没有重新询问 worker。

首错 private-copy，RuntimeException，HRESULT -2146233087，raw=null。该阶段合并了 Profile ACL 检查、创建子目录、子目录 ACL 检查和写入，无法归因于其中任何一步。私有副本未获验证，事件投影没有生成。准入、扫描、V2 Intent/Pending 及回复均为 Unknown。控制侧核验原回执字节和失败结构，未读取私有原日志。保留原日志、宿主和旧失败现场。
