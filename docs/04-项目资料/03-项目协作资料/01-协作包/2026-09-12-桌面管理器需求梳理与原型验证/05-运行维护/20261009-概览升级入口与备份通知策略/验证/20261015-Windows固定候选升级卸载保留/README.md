# Windows 固定候选升级与默认卸载文件保留

2026-10-15，Asia/Shanghai。本机账户/VM方案待确认期间，完成可独立执行的安装器文件保留子集。工具源码`feature/0.1.10-windows-query-etw@3c51367f1be89abf66f69805d29b2a660a570a2c`，被测候选仍为`43142bdbcc7a0b9f1325103febae3f44dce407ab`。两者分开记录，不把固定431包改贴成新工具提交的产品；apps/desktop整树未变。

## 准入与实际安装路径

新增仅临时Windows Actions runner的`installer-retention`模式，MSI/NSIS分别分配独立runner。读取官方0.1.9固定Release407422226及对应资产，核对包名/大小/API摘要/下载字节；431候选固定run38709701621/artifact11960342769，核对来源、API摘要、ZIP安全路径/CRC、包内回执及安装包字节。下载重定向不跨host转发认证，准入本身不执行安装器。

本机6项离线准入、宿主Execute拒绝及PowerShell解析通过。两轮同SHA工具CI38713455586/38713465220完整success，各Mac900/Windows627passed、忽略6/7、35组；push实际3c51367f、PR实际6f92618e，整树相同、二进制身份不推定。原日志ZIP及选定checkout/test原文逐项保留。

17:54:15单次dispatch HTTP204，实际run38716371739/attempt1 completed/success；`retention (msi)`和`retention (nsis)`分别runner1000001100/1000001099，package/query-etw跳过。实际checkout均3c51367f，有独立原job日志证明，没有额外产品启动、打包或ETW诊断。

每个runner顺序安装官方0.1.9 → 写入6份自有夹具 → 升级固定431 → 默认卸载 → 重装431 → 默认卸载。安装目录为任务独占边界下含空格的目录；夹具包括安装目录下data中的偏好/备份文件、自有兄弟custom-root中的两个文件及边界内范围外哨兵。只证明这些文件的保留，不声明应用已选择或解析custom-root。全流程未运行桌面应用，未修改/清理宿主数据。

## 核验结果

| 检查 | MSI | NSIS |
| --- | --- | --- |
| 旧版真实安装版本 | 0.1.9 | 0.1.9 |
| 升级与重装候选版本 | 0.1.10 / 固定431 | 0.1.10 / 固定431 |
| 5个安装/卸载步骤 | 均exit0，无重启待定 | 均exit0，无重启待定 |
| 6个文件 × 5阶段读回 | 字节数/摘要全部保持 | 字节数/摘要全部保持 |
| 两次默认卸载主EXE移除 | 均实际核对不存在 | 均实际核对不存在 |
| 最后安装状态 | 已卸载，未自动重试 | 已卸载，未自动重试 |

文件读回阶段为升级前、升级后、默认卸载后、重装后、最终卸载后，各阶段时间顺序和同一组文件摘要独立重算。MSI升级日志实际`WIX_UPGRADE_DETECTED={7C063778-E6A6-4D0E-9770-4C4349101866}`，`RemoveExistingProducts`返回成功，确认识别并移除官方旧产品，不仅看见新版本EXE。

MSI旧EXE SHA626d63ad…与既有官方只读载荷相同；NSIS旧EXE实际SHA238067b7…单独保存，不用MSI marker推测替代其身份。候选MSI主EXE SHA3be7ea60…、NSIS主EXE SHA014fae29…均与固定431候选回执精确匹配。两包不能互换摘要。

MSI artifact11962517957、NSIS artifact11962713665分别API摘要/ZIP CRC/逐项字节核验，原MSI完整安装日志、原runner回执和实际步骤保留。证据`.adg/evidence/OCX-0110-20261015-WIN-INSTALLER-RETENTION/`139项独立清单/verify-evidence通过；分析初稿另存，后续补强MSI旧产品移除日志证明，不覆盖原始安装回执。

## 未覆盖项与保护边界

通过的是**临时独立Windows中实际安装器升级、默认卸载及重装的指定文件保留**。尚未验收实际应用默认根解析、自定义根优先/普通权限/不可写反馈、旧AppData迁移、真实备份语义、安装失败回滚和显式数据清理。兄弟custom-root夹具未通过应用绑定，不当完整自定义根测试。该证据可复用以减少重复安装动作，不能关闭整个installer/storage门禁。

下载NSIS artifact时两次TLS错误都发生于输出创建前，原作业未重跑；native只读下载独立处理API302，首次状态字符串CRLF误拒绝后只校正HTTP帧空白，下载主机不带凭据，未降低证书校验。原job日志一次TLS读取失败后取回同一终态ZIP。方法边界保留，不当产品失败或增加成功运行次数。

17:59:15本机保护检查native0/工具3c干净，日常EXE43e06bc3…、旧隔离四摘要、原TASK3878bytes/SHA bc7965b4…保持。账户/VM/ACL未改，未正式Mac原生交接。PR5终态说明已更新并独立读回，仍open/draft/unmerged/无auto_merge。

原bd2/e6失败窗口ETW及根因仍缺，Mac CPU/RSS failed、原性能预算/真实周期、beta有效来源及剩余原生/事件/根/安装专项继续开放。原TASK八检查不改，不合并、关闭、签名、晋升或发布。
