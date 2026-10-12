# Windows0.1.9基线Known Folder边界

2026-10-12，本机普通账户令牌下只读实验。证据归 `.adg/evidence/OCX-0110-20261012-WIN-BASELINE-KNOWN-FOLDERS/`。未运行官方0.1.9、未改变其源码/身份或当前用户Profile绑定。

## 实际边界

两个短命辅助进程直接调用 `SHGetKnownFolderPath(flags=0, token=NULL)`；一个保留现有用户环境，另一个把HOME/USERPROFILE/APPDATA/LOCALAPPDATA和显式sandbox指向任务runtime根。这里只读取返回值，不调用创建/重定向API；真实用户路径仅保存规范化摘要和边界布尔值。

| Known Folder | 环境覆盖后与之前同路径 | 覆盖后在测试根内 | 等于覆盖后的环境路径 |
|---|---|---|---|
| Profile | 是 | **否** | 否 |
| Roaming AppData | 否 | 是 | 是 |
| Local AppData | 否 | 是 | 是 |

实测指出当前账户的Profile/HOME缺口；不把所有Known Folder都称为不受环境影响。不会改当前账户Profile/注册表来获得基线隔离，也不会用修订后的0.1.9二进制替代官方基线。

## 来源与证明限度

官方de6862d4源码的platform::home_dir调用dirs::home_dir，启动取该HOME用于受控发现和共享来源。读取该源码的Git blob身份保存，没有复制实现到文档分支。

下载官方dirs6.0.0及dirs-sys0.5.0原始crate，摘要与当前提交锁一致，核对Windows home_dir→known_folder_profile→上述API的符号链。原官方0.1.9源码没有已取得的依赖锁，这只是一份相关主源核查，**不反推其精确构建依赖版本**。

另以纯只读PE导入表解析核验官方0.1.9 EXE SHA626d63ad…，确有shell32.dll/SHGetKnownFolderPath导入；不执行EXE、不据导入表推定所有调用频率。联合当前账户API实验与旧源HOME路径，已确认仅环境覆盖不足以准入安全基线。实际基线运行仍需能隔离Profile及所有受控发现路径的专用Windows用户/测试环境，保持官方生产身份；环境配置与权限按原授权门禁办理。

## 保留当前周期

未启动第二桌面应用，原PID38924/监测18944实际句柄仍活，周期文件原字节保持，日常EXE SHA43e06bc3…与官方基线SHA均未变。13个原始文件集合/哈希与上述正负边界断言冻结并重验通过；这不是性能测量或新候选原生通过。

原冷/热各≥20、p95≤100ms且≤5%、10min CPU≤0.5百分点/RSS≤10MiB、主线程任务≤50ms及缓存100切页/10重启零自动查询预算保持，未缩减慢网/离线/帧/进程等范围，既有性能failed不清。TASK原件/脚本二次确认、其余安装更新/原生/事件/真实周期仍开放，IMP in_progress，不关闭或发布。
