# 实际验证与失败历史

- 改前：先更改两个原来要求人工 review 的行为断言，再对原生产源码运行 manual_review。两项均失败：File import default false 返回 capture_assets_review_required；mapped false 构造拒绝。原日志保留。
- 改后新 case：default false 实际 FileTransferOwner 导出/导入 Skill+MCP，NoCredentials 三接口 panic，未选秘密不读取；多源未选/错误来源/错误 definitionDigest/缺 Skill/取消/未选 MCP/preview 后目标修改/软链反例仍拒或不应用。实际 receiver 回执落盘。
- 新 actual LocalApplication case：false request + 实际 ReceiveFileHost、portable_manager、portable_assets；模型回执真实存在，偏好/Skill/MCP 实际写入。源变更时首轮拒绝且模型回执不存在、Skill 原字节不变；恢复原源后按明确范围完成，不用 fakeApplied。
- 原旧测试完整保留其余来源/范围保护断言。mapped case 仍测试 owner-issued binding、scope 扩围、伪造摘要拒绝；仅将“必须手工 review”断言改为 false 可构造。文件 import case 仍核秘密隔离、实际内容采用。
- 全 file_transfer_v2 模块（含 output）39 项通过；全 sync_local_v2 模块（含范围恢复、时钟与原 None 只读现状测试）46 项通过。两组不重叠，共85项；新 false 2项包含于这85项，不能另加为87。
- 普通应用 crate offline strict Clippy --lib --tests --no-deps -- -D warnings 通过，rustfmt 和 git diff check 通过。没有 #[ignore]、降低 warning、全局 allow 或删旧反例。
- 新测试首轮编译缺少 atomic_write import，局部补 import 后通过，失败原日志保留。另一个宽过滤 review 命令在执行目标集之前由本线中止以避免误匹配所有 preview 长案例，随后改为 precise false_review_ 和全部两个受影响模块；中止不记测试失败或通过。

本次基线合并中与 Main 的单一 add/add 冲突为范围恢复测试目录：采纳 Main test_sandbox::temporary_parent()，保留本线原 None 现状测试。该合并不是本片产品变更；最终修复 commit 仅批准四文件。

没有重新操作原生 app；只读查看并归档 Main 已有 sync-file.png。未修改 UI/docs/Schema、全局引用行为、日常配置或真实 WebDAV；没有发布。
