# Windows 偏好主题回读与历史缓存导入

2026-10-12，Asia/Shanghai。关联 IMP-20 / TASK-182。本次由真实隔离偏好读取暴露问题，原始证据位于 `.adg/evidence/OCX-0110-20261012-WIN-PREFERENCES-THEME/`。TASK-182 原件仍缺失，未创建正式 RCP、改变 required checks 或关闭版本。

## 实际发现

唯一运行实例 PID21592，源码 `ee65f1158c4f0bb7c283ff09e24120a7884472d0`，EXE 摘要 `bb255ce04ecb1582483bb3982aebbbdf5b6070b17fa408e1336e99b3f6e79731`。只读调用生产 `get_preferences` 得到 theme=system、themeNeedsImport=true；实际磁盘为 schema_version=2、appearance.theme=system、没有顶层theme。读前后SHA-256均为 `8498f7cf8984c74a555202027800027fa7fd1366e0e5ce51c455aaf3ba1fdb7a`，没有修改偏好。

原 `preferences_file_has_theme` 只检查顶层theme；保存后的 schema2 appearance 域被误认作需要历史导入。该观察只证明错误DTO标志，没有声称用户真实数据已被覆盖。

同时检查前端发现，load先apply后才判断/读取localStorage历史缓存；apply本身会写缓存，导致：

1. 真实历史缓存先被当前默认值覆盖，随后导入已经不是原历史选择。
2. 原本不存在的缓存被默认投影创建，随后触发本不需要的save。
3. 一次性save返回失败后，load继续清error，使持久化失败被隐藏，原缓存也失去重试依据。

上述三个行为在原store上用新增契约回归复现：12项中9通过/3失败，原失败日志保留。

## 修复与验证

提交 `cf71ea4f84b82b2e4c1bb3a15de88fd5815dc066` 已推送 `feature/0.1.10-preferences-theme`，基于 main@ee65f11，尚未并入main：

- 后端识别appearance.theme和旧扁平theme。新增真实临时文件测试，覆盖system/light/dark、当前嵌套/旧扁平、缺失/损坏文档、只读不改字节；加载损坏文档仍拒绝。
- 前端在apply前捕获历史缓存；有真实历史待导入时不先覆盖缓存；无缓存不制造自导入。一次性save失败保持错误和旧缓存，后续可重试；已有持久主题直接成为投影。
- 前端新增四项行为回归，不降低其他测试：显式持久主题优先、真实历史值捕获、空缓存零额外save、失败保持缓存/error。

本机修复后定向12/12通过；完整UI95文件/480项通过；typecheck、生产构建及Rust修改文件fmt通过。jsdom canvas警告与四条既有动态导入分块警告保留。Rust编译/磁盘回归及修复后真实原生DTO仍待精确CI与制品，不能用本机fmt代替。

CI `38211754315` 和 Windows候选 `38211818636` 已启动，结果按精确SHA回写。工作目录补丁e1a1aaa的候选38210653937已completed/success，普通CI38210513009仍在运行；最终候选回执单独保留。不能把两条特性分支的通过结果混记为已集成main的门禁。

## 未改变的授权与门禁

脚本执行二次确认未收到，allowScripts仍false；真实2.82.0安装停在scripts_required。本轮只做已授权的代码/只读复验，未借主题问题绕过脚本门禁，未复制或读取真实用户偏好载荷。

当前RDP session2仍断开，原TASK仍缺失。管理员/其它物理DPI/安装与卸载、完整更新/事件、状态错误实际30min、同机0.1.9配对性能及真实stable24h/beta6h仍待验。此前性能failed、原预算和八项required checks未整体闭合的结论保持；IMP in_progress，不晋升核心事实、不改日常安装、不迁移真实数据、不关闭或发布。
