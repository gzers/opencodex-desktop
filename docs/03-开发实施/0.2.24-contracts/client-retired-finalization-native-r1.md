# 已退休根显式收尾的原包定向验证 r1

revision: 1；2026-11-13；IMP-34 P24-04；干净maincbc6174de17ef658f34d361ee4a6c734bff8460c；专用显式恢复诊断通过，accepted=false，非正常双轮／固定候选／整版验收。承接 [功能及回归](client-retired-finalization-r1.md)。

| 动作／事实 | 原门槛与本轮结果 |
|---|---|
| 官方2.99 Start／同连接签名Status | 7451ms<30s、238ms<5s |
| 原Stop | 5204ms，真实service/launcher正常exit0、4个监听缺席及root move/readback后，测试专用点刻意中断metadata close；动作仍失败，pending8 |
| 新的用户确认收尾 | 2694ms<独立原stop10恢复预算，写入RootAbsent9 |
| 重复同一确切确认 | 2691ms<独立原stop10恢复预算，返回同样RootAbsent9，无新增revision |
| 完整program扫描后currentGrok漂移 | 单独测试注入后拒绝，合成变化原字节保留 |

该中断仅cfgtest／显式flag／ownedProfile路径使用，未把生产Stop门槛降为通过。新恢复证明执行真实当前OS缺席／完整程序来源／归档证据／双stable锁获取释放，独立于旧正常Stop。旧动作的result=null／client_original_restore_pending、canStartfalse／recoveryRequiredtrue明确留存，新的restored结果不能回填为旧动作成功。

32个关键源及干净commit/tree、实际release库宿主b22eb8b7e75ede414d6002fbd4486c9ccb55dd90f7f7d4d6372853a2b1929eb7核对，构建6m42s。官方2.99／descriptor／bundle沿原摘要，内核沙箱实证dailyHOME／外部网络拒绝、loopback可用。开跑前假设后唯一一轮exact用例，exit0／专用marker与原断言通过，无重试或跳过错误。本轮不是已通过正常双轮的重复执行，不声称验证两代或旧19→20实机路径；fixture已覆盖旧close到期pending追加历史保持。

## 字节和状态证据

原pending8与closed9除revision/state/completion外完全一致；source7、retirementPlan、refs及restorePlan保持。数据库最终9个canonical事件与报告逐项一致；intent／published／Original／Ready／PreStop／Stopped／scope／Live／artifact Plan／retirement Plan／Retire intent/readback完整摘要链离线重验，closed completion来自原现存readback，未生成替代原件。

程序扫描后重新核当前事实的P1用实际专用测试证实：首次收尾及相同确认重复都成功后，在第三次重复的完整image scan之后创建合成currentGrok配置，之后的current核验拒绝，文件内容原样保留。该变更发生在此前两次完成之后，不能把其后的current状态当作继续可启动事实。

源字节P2也已实证：before、after-retired-finalize、after-retired-duplicate三份独立持久快照覆盖7个tracked路径的存在性／SHA/Base64并逐字相等，两张alltrue表留存；snapshot均先于有意Grok漂移。最终rawProfile含该后继漂移文件，离线确认精确为owned-test-drift-after-image=1，不是用户数据。

## 清理和验收范围

证据 [.adg/evidence/0.2.24/client-retired-finalization-native-r1](../../../.adg/evidence/0.2.24/client-retired-finalization-native-r1/manifest.json) 含开跑前假设、完整source/build/binary、所有压缩原日志和专用回执、9events／51原文件字节及独立verifier。各源／Frame／Plan/readback原字节逐项验证，不依赖测试总exit0单独放行。

精确新3协调对象先留字节、核first-seen identity／exclusive重获后清理，globalL原inode／保护／bytes保持释放，未重建或还原。tracked PID+birth／自有exe全缺席、导出primary port独立内核TCP快照缺席；51原文件逐字核对压缩归档后清Profile／程序副本／测试exe，全部诊断材料再压缩解压逐字比较后清证据临时根，unitTMP为空、工程target保留。未修改日常配置／Shell／安装app／官方包，没有真实WebDAV或模型请求。

本轮关闭的是“已经退休并有完整现存读回”的显式metadata收尾与确切重复确认定向验证。根尚未移动的continue/rollback、缺Ready或阶段证据、创建根取消、一般客户端图路径／完整进程集合及活跃刷新、NAME、复杂代理、WindowsW1W2／安装CLI更新卸载接线、官方交付verifier和固定app/DMG/MSI/NSIS仍在IMP34原范围。先前正常双轮通过及所有旧失败证据均不变；未发布，目标active。
