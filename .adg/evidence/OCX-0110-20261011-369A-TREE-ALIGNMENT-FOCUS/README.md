# 369ab12d 树表首行对齐与刷新焦点证据

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182，生产源码 `main@369ab12da55448fbfdc4d0c9bf03970b56fd6c27`。这是局部验证记录，不是整体验收证书。

父提交的生产组件在两行名称下，选择框、箭头和图标中心低于首行 11 CSS px。修复后统一 22px 首行槽位，16px 选择框居中于槽位；浅深色、300 / 420px 名称列的浏览器几何偏差为 0。夹具导入真实 UiTreeTable、生产 token 与 base CSS，不调用后端、不读取用户文件。

扩展焦点用例的修复前结果为 8 passed / 6 failed；中间修复结果为 14 passed；最终新增选择框方向键用例后，全前端 94 文件 / 466 项通过，其中树表 15 项。失败原件保留，不用最终测试覆盖。基线回归测试当时已本地扩展，并非父提交原有测试全集，来源见 source-provenance.json。生产构建和 diff check 退出 0，原始 stdout / stderr 与命令信息保留。jsdom canvas 未实现提示及既有动态导入构建警告未记为失败。

刷新保留混合态 INPUT 焦点；方向键转至下一可见行，选择 / 展开不变。行或选择框移除、禁用时退至同一行主控件、可见祖先或首行；外部焦点不抢夺，错误状态退至重试入口。这些分支由定向测试覆盖，浏览器操作仅实测混合态刷新与方向键。

PNG 为浏览器截图，geometry / focus / console JSON 是只读 DOM 观察。它们不替代 macOS 原生全密度 / 画质矩阵或 Windows 物理 DPI / 整窗合成验收。旧 03e0 候选不含此修复；新 SHA 的候选需要另取证。临时 QA 标签和精确 5179 Vite PID4947 已关闭。

大日志以 raw.log.gz 保留原始字节，summary.txt 仅筛选失败名称与结果。checksums.sha256 校验留存文件，source-provenance.json 另列原始日志摘要和生产文件摘要。

## 同源码候选后续状态

CI 38124355301 五项全部成功。最新 macOS 候选已构建，原生局部键盘 / 展开保留 / 浅深色已观察，但原件正式签名失败，完整矩阵仍待验；见 ../OCX-0110-20261011-369A-MACOS-TREE-CANDIDATE/。Windows 单次候选 38127059287 已通过锁定依赖回归、正在构建安装器，未完成安装与资产核验，见 ../OCX-0110-20261011-369A-CI-CANDIDATE-DISPATCH/。

构建 stderr 的原始 1772 bytes 保存在 production-build.stderr.raw.log.gz；同名 .log 只移除末尾空行作为可读视图。source-provenance.json 分别登记原件与视图摘要，构建警告和通过结论未改变。
