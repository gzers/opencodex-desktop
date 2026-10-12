# Windows 主题与 npm 工作目录集成

2026-10-12，Asia/Shanghai。关联 IMP-20 / TASK-182。继续按同 SHA 验收；本轮仅完成集成、原支线 CI 核验和候选下载，不声明合并制品或原生主题复验已通过。证据归 `.adg/evidence/OCX-0110-20261012-WIN-THEME-INTEGRATION/`。

主题修复 `cf71ea4f84b82b2e4c1bb3a15de88fd5815dc066` 的精确 CI38211754315 / Windows候选38211818636均completed/success。Windows fmt / Clippy / 回归 / release EXE / 最终smoke及候选安装、首屏、卸载步骤全部success；feature上macOS打包条件跳过不计通过。此前本地480项是该主题补丁的前端结果，不外推为合并版本原生通过。

主题候选ZIP11730201193 SHA-256 `6ebefddfc38ada336abb70b6608fd569db1aae066aa25535edeb902dc5391cd6`；证据ZIP11729932579 `1b10a0035607c8cc93c2273117de8b0d6b318dbc89c39df50a77a8609f0dd709`，下载后摘要一致。仅保存到忽略目录，未运行该主题支线应用、不以它替代含cwd补丁的main制品。

main在 `e1a1aaa54ccc56530d5da507d0666b6ec8d2c138` 已包含通过回归的受控npm cwd补丁，主题支线基于ee65f11。按真实双亲合并到 `afff490a430a81f8c233d44928aa3681a23e91e0`，已推送main。首次合并因本机未设提交身份而停止，未生成提交；后续使用与本任务既有提交一致的命令级Codex身份完成，无冲突，不改全局Git身份。合并差异只含主题命令/store/回归三个文件，代码/文档分支归属保持。

合并main CI38213445710已启动，Windows候选38213528684已dispatch，均head_sha=afff490。尚在运行/排队，不能沿用cf71或e1结果为合并SHA整体通过，未来取该SHA的ZIP/安装后EXE/本机startup log再接续原生DTO复验。

当前唯一测试应用仍为原根e1 PID32564；其无脚本真实npm pack/install cwd已验。脚本二次确认未收到，allowScripts=false，运行来源仍未成功安装；待确认计划不因自动目标续行而获批。原TASK-182仍未同步，RDP桌面断开。管理员/物理DPI/完整安装卸载/切根/升级/全事件/30min退避、同机性能及真实周期未完成。原性能failed、预算和八项门禁保留，IMP in_progress，无正式RCP、证书、稳定核心晋升、日常替换或公开发布。
