# 标准双SID集成core：完整失败表

Run42593238885/job129079673708/source dac30811c14d25f5bbbc60f31409d986e0edbf9d；523.737秒/8分44秒；6pass/2fail，**accepted:false**，H04被反证。

| 项目 | 结果 | 首错 |
| --- | --- | --- |
| 01 hypothesis-and-inputs | pass |  |
| 11 two-real-sids | pass |  |
| 12 private-bundle-supply | pass |  |
| 13 runtime-readiness-and-ports | fail | readiness: user1: no unique successful actual runtime start receipt; environment-ready is not service-ready; ports: user1: runtime_start_not_healthy; stage=wait-owned-health |
| 15 two-user-paths | pass |  |
| 16 same-user-ipc | pass |  |
| 17 cross-user-ipc | pass |  |
| 18 account-and-process-cleanup | fail | Two exact real account/Profile removals not proved |

启动实际是exit_code=1/lifecycle_start_exit，未发布runtime-port；不是上一轮31秒迟到健康。实际app日志duration25848ms，测试共享30秒后保持失败；没有成功start回执，未到正常Stop/peer协议，所以不能声称新的Stop消费者通过。错误尾部当时仅固定分类，缺少lifetime拒绝阶段，具体系统原因尚未证实。

清理：user1新SID…1003/Profile已移除；user2…1004的bootstrap身份/后代查询阶段未证明，bootstrap-identity-retained，最终账户/Profile保留。desktopAclRestored/evidenceAclRestored/sealedCandidateUnchanged均true，后续tracked stop回执不补判整项清理。用户不需要手工清理该GitHub临时VM；不得把runner销毁当作成功清理证明。原失败保持。

根因/修复边界：父HasExited后bootstrap可能在Assert-TwoUserLive前退出，需用先前保留的原生同句柄作正向退出判定，仍继续原parent-chain所有子进程精确身份枚举；未读到或仍活未知继续拒绝。新增固定字段计时/错误类别/原bootstrap退出回执，不清除旧unknown。启动侧仅补固定阶段和Win32数字码，不输出raw错误/路径/SID/凭据；没有证据前不改ACL/Job或以提权兜底。

原artifact1391782字节/SHA25635273aa4fe76b01c41ab59b21ec924d7f7ca27a6bb2e845896c8bc9d046a8fce和所有成员摘要已核验。与先前6组技术原型/producer局部通过分开，实际当前用户启动/完整Stop/配置还原/peer保护及整版发布继续未完。
