# Windows 焦点分支CI失败与隔离诊断

焦点特性源码`d975b2351920bc027c931c8135b5605fde7a46c2`的CI38349316806于2026-10-13 13:12:56（北京时间）核验为completed/failure。原13:02在途观察保持，此处记录后续真实终态。原件归`.adg/evidence/OCX-0110-20261013-WIN-UPDATE-FOCUS-CI-FAILURE/`。main仍eae8532，未因前端局部成功集成特性或触发候选。

## 完整失败事实

frontend、Linux backend、release-tools及windows-smoke-evidence均success。Windows job中的前端构建、fmt和Clippy success，但实际Windows backend regression失败，主库502 passed/3 failed/4 ignored；Windows release EXE构建、最终EXE smoke及总build均skipped，不能当构建验收。

| 失败版本探针夹具 | CI实际结果 |
|---|---|
| success | 2.1725273s，Timeout |
| nonzero | 2.174039s，Timeout |
| timeout | 2.1548715s，Timeout |

三个observer都在取得后代保留句柄前报“fixture descendant did not start”；元数据进程树四项通过。当前日志不能定位是Node/进程启动、夹具调度、宿主负载或产品路径问题，不能因来源相同就宣称只是CI噪声。

源码diff仅UpdateCenter.vue及其前端回归；Windows夹具、官方版本探针和OwnedQueryJob三个Git blob与eae完全一致。这个比较只证明没有改这些文件，不证明不存在全库/平台回归，也不直接解释失败。原失败断言、2秒生产预算、4秒夹具收尾上限、原并发及完整门禁均未改。

## 同run测试EXE与Node对照

下载run制品11790820992，ZIP SHA8da4ce67…匹配已认证API digest、CRC通过；导出的测试EXE SHA e5aaed4c…独立匹配CI摘要，是真实失败run的EXE，不重新编译或替换产品来试验。

仅在独立HOME/AppData/temp测试根执行一次串行及一次7线程的`windows_tests::`过滤样本，真实测试令牌false/type3、清空后白名单环境、无凭据继承或第二桌面。这两个样本使用本地Node24.19.0；之后从Node官方24.21.0发布目录读取SHASUMS，精确Windows x64 ZIP SHA158f7685…匹配，提取测试目录内node.exe并实际读回24.21.0，按同样串行/7线程各测一次。

| 实际本机对照 | 结果 |
|---|---|
| 原CI EXE + Node24.19.0，1线程 | 9 passed/0 failed/0 ignored，500 filtered |
| 原CI EXE + Node24.19.0，7线程 | 9 passed/0 failed/0 ignored，500 filtered |
| 原CI EXE + 精确Node24.21.0，1线程 | 9 passed/0 failed/0 ignored，500 filtered |
| 原CI EXE + 精确Node24.21.0，7线程 | 9 passed/0 failed/0 ignored，500 filtered |

每组9项实际为7个查询/探针进程树加2个Windows路径测试；早期进展提到7/7仅指进程树场景，原始过滤日志完整保留9项，不能删去额外计数。超时场景仍约2.00秒，成功/非零有真实生产分类；没有外部看门狗替生产清理，没有扩大预算、改夹具源码、序列化CI或删除断言来凑通过。

精确Node版本仍不复现，因此版本差异不能单独解释失败。四个本机定向对照不等于完整CI库/工作区负载复现，不能覆盖CI failure或声称其故障已修复。没有重新请求CI求绿，原因继续未隔离；须诊断完整Windows失败后才集成main。

## 原周期保持与归档

原eae普通桌面PID37276/监督器24300/监测3920实际句柄、创建时间和唯一实例保持。偏好、根绑定、策略和update-schedule原字节不变，调度SHA6e51fe41…，日常EXE SHA43e06bc3…保持；真实截止仍2026-10-13 22:55:39/40（北京时间）。测试Node未进入日常位置、不执行包生命周期脚本，后台诊断没有再抢前台。

只读观察未发现这批夹具Node残留，范围只限已知fixture路径/命令，不推断系统全进程或网络计数。首次生命周期收尾调用误写历史路径/省略绝对路径，PowerShell将`.adg`当模块而拒绝；助手未执行，随后用真实存在的20261012 helper全路径取得原句柄，不重跑任何probe。

31份原件的全集合/字节哈希、真实CI failure及制品身份、四组本机结果、精确Node来源、后端blob一致及原周期保持断言冻结重验通过。归档passed不代表CI或修复制品通过；主库ignored保留、完整原生/配对性能/真实周期及TASK182/脚本许可/安全环境等仍开放。IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。
