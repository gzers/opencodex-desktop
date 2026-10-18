# PR7 集成CI与发布准备核验

2026-10-18，PR7基线main@eae85322e，head8e69ff6db，首次PR事件运行39095241645/attempt1已completed/success。五份必需作业的原日志actual checkout均为合成提交48cff7dd7079e6872e804709cd1da6c87ab1e126。

| 检查 | 真实结果 |
| --- | --- |
| Mac后端 | 35组，938passed/0failed/6ignored，格式和严格Clippy成功 |
| Windows后端 | 35组，667passed/0failed/7ignored，严格Clippy、最终release EXE构建及首屏冒烟成功 |
| 前端 | 100文件/542passed，类型和构建成功，CDP协议回归成功 |
| 发布工具与Windows方法 | 对应作业均成功 |
| main-only build | 按事件正常skipped，不计打包/发布资产完成 |

这里只沿同一次运行取得终态和日志，没有重派CI、重建候选或重复Windows现场。初次只读日志下载被gh的ANSI输出保护拒绝，随后以其显式原始输出选项捕获为数据文件，不在终端渲染控制字符；不影响CI或产品执行。早期in_progress快照保留，不改写为当时已经完成。

发布预检位于 `.adg/evidence/OCX-0110-20261018-RELEASE-PREP/`：前后端版本均0.1.10，HEAD/0.1.9/0.1.7解码公钥SHA相同，CHANGELOG草案补丁git apply --check成功但未应用；Secret只读名字/更新时间，未读私钥；当前公开0.1.9资产存在。本轮只准备提案，未创建tag/Release、签名、合并或晋升。

初版发布提案冻结身份之后，另补共享README旧事实纠错清单；本目录保存去掉该新增章节即可精确匹配原SHA的旧正文和当前提案身份，明确两个时点，不改初版封存清单。其余三个提案摘要与初版相同。核心事件投影覆盖81事件/39任务/27策略，登记不是行为验收。

`ci/`与copies.json保存12份来源无损复制；manifest按路径/大小/SHA封存。原CI/性能失败、CPU/RSS接受范围、DPI取消、未验真实更新/事件、原生组合和调度/外部条件继续沿原清单。TASK-182八检查没有整体核销，main仍eae，尚未请求最终发布确认，不提前关闭。
