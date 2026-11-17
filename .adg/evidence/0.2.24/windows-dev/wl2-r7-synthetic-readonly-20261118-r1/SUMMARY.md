# 栈溢出合成残留只读身份

2026-11-18 00:08:30 +08:00，以PROCESS_QUERY_LIMITED_INFORMATION 0x1000查询27960，实际QueryFullProcessImageName得到失败夹具.tmpMgOtNr/never-resumed-fixture.exe，原创建FILETIME134394012184503726与同次前后观察一致，对应历史CIM23:00:18.450372；任务镜像文件SHA4699667b79d3aebd6e1e90c8b063f80a95b3f32c5c4a69ec49bd4c70217e4a38与原24文件失败归档一致。原父14936/session2，exitCode查询259，inSomeJob=true，CIM单线程132708/state5/wait5。全部查询raw成功，观察句柄明确关闭。

这只是新观察，非原创建handle或原Job拥有者。缺原Job身份/完整成员和monotonic neverResume证明，当前线程状态不补历史；不拿PID或路径摘要作终止能力。准确处置计划executionReady=false，未请求/执行force、Resume、删除、恢复/收养/权限变化；保24文件失败树与原ZIP。它与marenben/session7的r2不同，旧50580/54692授权不延伸。其余Windows开发继续；当前r2、40588、旧删除拒绝根未触碰。
