# Windows真实GUI令牌Profile修复

源码6e312f02270f247d2f440ec369385e523a4a8043，treef669be64f862007a8b148bd2bfc99e1f71c5e528，已推codex/024-windows-native。只改Windows gui_probe；原Stop/Restart/native原件不受此改动影响，复用前轮57项及所有既有不受影响证据，不跑W2/双SID/发布。

原0c真实桌面首错raw101/gui-normal.json缺席和GUI admission refused/rawNone保持在相邻wl2-real-gui-run-20261115-r1。根因是sandbox启用后platform::home_dir返回sandbox/home，探针却用它验证原Token Profile父目录。现消费既有Windows current_user_profile原Token/GetUserProfileDirectoryW接口，继续保留exact SID/session/ordinary token/任务名/sandbox边界/未安装runtime/原源/私管道/窗口/原5秒断言。逐项失败谓词写stderr，不跟随未准入路径写文件，不因诊断授许可。

定向MSVC custom-protocol库7过/0败/4忽略/raw0/220268ms；其中私有helper由父测试在独立子进程中以原sandbox环境、不同USERPROFILE执行，验证app home与原Token Profile独立且未创建路径。它不是桌面组件验收。最终clean MSVC custom-protocol debug全部bins raw0/101991ms/lib69 warnings；99份产品输入和55份测试Rust对最终工作源/Gitblob行尾等价，32份前端1162800字节不变，非release预算或冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77505024 | c780330b94511602772e3740d82659412a1c3f5557ccf005458f4256ca3c1927 |
| ocxd | 5398528 | b80dd57317c91700aee4c77de1023f53b6da0f2c38e6760006db4b6abf8d3a67 |
| ocx | 306176 | 430e193f62cc68f96e945e381758d9c761379831cbd2fe5c5990226b770712d8 |

本轮prepare01因调用者给strict full SHA检查传短SHA而raw1，前置第14行拒绝、尚未创建任务根，原事实保留；prepare02传完整SHA后raw0，未削断言。新Public任务根只在自有对象上设controller/SYSTEM full、marenben read及out Modify，三输入摘要/脚本解析通过，prepared脚本仅将原exact任务根替换为新的exact根，其余commands/guards/budgets逐字保持；原失败根和其pin图像未改。

新入口尚待marenben原普通桌面执行，未取得结果，不计通过。

完整B1/B2/B3/O1/O2/H/S-D/P/MSI表归final-status.json。公共Start仍拒绝；真实官方副作用/NativeLease依赖独立路由声明解析器最小提取，共享StatusSource还缺同一次采样fetch/fetch_light/recovery/debounce的原5秒上下文，两个具体最小请求沿前轮路径，不擅改共享trait/HMAC/其它平台。普通MSI、非空CAP/trust环境缺口不重跑旧前提失败。

本轮创建1处新Public任务根，无账户/Profile/日常PATH配置策略改。旧失败空broker40588仍未取得原始退出码，不填0、不凭PID强杀/收养；所有原失败现场和旧自动审批拒绝保持，不重试删除。无新删除尝试，overallCleanupComplete=false；原档/ZIP不是NTFS身份安全克隆。无官方进程/业务同步/模型请求/W2/双SID/发布，allAccepted/productionExecuted/startOpen=false。
