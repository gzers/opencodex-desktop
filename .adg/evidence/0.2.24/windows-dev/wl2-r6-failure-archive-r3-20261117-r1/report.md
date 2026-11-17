# r3只读归档：PS51完整验证与原失败保留

源码c0d42f8dcc72d56b84fb5bbea050efe2d66ee65f；仅4Windows测试脚本，生产Rust及制品仍3afd4e59d，不冒重编。r2真实普通桌面已验证broker54692与挂起Node50580同原FILETIME，但在Hashtable输入的Measure-Object bytes统计处失败，ZIP未创建，原目录与进程保留。原完整617148字节receipt及548字节错误日志收存。

归档逻辑抽为同一archive-r6-content.ps1供实际入口与本地定向脚本调用；显式Int64累加沿原每文件512MiB/整树2GiB/20000文件上限。完整测试又揭示open稳定日志的目录项length滞后，改为只读原文件handle Length；前后长度/字节摘要/全目录文件membership和ZIP逐member校验保持，不把大小不一致忽略。

实际WindowsPowerShell5.1完整通过70文件/1049255源字节，包含嵌套/空目录、Unicode路径、零字节SQLite、稳定但持写handle日志及随机1MiB文件；独立Python逐member与源bytes/SHA核。原ZIP不可覆盖、重复relative/越界relative/缺失源四反例拒绝。前两本地partial69member失败日志/执行脚本/ZIP留，原marenben进程从未被localproof操作。三个测试scene与ZIP均留，无删除/旧拒绝重试。

批处理现保留PS exitcode，明确成功或失败，旧r1/r2/Public内容不改。r3入口C:\Users\Public\ocx-r6-failure-archive-20261117-r3\archive-r6-failed-preresume.cmd及全部4文件SHA已核；测试helper hash与staged相同，pwsh/PS51解析通过。普通r3尚待实际桌面，不能当归档已完成。

原工作树tracked checkout在执行期间消失，已通过managed工具从remote625101e7b/治理3b3f222ed恢复新独立code/docs检出，旧D feature未改；当前根及此事件明确记录。之前删除拒绝不重试，原失败Public/Profile/旧40588/本轮54692与50580均留。后续完整原包/Closed/Stop/后继/GUI交接/初始化/ownerreceipt/原缺席安全退根/全容量/普通MSI/CAP-trust/历史清理未完，Start/allAccepted=false，不业务/模型/W2/双SID/发布。
