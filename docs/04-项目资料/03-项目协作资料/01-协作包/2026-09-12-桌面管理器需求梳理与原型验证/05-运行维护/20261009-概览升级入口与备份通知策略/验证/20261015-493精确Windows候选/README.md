# 493 精确 Windows 候选准入与 beta 来源补核

2026-10-15，Asia/Shanghai。源码已因[独立 Profile 准入工具](../20261015-Windows独立Profile准入/README.md)推进到 `493da80675e6508e46e0fd83d86ee7216abf96fe`，旧 ca26 EXE 不作为新 SHA 制品。本轮补齐当前精确来源候选，全部宿主操作为后台读取、下载和 MSI 只读提取，没有前台授权延长、宿主安装或产品启动。

## 一次请求与真实终态

请求前核验代码 worktree 精确 493 且干净、远端分支相同、push CI `38670771363` completed/success，且该 SHA 没有已有 Windows candidate run。10:52:40 独立读回一次 candidate 请求的 run `38673288645` / attempt 1，package 在运行，query-etw 明确 skipped；不是第二轮 ETW，也没有重派诊断。

第一段只读观察持续 633.657s，结束时远端仍在 NSIS/MSI 构建，conclusion=null；这不是远端停止。随后读取同 run，第二段 387.735s，于 11:10:02 取得 completed/success，package 成功、query-etw skipped。两段都没有再次 POST。旧观察快照、传输错误与独立终态均保留。

原始日志 ZIP 79543 bytes / SHA256 `9755718757706a3ce8e11d6001c0768b408f468d898ef8f62d6ed4d6facb02d8`，CRC 与条目检查通过；20 份 package 原始阶段日志逐字节压缩保存。checkout 日志确认实际检出完整 493；包锁回归 35 组、627 passed / 0 failed / 7 ignored。

## 制品与载荷身份

| 产物 | API 身份与大小 | SHA256 |
| --- | --- | --- |
| 安装 ZIP | artifact 11945931457，7626883 bytes | `921c52cf8720fc0731f47c4ff3f983f454eb4a0b3e745bcd7ac56019fa24ccf6` |
| runner 证据 ZIP | artifact 11944729636，1688746 bytes | `77ee9a3590bbb5510fb381b973dba5b6491b39c4a2d12ba466f15fdefae953aa` |
| MSI | 4567040 bytes | `3db2d45a2301e2c44e1daa696a1c9d292d80074dd837b36df8039af96a0eb4e1` |
| NSIS | 3270463 bytes | `09ace38eb9eb0a2ed7232a1198a18afba433d1ba841295e79a2f9e023a07b6af` |
| MSI 内主 EXE | 8777216 bytes，0.1.10 | `9ca21e30c9c2eee68000e1e8a03c58a42918e46d97387f96a102b7f11afb326a` |

两个 ZIP 逐项匹配 GitHub API digest/大小，归属同 run/head，CRC、路径逃逸、大小写冲突与链接检查通过。包内 environment/安装回执均绑定完整 493。下载未向跨主机重定向转发 Authorization；没有保存凭据或带签名的临时下载 URL。

runner 临时隔离环境内，MSI 与 NSIS 各自完成新装、最终 EXE 首屏及卸载，均 exit0，卸载后 EXE 移除；MSI/NSS bundle marker 各自匹配精确安装载荷。MSI 与 NSIS 的 EXE 摘要不同，按各自回执核验，不拿裸构建 EXE SHA `8a440cbb…` 代替 MSI 载荷。codeSigned=false / releaseAccepted=false。

本机仅 WindowsInstaller.OpenDatabase(mode=0) 读 MSI 数据库、导出内嵌 CAB，再以 expand 提取载荷；没有执行 msiexec 安装或 NSIS。提取目录为代码 worktree：

`test/software/candidate-493da80-38673288645/msi-readonly/opencodex-desktop.exe`

