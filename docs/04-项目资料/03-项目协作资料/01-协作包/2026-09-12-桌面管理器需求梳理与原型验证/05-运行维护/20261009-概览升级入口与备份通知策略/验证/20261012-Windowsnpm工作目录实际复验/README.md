# Windows npm 工作目录实际复验

2026-10-12，Asia/Shanghai。关联 IMP-20 / TASK-182。证据归 `.adg/evidence/OCX-0110-20261012-WIN-NPM-CWD-FINAL/`。脚本二次确认仍未收到，未执行任何启用脚本的安装，本轮只验证无脚本流程的 cwd 合同。

`e1a1aaa54ccc56530d5da507d0666b6ec8d2c138` 精确特性 CI38210513009与Windows候选38210653937均completed/success。原始候选job日志以无损gzip归档，明确包含实际Windows测试：canonical npm目标/cache与cwd、普通npm命令环境、独立沙箱HOME/管道，三项均ok。fmt / Clippy / 完整回归及最终EXE/安装器步骤结果见精确回执；feature条件跳过的macOS bundle不计通过。

该提交已快进并推送main，main集成CI38212329011仍在运行。候选ZIP11729581121摘要 `ec5524e9d1d5a180785f5c2b031fd3d6742d349c566712a0a6f30fa16bfb6ba9`，证据ZIP11729521373摘要 `c3b6ee9f7a25d60fa1f75041885dafd0b698890c6f7455ddab1ca86e3fc7f69b`，均与下载元数据一致。MSI摘要 `b94c2b76ad5a794bee10b89d9fc247e145b49e98777e872fe9ca12750f71e08f`；提取主EXE摘要 `e3dd402aff7e9e194940b013cbec017de28b0b20c5e42fa138937f0de0fe3fe9` / 0.1.10，与CI安装后的EXE一致。本机没有运行管理器安装器。

确认旧ee进程退出后，在原容器/data根启动唯一e1候选PID32564，实际普通令牌false/type3、源码启动日志匹配。仍使用普通权限和受控测试HOME，不并行开启另一实例，也不替换日常安装。

真实生产install_runtime无脚本重跑精确官方2.82.0：06:27:04.768Z开始、06:27:20.809Z终态。npm pack和install均exit0，进度输出“added 102 packages in 12s”；随后版本探针返回原scripts_required，来源仍unresolved，监听已解除，完整安装/官方升级保持未验收。

原npm日志直接证明：

- install cwd与`--prefix`一致，位于测试数据根的 `runtime/opencodex.new-1cf562917a10`。
- pack cwd与`--pack-destination`一致，位于上述临时前缀的 `.downloads`。
- 两项参数都是安全简化后的普通绝对路径，无`\\?\`输入，install实际传入`--ignore-scripts`。

初次日志解析未将npm argv的JSON转义解码，比较失败；只修正证据解析后再次核验，原日志未改，路径身份检查未放宽。真实程序cwd修复由上述原始日志支持，不能扩展为入口可读、脚本安全隔离或成功升级。

主题修复cf71ea4仍在特性分支验证，不能混记为e1制品。脚本开启的待确认动作仍是先前记录的一次精确2.82.0安装，审批计划保持approvalProvided=false；没有因goal自动继续而批准脚本。

原TASK、RDP桌面、管理员/完整物理DPI/安装卸载、切根成功、更新详情/真实进度/回读、30min错误退避、同机0.1.9性能和真实周期仍未全部闭合。原性能failed和预算不变，IMP in_progress，无正式RCP/证书、核心晋升、日常替换、真实数据迁移、tag或公开发布。
