# Mac 接续与原 TASK 同步

2026-10-14，Asia/Shanghai。远端 main@eae85322ebca3b755c034db97cbebf2ea5bf71fc、feature/0.1.10-update-center-focus@be7395acfce0ec1e36b83300b576db408f8c35bb、docs/governance-main@6bd9547c77785b4faebc8472731e3134c9aea81e 已 fetch 并核验。main 与文档工作区快进同步；PR #1 open / draft、未合并，焦点修复不在 main。

## 原文件来源

Mac 本地 `.adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json` 在本轮开始前已存在；文件系统创建与修改时间为 UTC 2026-10-10T03:16:25。3878 字节，原始 SHA256 为 bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93；内容摘要为 23908e0d312409d81886b6f0f570c39581d8f9fc413ac67b0d043102eecfc29c。治理 provider 的对象 / 摘要校验通过，content / summary / baseline 三项与既有性能、调度回执相符。原件未修改，也没有从回执重建任何字段。

此前 `.gitignore:4` 忽略 `.adg/work/`，该 TASK 从未进入 Git；这是跨端缺原件的同步缺口。此次只强制跟踪这份原 TASK、同目录单文件 `-text` 属性和本轮回执，未全局取消忽略。原件为 revision=2、in_progress / pending，八个 required_checks 保持；同步不代表验收或更新任务基线。文件时间戳仅作本地来源线索，摘要校验不扩张为签名来源证明。详见 task-provenance.json。

## 工作区与执行分工

同步前的文档 19356 个可见未提交路径和代码两个未跟踪 probe 均已记录；快进后逐个验证原有文件摘要 / 缺失状态一致。另有其他工作同时新增的两份 model-picker 证据，保留原状，不纳入本轮暂存。

按用户分工，Mac 负责源码审阅、定位、修复与后台单元 / 集成回归；Windows 负责固定 SHA 真机验收。Windows PID10904 / be7395a 仅为用户交接快照，本机没有直接重新证明其存活。Mac 不启动桌面应用或原生性能测试；切平台原生验收前须协调 Windows 结束实例并确认实例数为 0。FZ50 生命周期脚本仍未获显式许可。

后续先审阅 PR #1 与标题栏“还原”原件、离线复核 macOS 官方 0.1.9 配对失败。caption 点击坐标 / 命中未记录，尚不能归因；CPU +1.817629 百分点、平均 RSS +117.008965 MiB 失败仍保留，Windows 局部低资源样本不解除它。原预算、冷 / 热各至少 20 次、真实周期与全部未完门禁保持。无真实迁移、日常替换、自动合并、签名、发布、通道晋升或关闭证书。
