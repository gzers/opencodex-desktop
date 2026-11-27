# TEAM84 普通桌面启动夹具首错及修复

marenben 在交互会话 3 通过原启动器的 SID 与普通令牌检查。Full 第一次停在第 70 行的 Join-Path，错误为 CannotConvertArgument。Windows PowerShell 5.1 把 ConvertFrom-Json 返回的数组作为一个管道项；外层数组表达式使清单变成一项，访问 path 得到 System.Object[]。错误发生在复制第一项和启动候选 EXE 之前。这是验收夹具首错，普通桌面产品首错仍未取得。[原始回执](raw/full-r1-launch-failed.json)

r2 将解析结果先存入变量，再把各清单项送入筛选。复用 r1 的只读载荷并校验清单 SHA-256，从新 Public 入口将文件复制到 marenben 的新 r2 根。Windows PowerShell 5.1 复现原绑定错误，解析并直接执行新入口的清单读取函数，逐项核验 10 个文件的路径、长度和摘要。Missing 仅省略随包运行归档，余 9 项完全一致。[原始核验](raw/launcher-r2-powershell51.json)

候选仍为 e663369226337c9a9cad594ccd3cafd1719c3711，Windows 制品 15187611477。r2 启动器 C:/Users/Public/ocx-team84-e66336922-r2/run-team84.ps1，SHA-256 fed864644478bc8faeb92568c31d7fdca60dbbcb2b8e98b1f046843fafb4ddf2。原 r1 启动器、失败根和回执保留。等待 marenben 以 Full 模式启动 r2，首次 Status 和产品结果待取证。正向启停仍需可信空闲，继续使用 30／5／10 秒。断网桌面按 TEAM84 跳过。accepted:false。
