# 0.2.20 实施证据（持续更新）

TASK-UI020-20261102保持active。代码分支codex/0.2.20-prototype-alignment，基线cb4c9c9b7；采用原型cfbd90c4b+1da4cabca。DMD-UI-020、IMP-28与0.2.20-bugs.md第9节持有需求和实施状态。

本目录保留失败与后续轮次；checkpoint前缀日志是执行时工作区局部结果，之后仍有开发，不能作为最终整版通过证据。direct-red2证明旧流程产生草稿；direct-green/backup-e2e验证普通导入实际落盘、保留Key与范围外字段、同操作重放及范围恢复。convergence-red文件名表示最初预期反例驱动，但该轮实际通过；以日志真实退出结果为准，不冒称失败。convergence-time是新增时间分离改动后隔离双端流程的一轮通过。flows-test2含31条组件/重基准行为通过。

全部运行使用临时配置/内存transport/IPC模拟；没有日常真实同步，没有日常配置变更。日志中的SECRET/password值仅测试夹具。原生安全库、完整视觉/入口、Windows、正式包/安装/签名/OTA均没有由这些局部结果替代。

当前正式required_checks均未完成；尚未发布0.2.20。完整测试轮次仍在进行，结果会继续追加。


## 追加的真实链路和全局回归

- ocx020-native-secure-store.log：macOS原生安全库在sandbox服务随机账号上写/读/关闭加密保留口令/轮换，结束删除自有账号；通过1项。不是Windows证据。
- ocx020-owned-https2.log：真实reqwest校验临时CA签发的localhost服务证书，2项实际HTTPS条件提交/竞争拒绝/下载与直接落盘通过；首轮CA作为叶证书被拒绝保留在owned-https.log，不放宽产品TLS。
- ocx020-real-writer-webkit3：47项浏览器连生产Rust配置写入模块的隔离流程通过，普通导入已在模型页保存前落盘；保留未保存标签和Key，下一次保存保持导入结果。
- Chromium与WebKit基础均有宽窄实屏；三档/亮暗/50/100/200%的648项矩阵Chromium通过。WebKit648项断言跑完但冻结原型出现1条ResizeObserver异常，保持失败，未过滤；后续整版仍须复跑。
- 全应用旧audit在v0.2.19和当前工作区均复现411项中的同10失败，报告保留；迁移旧设置路由、选择器/通知动作与运行保护夹具，装饰动画测量排除独立通知倒计时，后续报告仍待回收。基线相同不等于验收通过。
- native-first：真实macOS调试包在独立OPENCODEX_SANDBOX_ROOT运行，含合成渠道/Key/Skill/MCP；单次明文导出成功、持久结果可读，但发现21/22对象数及凭据子项差异，已登记并修正，最终代码原生回归未完成。仅停止该自有PID，保留隔离根证据。
