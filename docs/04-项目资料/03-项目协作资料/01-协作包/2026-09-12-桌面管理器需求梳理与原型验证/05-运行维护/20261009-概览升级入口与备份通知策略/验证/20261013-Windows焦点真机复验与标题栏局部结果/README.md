# Windows 焦点真机复验与标题栏局部结果

be7395a候选的原生焦点回归子集已通过：管理器和面板详情中真实键盘聚焦“全部更新”，Enter返回后焦点仍位于打开的dialog，未外逸到BODY；Shift+Tab/Tab边界循环及Escape关闭归还原入口通过。此结果只闭合本次观察的焦点缺陷复验，不代表完整原生更新/发布门禁通过。

本轮复用原唯一app10904/监督器32492/exec42581/HWND6685560、EXE3a26098e…，没有重启或第二实例。用户“恢复了连接”后，23:19:46（北京时间）query session确认Active；新的前台窗口到23:30，最后实际输入23:27:46.307，之后仅后台保存/校验。

原件归`.adg/evidence/OCX-0110-20261013-WIN-FOCUS-NATIVE-RESUME/`，48份全集合/字节摘要冻结重验通过。

## 实际焦点路径

| 场景 | 真实输入及只读结果 |
|---|---|
| 管理器详情 | 起点焦点dialog；Tab→关闭、Tab→“全部更新”真实BUTTON；Enter切换后标题“更新中心”，activeElement仍为dialog |
| 面板详情 | 原生点击“查看 OpenCodex 面板更新详情”，焦点为该实际控件；Shift+Tab→“全部更新”；Enter切换后activeElement仍为dialog |
| 全部视图键盘边界 | Shift+Tab从dialog到末项“重新检查更新”，只聚焦未执行；Tab循环回“关闭更新中心” |
| 关闭归还 | Escape后dialog不再存在，activeElement为原“查看管理器更新”BUTTON |

所有输入由Computer Use对当前返回的唯一窗口执行，焦点判断来自随后只读DOM指标。未通过CDP设置焦点、触发click/路由或伪造键盘事件。工具Accessibility focused_element常返回WebView宿主pane，因此不拿该宿主字段代替DOM的实际activeElement；两类原件保持。

管理器缓存当前0.1.10/可用—、最新、备份勾选/安装禁用；面板本地版本未知/已有本地查询错误、远端缓存2.84.0、更新禁用。没有点击check/install/restart或外链，不把“检查失败”的已有本地状态称为本次远端更新失败。短缺省说明、无progress不抵长说明滚动、真实下载/安装/失败/签名/重启进度矩阵。

## 标题栏与云母

普通真实false/type3、实际DPI192/200%条件下，Win32正常窗口foreground=true、DWM backdropAttribute2/result0。重新连接后这个实际属性与前轮断连时attr1分开保存，不把前轮未通过篡改为通过。原生点击最大化后Win32 maximized=true、visible=true/minimized=false/foreground=true、attr2保持。

随后从重新观察的AX“还原”控件执行click，但实际窗口仍最大化、边界不变；这次caption还原点击**未通过**，不能称产品缺陷或工具点击偏移已定位。Alt+Space打开系统菜单，实际截图显示已选择“还原(R)”；Return执行后Win32 maximized=false、visible=true/minimized=false/foreground=true，窗口边界准确回到原正常值，attr2保持。同一实例未重启。

本次证明最大化和系统菜单还原功能，不用菜单成功覆盖caption按钮点击无变化，也不覆盖中午eae还原意图造成最小化的旧未隔离记录。Snap、托盘、正常退出、管理员原生及100/125/150%物理DPI等仍待验；200%与界面scale100不替代原物理矩阵。

## 证据与保持

13个实际动作记录完整，最后输入在23:30前，最终正常窗口/版本设置页面保持。最大化/系统菜单capture包含13×13小辅助帧，同时保存3200×1144完整窗口及176×142系统菜单帧；小帧显式标记不当完整窗口。

截图保存助手用了“.png”文件名，工具实际返回JPEG字节；首次PNG签名校验失败后核对全部原件格式/尺寸并使用实际JPEG解码校验，原字节/文件名保持、未重截图或重跑测试。`screenshot-scope-inventory.json`和`…format-wrapper-correction.json`明示实际格式及范围，不能按扩展名宣称PNG原件。

首末偏好、根绑定、备份策略、成功schedule四项字节完全相同；auto=false继续保持，manager/panel成功/下一截止及beta分区不变。应用/监督器真实保留句柄、创建时间和普通令牌、全机唯一桌面及日常EXE SHA43e06bc3…保持。没有为焦点取证生成新备份、执行包脚本、改root/权限、重启周期或改时钟。

PR #1焦点原生子集现在有实际证据，但仍保持草稿/main未集成。旧CI根因未隔离、原TASK182、FZ50显式脚本许可及独立测试环境/官方0.1.9 Profile仍缺；全更新/备份/树表/事件、安装数据保留、完整真实周期/原始计数、30min/beta6h、同机配对原预算与历史性能failed继续开放。IMP in_progress，无正式RCP/关闭证书，不降低门禁、不关闭或发布。

23:37:31仅更新PR #1标题/正文，标题为“fix(updates): 切换详情视图后保留弹窗焦点”，移除已不准确的“待Windows原生复验”括注；正文引用本轮5dee74b证据并说明caption/完整门禁待验。head/be7395a、base/eae和open/draft状态保持，merged=false/auto_merge=null，没有请求ready/合并或发布。前后API原件及正文归`.adg/evidence/OCX-0110-20261013-WIN-FOCUS-PR-NATIVE-WRITEBACK/`；旧草稿标题记录保持历史含义。
