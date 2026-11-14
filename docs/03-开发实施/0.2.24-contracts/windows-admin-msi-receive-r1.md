# 临时管理员MSI事务接收复核

revision: 1；2026-11-14；IMP-34 P24-W1/03。精确代码0f1dc195ed2e39bbbeffd23eff05f97a78905c57、证据c84fc4a9928c72323b7458298b5e24b08785439e，已fetch并消费最新windows-dev-handoff。净差只10份已分配的Windows测试/账户/诊断/离线验证文件，无产品源/安装器变更。四笔主题按ed01402a7/3eb32a3bd/42c030f6e/6b33016ba接入main，同一净树核对；不merge旧代码历史。181原文件共826438字节按Git blob逐字复制，未覆盖原失败或并行原型。

| 项目 | 本轮可接收结论 | 限制 |
| --- | --- | --- |
| 临时管理员实际MSI安装 | exit0，per-user unmanaged查询0、machine1605，原测试包SHA及身份一致 | 用户另行授权的本机专用临时管理员/真实提升令牌场景；不是普通用户或per-machine成功 |
| 真实pending失败保留 | /x exit1603，helper exit1，准确原pending/程序卷与文件ID/摘要、PATH、MSI登记、D/H/config/哨兵/selection保持 | out01原测试把调度Doing action误当实际删除而失败，原exit和原始日志保持；判定修正同时核调度及实际执行脚本，不能擦除首次失败 |
| 同SID继续及正常卸载 | out02重核原SID/pending后显式继续Enable，无重装，真实/x exit0、产品状态-1、程序/登记/pending移除，PATH还原，D/H/config/live runtime保持 | 非固定release包/最终W2/跨SID验收 |
| 普通用户验收 | ordinaryUserAcceptance=false | 旧普通账户1625与环境缺口保持，管理员成功不替代 |
| 账户/任务/登记/PATH | 唯一新账户SID尾1040、两任务及MSI登记移除；测试SID PATH恢复，daily user/machine PATH摘要保持，输入ACL还原 | 交互桌面ACL未改；不做全机无进程声明 |
| Profile | 85文件118556840字节仍在，Loaded=true；真正NULL修正后error32，Restart Manager原证据为System PID4/Registry PID328共享hive占用 | 不杀共享系统进程、不强卸hive、不重启系统/服务，不把accountRemoved等于profileRemoved |
| Public根 | 83文件184522112字节仍在，自动审批精确删除被blocked by policy拒绝、未执行/未绕过 | 旧所有清理拒绝不重试；cleanupComplete/allAccepted=false |

原MSI316784640字节，SHA2569a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282。实际out02测试EXE92022784字节，SHA2561dcba506281e02bdbf3714779294f32752b5ff6251198bb2ede14ad5eba09622。Windows已只读重算本体；Mac只核Git原记录/执行源/清单/前后像/原日志及其摘要闭环，未传入可选大包/测试EXE，不冒本机本体重算。

新增verify-msi-evidence.py实际离线核验通过：固定1c6a0881c源码/b0e7c6d6a证据Git对象，134份原证据、133份payload681196字节；9个内存反例（改原字节/先执行删除/保留内容漂移/pending残留/丢失首错/换令牌/隐瞒清理失败/冒报全通过/路径逃逸）全拒绝。验证器exit0仅证明管理员场景原证据一致，结果明确ordinaryUserAcceptance=false、cleanupComplete=false、allAccepted=false。原build02宏语法exit101、首轮逻辑误判、DeleteProfile空串87后真正NULL/共享冲突32分开保留，不用最终成功改写。

证据.adg/evidence/0.2.24/windows-admin-msi-receive-r1含181原blob接收清单和独立接收核验；原管理器/测试Profile/清理诊断同源在windows-dev原轮次，不复制为第二事实源。本机没有重跑MSI、创建Windows账户/任务/服务或改日常配置/PATH；接收不触发旧拒绝重试。

生产接线后继见[Windows生命周期共享接口r1](windows-production-lifecycle-interface-r1.md)。它只解决真实已发现的旧维护包lifetime依赖/生产调用缺口，不把本管理员MSI成功说成普通用户及完整生命周期/恢复已过。普通用户验收、Profile/Public清理、其它IMP-34未完及固定候选前不W2/双SID/不发布的边界保持。
