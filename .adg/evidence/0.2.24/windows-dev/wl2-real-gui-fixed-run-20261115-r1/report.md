# 修复后真实GUI普通桌面结果

原marenben SID1003/session7/elevated=false，用户从实际桌面运行修复入口。源码6e312f02270f247d2f440ec369385e523a4a8043/treef669be64f862007a8b148bd2bfc99e1f71c5e528，沿用前轮最终MSVC custom-protocol debug准确三输入，未重编译/重跑其它已通过项。observer114108/raw0，1过0败0忽略，5.78秒。32份sameSID日志逐份base64/长度/SHA256核验，原私有ACL不改。

| 模式 | manager PID | generation | 原退出码 | 事实 |
| --- | --- | --- | --- | --- |
| normal | 89060 | 1 | 0 | 实际main原生可见窗口/pageFinished，正常退出后原broker存活 |
| hold | 91208 | 2 | 1 | 仅对原创建Child GUI句柄执行故障终止，原broker保持 |
| after-failure | 46168 | 3 | 0 | 新真实GUI fresh认证、原broker身份一致 |
| finish | 51796 | 4 | 0 | 新GUI认证、明确请求空host退出 |

每轮由独立observer确认HWND属于刚创建的GUI且visible；实际main URL http://tauri.localhost/、完整normal setup、fixtureManager=false。每轮私有pipe核实际server身份，managerGeneration单调，旧连接重放拒绝。broker31484/FILETIME134392264041593579/SID1003/session7/volume247287385/fileID562949954696391全程同原对象，observer保留原创建句柄；bootstrap/挂起broker与转接边界noJob。finish后原句柄Wait确认退出，host-events记明确非force空host退出，但探针没有采集broker原始exitcode，仍为null，不能用manager0或host事件补签0。

这是实际产品GUI加空broker连续性组件通过；officialOwnerCreated/lifecycleAccepted=false，frontendDomAssertion=false。未创建官方Job/Owner，不能代H完整原官方Owner/Job/source/H生产组合，不产生原件Closed资格或开放Start。完整B1/B2/B3/O1/O2/H/S-D/P/MSI表在final-status.json，原公共门拒绝。

首次0c失败raw101和broker40588现场仍保留，既有原始日志不覆盖，旧broker退出状态仍未知。不凭PID收养/强杀；本成功任务根C:/Users/marenben/ocx024-h-1fa5e23f-12c9-4289-84f3-bcc901642b35保留待同SID全目录归档核验后仅清自有成功目录。原export仅包含日志，不声称已归档完整WebView缓存/制品树，也不是NTFS元数据克隆。无账户/Profile/PATH/日常策略变化，未删除或重试任何历史自动审批拒绝，overallCleanupComplete=false。

后续范围外最小请求仍是shared pure route declaration提取及Status统一原5秒上下文，见前轮精确JSON。Windows完整官方副作用/NativeLease/真实S-D与生产组合继续未完，普通MSI与非空CAP/trust/历史清理缺口保持。无官方修改/业务同步/模型请求/W2/双SID/发布。
