# Windows剩余项本机连续处理

2026-11-09，Asia/Shanghai。用户已明确扩大授权到其余Windows未通过项的定向修复/验证，包括双SID、安装器及第18项；后继授权覆盖此前排除这些范围的限制。继续不跑core-sweep/全量、不改日常配置、不运行真实业务同步、不发布。原归属/权限/就绪/实际还原/退出及30/10/5秒预算保持，失败先保留、写假设，有新线索才修复和重验，不盲重跑。

第13项已有本机普通账户证据ocxu1-20261109-223434通过：Start15351ms/Stop及还原8727ms/原服务exit0/账户Profile清理通过；不重复它。产品源码fdf5d4db551f71cbf493dc97b428f8b9a9364a7a，sealed build42631183420/artifact13681803354/lifecycle.3保持。最终证据提交a6f1bbb408b4eef08ec611bbe711aaee1041ced7已在代理节点恢复后推送；2026-11-10再次fetch确认代码HEAD9859977d6与远端相同。所有下载继续通过127.0.0.1:3060。

## 第一轮：双普通SID服务重叠与双账户清理

旧失败来源：run42593238885/source dac30811c14d25f5bbbc60f31409d986e0edbf9d，13因启动exit1、无唯一成功回执失败；18因一个bootstrap原身份/后代观察未证实而保留账户。后继helper已增加原生保留句柄退出的正向判断，不先对该处再次打猜测补丁。最新产品已修维护包路径及Desktop等价HOME比较，单用户原链已过。

假设H-W1：两新普通账户均能用当前sealed程序在原30秒内确认自己服务，双端口不同并同时保持健康，B正常Stop后A原PID/端口及健康不变；A随后正常Stop，两边实际客户端还原和服务原句柄exit0，双账户/Profile/目录和ACL均精确清理。任一不满足即反证，不自动重跑，不把原第18项失败改成已通过。

执行方式：新增test/smoke/windows023-local-two-user.ps1，需显式-LocalWorkstation及本机已提升控制器，创建两个真实临时普通账户，真实Profile及私有安装。复用已验证单用户Child和原RuntimeCommand；原CI-only主脚本/guard不改、不伪造GITHUB_ACTIONS。test-only可选coordination在每个原动作预算之外等待对端，不改变动作预算；每方一次Start、一次正常Stop。失败只精确清理身份已证明的自有进程；账户身份或后代不明继续保留报告，ACL逆序恢复。

执行前PowerShell AST和生成Node脚本语法通过，apps及runtime policy不变；产品沿用精确sealed字节，测试脚本另记实际checkout和摘要。本轮只补重叠服务/Stop后peer保持与账户清理，不声称提升、固定端口冲突、全部IPC/凭据或安装器已过。环境和每轮源码/命令/回执/摘要独立保存。

## 后继顺序

首轮实际更新：two-sid-01因新增测试Description52超过48而前置失败，账户0/产品Start0；原失败保留。测试修复5ad443222后two-sid-02两个真实普通SID首次实际运行：A启动15973ms/正常Stop还原7245ms、B启动16992ms/正常Stop还原7864ms，端口57052/58541，停止B后A原PID端口/认证健康保持；原服务句柄均清理前exit0。两账户/Profile/目录删净、进程端口无残留、ACL恢复。该有界双普通SID服务/清理已过，提升/IPC凭据/安装器等仍待。[原四项证据、前置失败与环境](../../../.adg/evidence/0.2.23/windows-local-followthrough-20261109/README.md)。第13项待推提交a6f1bbb40已在代理恢复后成功补推；最新产品仍fdf5d4db5，测试/构建入口后继已推9ae6b14f7。

先完成H-W1并按首错修复；再独立核对真实提升令牌下启动/正常Stop及互不干扰，随后补仍缺的IPC/凭据/同SID与安装器受影响路径。安装/更新/卸载须限定新测试账户或受控隔离目标，不覆盖日常安装。源码或制品不足、网络不可达、身份未知时写明阻点，不以旧候选冒充新源验收。每轮及时本地提交并经指定代理补推；通过即停止该项追加验证。

先前三个自建编译/ZIP分段缓存目录删除被自动审批blocked by policy拒绝，不绕过，准确目录仍见上一批task-scratch-cleanup.json；扩展任务授权不等于获得绕过自动审批许可。

## 11月10日接续

控制台console-01、NSIS default/custom和隔离MSI run42649281070定向证据均已通过并归档；本机MSI1625原失败独立保留。Profile1018仍Loaded并已禁用/去管理员身份，保留清理阻点。下一步仅H-W6实际控制台/RDP并行固定端口冲突及停止peer保持，假设、精确尝试次数和限制见证据README末节；不自动关闭IMP或发布。

## 11月10日有界结果

H-W7新源cross-session-02真实控制台/RDP协议已通过，Start13399/12692ms、正常Stop及还原9621/7862ms、双原exit0/peer保持，停止追加本场景。H-W6原首错原因仍未知，日志补丁不冒因果修复；Profile1018 Loaded和旧审批缓存阻点保留。下一动作按[最终有界交接](windows-local-followthrough-result-20261110.md)，不盲重跑，不关闭整版或发布。
