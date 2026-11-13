# MSI隔离环境盘点接收

revision: 1；2026-11-14；归IMP-34 P24-W1。接收代码引用f66d9d59f608d13920db58905682ac3c6435860c、证据24a4a467368bddc55979fd01d969e6cd7f9bbc92。不是卸载补验通过，也没有新的产品缺陷结论。

- Windows代码线与main fd2538e4d树完全一致，tree b276dd036de49133e9d3f3c47149300cded5b113；没有实现/安装器/测试变更，不再merge重复历史或创建空代码提交。
- 21份原交接文件共56573字节按Git blob逐字接收；10份只读提升盘点原件逐项SHA/长度核验。实际受管开发MSI在Windows重新计算本体SHA及316784640字节，与原9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282一致；Mac未下载重算本体。
- 正常卸载和失败保留均not-executed，msiExecutions=0、environmentReady=false、allAccepted=false；不重跑已过终端/原官方消费者，没有新账户、VM、安装/卸载进程或配置/PATH/机器策略写入。
- 在本机可达的只读盘点范围内：Windows Server2025物理主机；提升后Hyper-V registeredVMs为0，Docker default为Linux，无已安装WSL发行版/已配置PowerShell远程会话或SSH测试目标。初始权限拒绝和后续只读成功分别保留；不宣称全网无机器。
- **当前HKLM/HKCU Installer策略键的32/64视图均缺席**，不能继续将上轮有效DisableMsi=1说成当前注册表仍明确为1；同样不能将键缺席说成已证明MSI可安装。旧out03安装日志的1625/有效DisableMsi=1作为原时点失败保持。本轮未在非隔离日常主机再次探测安装，也未绕过旧策略门。
- 新Public盘点根10文件5968字节仍在，自动审批以blocked by policy拒绝精确非递归删除，命令未执行/未换机制绕过。原历史大产物拒绝未重试；已无本轮账户/Profile/MSI/VM进程待清。Mac只接收Git文件，无新运行夹具。

证据.adg/evidence/0.2.24/msi-environment-receive-20261114-r1含原blob清单、独立verify_all.py与manifest。小型接收不重跑产品测试，未修改产品源或声明整版通过。

后继需要用户提供或明确授权准备一个允许普通用户per-user MSI的隔离Windows环境，具体入口/连接由Windows原对话使用；既有MSI-only合同的安装前提、真实pending失败保留、继续Enable后正常卸载及D/H/live runtime保留仍是两项最终证明。未获环境前，不再给同一主机重复1625/空盘点轮次；不在本机自行创建VM/改机器策略/提高测试令牌来替代。若用户决定准备新隔离VM，另明确可用镜像/资源和创建范围后执行。

本缺口仅阻塞MSI对应验收和整版收口；不把整个版本其它未完成工作说成已解决。W1生产生命周期/固定候选/W2、发行装配/最终sidecar来源、根中断/一般客户端/NAME/复杂代理等IMP-34原范围保持，不发布。