EXE 版本、大小、SHA 与 runner 的精确 MSI installedExecutableSha256 一致，二进制包含完整 493 来源字面量。EXE 未在宿主启动，不能据此声称普通/管理员 UI、活动材质、树表或更新流程在 493 通过。此前 ca26/e6 的局部原生证据保持原来源，不重标为 493。

## beta 来源的当前实读

10:59:54–56 对 493 `runtime.defaults.json`（Git blob `6d3131e4760fc5135fb32ed3ac583adc02240042`）冻结的 beta URL 作有界只读：

| 入口 | 身份 | 结果 |
| --- | --- | --- |
| GitHub releases/tags/beta API | 已认证，只保存响应体 | HTTP 404，133 bytes / SHA `010563d2cfb8f1e87f700936ce59ee3a33364da46e3db41b0049227918ece74d` |
| 配置的公开 beta/latest.json | 未认证，不跟随重定向 | HTTP 404，21 bytes / SHA `e28bf7d9fcc22cdfde0c7c8f31e3648a4847e7bda9cb69f309f24257eee3dd41` |

这是来源可用性的前置观测，不是产品检查、失败调度或 beta 6h 周期；没有发布 beta、改配置、改时钟或触发自动任务。有效来源的真实 beta 周期仍未开始，不能通过拼接此前经过时间补齐。

## 传输错误与归档

原 curl 日志读取返回非零，原方法只在工具输出中留下断言，没有保存其具体 exit/stderr；如实记录这一缺口，不补造原错误。没有残留 ZIP。随后独立 .NET 读取亦失败，状态/错误类型保留，仍无部分 ZIP。第三个有界 urllib 读取保留两个 URLError 后成功取得同终态 run 的原始 ZIP；禁用自动重定向，验证 HTTPS 目标再创建不含 Authorization 的请求，证书校验保持开启。

保存原第一次日志方法，修改后的日志解析器仅复用已核对大小/摘要的 urllib ZIP，不覆盖失败原件或重跑远端。原始 ZIP、20 份阶段日志、回执/原始字节副本、只读 MSI 方法与结果、来源补核及全部观察纳入独立 94 项清单。

证据：`.adg/evidence/OCX-0110-20261015-WIN-493-CANDIDATE/`；`manifest.json` / `archive-verification.json` 与 `verify-candidate.py` 通过。该 verifier 校验本段精确来源与制品，不是 TASK required_check 或版本验收通过。

```powershell
python -X utf8 .adg/evidence/OCX-0110-20261015-WIN-493-CANDIDATE/verify-candidate.py
```

重验仅运行此离线 verifier；不要重新执行 dispatch、下载、提取或业务启动方法来覆盖已有原件。

## 保护与下一步

11:10:40 宿主 native=0，日常 EXE、旧隔离 preferences/data-root/policy/schedule 和原 TASK 摘要均保持；代码 worktree 干净。没有安装器执行、前台输入、账户/Profile 创建、第三方生命周期脚本或 Mac 原生交接。用户原未提交删除不入本次提交。

下一前台的精确候选已就绪，仍须用户新的前台开始信号。普通/管理员完整材质窗口、托盘退出/Snap/物理 DPI、树表全矩阵、更新详情/真实进度与事件通道、最终源码真实 30min 状态退避、stable24h/beta6h 与睡眠/跨启动、官方 0.1.9 同机配对性能仍开放。独立普通测试账户不存在；安装需无日常机器级产品注册的独立 Windows 环境，新用户或更换安装目录均不能代替。

原 bd2/e6 查询失败、实际失败时间窗 ETW 缺口、Mac CPU +1.817629pp / RSS +117.008965MiB 失败及原预算保持；本轮没有重复成功对照求绿。原 TASK 3878 bytes / SHA `bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93`、revision2 / in_progress / pending 与八项 required_checks 不改。候选准入不替代完整门禁，不关闭、合并、签名、晋升或发布。
