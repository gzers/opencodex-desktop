# Windows 分工与原 TASK 复核

2026-10-14（UTC+8），按用户“还有windows修改本对话的目标模式，继续开工”接续 Windows 工作；Mac 负责协调后的源码修复，Windows 负责固定 SHA 制品验收。原完整目标、预算和历史失败保留，全局最多一个隔离原生验收实例。当前后台授权不包含新的前台输入。

## 原件与结论

- `.adg/evidence/OCX-0110-20261014-WIN-WORKSTREAM-CAPTION/`：12 份清单原件，冻结校验通过。现有 be7395a 实例 PID10904 / HWND6685560 的身份绑定、普通真实令牌和只读标题栏查询；未启动第二实例或发送输入。正常窗口 DPI192，按钮中心命中依次 2/9/20。最大化状态下还原按钮的真实点击仍待坐标和命中联合复验，未定位产品根因。
- `.adg/evidence/OCX-0110-20261014-WIN-TASK-SYNC/`：13 份清单原件，冻结校验通过。Windows 独立使用治理 provider 校验 Mac 在 ae6cd08 同步的原 TASK：3878 字节、SHA256 bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93；content / summary / baseline 与既有回执一致。revision2、in_progress/pending、九项验收绑定和八项 required_checks 保持，不重建任务或撤销性能失败。
- 最小化命中 2 已在原 Windows 测试的允许值 2/8 内；第一批诊断脚本仅期待 8 的局部差异已在第二批纠正，原件未覆盖。该只读差异不构成产品缺陷，也不代表真实最小化点击通过。

## 目标模式写入状态

标准目标工具读回仍为 paused，累计 tokens/time 与原 createdAt 保留；工具没有恢复或编辑目标接口。官方 CLI 协议支持 thread/goal/set，但本轮直接写入被当前 active writer 拒绝。已启动最长 600 秒、每 45 秒重试的后台等待器，使用 origin=user；只在同一原目标仍 paused 且目标/预算一致时继续，未绕过 writer 锁、修改数据库、清除目标或标记完成。

冻结快照仅证明等待器存活，尚不证明写入成功。最终状态位于代码工作树 `test/runtime/windows-goal-user-edit-20261014/status.json`，应在本轮 writer 释放后另行读回核验。不得把准备好的 objective 文本描述为已生效目标。

### 最终读回补充

旧等待器在2026-10-14 00:33:41结束，状态为 `bounded-writer-wait-ended-without-goal-change`；全部写入均被 active writer 拒绝。原件声明600秒，实际耗时630.172秒，此处更正“最长600秒”的先前描述；原快照保持不变。

按用户再次要求继续，00:48:34另做一次官方 `thread/goal/set`，仅传 `status=active, origin=user`，不传新objective或budget；仍被相同writer拒绝，随即读回原目标/paused，createdAt、tokens18453728、time63871及空tokenBudget均未变化。本轮最终宿主目标工具再读仍paused。官方接口明确新objective会重置累计用量，故不以更换/清除目标规避问题。

`.adg/evidence/OCX-0110-20261014-WIN-GOAL-READBACK/`保留旧等待器终态、单次恢复请求/读回、官方接口来源及已准备的Windows执行指令。指令尚未应用；用户已授权，无需再次授权，但宿主目标控件的恢复操作仍缺。没有新重试循环、数据库写入、writer绕过、目标关闭或“已自动续行”的声明。本轮Windows独立后台工作另见[坐标与精确CI](../20261014-Windows标题栏坐标与精确CI/README.md)，已推送c57c860；该局部完成不抵完整版本验收。

## 接续

同步 Mac 的 caption-diagnostics 工具和同 SHA CI 状态，在现有 Windows 实例做后台只读采集；下一次前台需用户新开始信号。安装隔离、双令牌/DPI矩阵、更新、备份、事件、最终源退避、真实周期和官方0.1.9同机配对全部按原契约继续。IMP/TASK 不关闭，不发布。
