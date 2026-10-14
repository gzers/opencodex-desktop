# Windows 透明图标接入

2026-10-14（UTC+8），用户先要求效果预览、确认后提交，随后“我看了效果可以，可以继续”。采用透明应用主体90%与无底板的独立单色托盘，保留品牌云形、渐变和聚合镂空；16px档增加一像素透明安全边距。素材原件、旧交付归档和当前清单见[品牌修订](../../../../../../../04-品牌与图标素材/README.md)。

## 实现与验证

代码：`feature/0.1.10-windows-transparent-icons@bb3122d678a9c6fdbd2c8bd78fa9f3f22b955c8a`，基于Mac更新说明修复d3b83b38。草稿PR#3以更新说明分支为base，30个文件只涉及Windows图标/配置/托盘选择与回归，不改更新逻辑；main、PR#1/2未合并。

Windows独立配置选择 `icons/windows/`。托盘独立亮暗纯色PNG按 `SystemUsesLightTheme` 选色，沿已有系统主题事件刷新，且放在应用外观缓存早退之前；应用内显式主题不决定任务栏前景。macOS默认配置、PNG、ICNS和模板托盘保持。更新配置仍使用原安全边界。

本机素材红绿：旧七档ICO均被“白底板/缺透明边距”合同拒绝；新七档ICO、十四档托盘通过，生成清单27份输出摘要与代码资源一致。官方rustfmt组件及匹配DLL按官方摘要核验，格式检查通过；首轮DLL缺失0xc0000135是formatter环境准备失败，原件保留。完整Rust编译与回归由同SHA CI证明。

| 运行 | 来源 | 终态 |
| --- | --- | --- |
| push CI38499759600 | bb3122d | success，五必需job成功/main-only build skipped |
| PR CI38499860360 | 同head，PR检出路径独立 | success，五必需job成功/main-only build skipped |
| Windows候选38499860139 | bb3122d workflow_dispatch | success，锁定回归及MSI/NSIS新装/首屏/卸载 |

两候选ZIP摘要/CRC、安全条目核验通过；两份安装回执原字节一致。MSI4562944字节/SHA311ffb8badd689b365ec324eb74f6647b46297173ac773409d619131a01715b4，NSIS3264876字节/SHA2e2419337611f3e61736eb1b0173bd35964be514c3a15c6b58649ac33e5a4ff6。只读MSI/CAB提取EXE8764928字节/SHA3b5c5176139fd9da4b7c5f444c2b5e7af08d2e6df9aaacd07fc9007e6ec1e989，与runner预期补丁摘要匹配；内存归一bundle标记后匹配原构建摘要，未写出归一EXE。PE资源按字节解析，全部七档内嵌图标像素与源码一致，两个单色托盘PNG均在EXE内。

## 本机顺序切换

09:48旧be实例只做临时图标预览，用户确认后10:11:06恢复原图标/托盘，helper正常退出。10:30:51原PID10904与监督器32492结束，本机原生实例数确认0。10:31:15从上述只读提取EXE启动新唯一实例，PID39680 / supervisor44136 / exec91561 / HWND2625640，普通真实令牌false/type3，创建UTC2026-10-14T02:31:15.1043658Z；新进程日志匹配bb3122d，CDP57640。继续使用原隔离根，不重置偏好或schedule，auto-check=false。

[真实新窗口原始截图](../../../../../../../../../.adg/evidence/OCX-0110-20261014-WIN-ICON-IMPLEMENTATION/native-bb3122d-window.jpg)仅包含原生标题栏与应用窗口，实际观察标题栏无底板；不是临时WM_SETICON预览。任务栏/托盘完整像素、系统亮暗主题物理切换、高对比/DPI矩阵仍待完整验收。日常EXE及隔离preferences/data-root/backup-policy/schedule摘要前后相同。

`.adg/evidence/OCX-0110-20261014-WIN-ICON-IMPLEMENTATION/`含原始阶段失败、完整CI/API、候选摘要、资源解析、原生切换和前后保护证明。`verify-evidence.py`无参数重验冻结清单。安装器未在用户宿主执行：ALLUSERS1和共享UpgradeCode仍不能用另目录消除安装冲突。

## 保留边界

本次完成图标实现、用户视觉确认、精确CI/候选身份和新源标题栏子集。其余原生/安装数据/更新说明进度/备份树表/事件/最终源退避周期/官方019配对及既有macOS performance failed、原预算保留；TASKrevision2/in_progress/pending和空正式验收不变，不以局部通过关闭版本。FZ50脚本许可仍缺，无真实数据迁移、日常版替换、自动合并、签名、发布或通道晋升。Mac原生仍需Windows实例归零后交接。
