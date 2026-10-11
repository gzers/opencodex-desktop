# 369ab12d macOS 同源码候选与原生树表局部验证

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182，源码 main@369ab12da55448fbfdc4d0c9bf03970b56fd6c27；版本 / 构建号 0.1.10，arm64。构建 cargo tauri build --bundles app,dmg -- --locked exit 0，用时 121.016s，完整源码 SHA 内嵌核验通过。

原件保存在 /private/tmp/ocx-0110-369a-candidate-tgpmm_zq/original/，二进制未加入仓库。DMG 4384841 bytes，SHA256 7ccb4927be704711997674ad58dba8342c80a70fcf0b62de080440cea351f743，hdiutil verify exit 0。原 EXE 7897776 bytes，SHA256 3e670b4e2e7517e4d10897919b56769030bde3eac944d4642fe5c0e5ded9f52e；ocxd 386416 bytes，SHA256 ee294c65ccc22f6f738e1dbcfce019ccf02ee848fee2ccbec17f00b5eaafc6c1。

原件严格 codesign exit 1：code has no resources but signature indicates they must be present；只有 linker ad hoc，无 TeamIdentifier / 密封资源，宿主 0 个有效签名身份，公证未验证。只读挂载 DMG 后复制安装并卸载挂载；测试副本使用独立 identifier com.gzers.opencodex.desktop.sandbox0110369a、ad hoc 严格校验 exit 0。测试 EXE SHA256 8bb3920180d03c54e78392df71d8e0e57fefda0ddb023d60f2879625e71f4018。测试签名不能覆盖原件失败或代表正式签名。

旧测试 PID75489 原生 ⌘Q 后进程退出、锁可非阻塞取得且锁字节不变；启动新候选为唯一测试 PID9422，显式隔离 test-data 根，环境与锁 metadata 匹配。日常 PID75764 和 Oct9 启动时间未变；未替换日常安装、访问其数据或启动第二个测试实例。

native-tree 保留 00—10 的 JPEG / AX / metadata：00 裁切；01/02 默认根和年展开、月份收起；03 点击月份 cell 未展开；04 鼠标展开后未建立焦点的方向键无效，不能记键盘通过；05 Tab / ShiftTab 建立焦点后 Down / Right 至备份并展开文件；06 Down 至 backup-manifest.json 叶控件；07 Left 回备份父箭头；08 刷新保留展开层级但焦点在 HTML 内容，不能证明异步行焦点恢复；09 切浅色回页顶，树不在截图范围；10 滚动后浅色树及两文件可见。无复选框，因此不替代浏览器复选框场景、完整密度 / 画质 / 物理 DPI 或整窗合成验收。

真实备份 bk_20261011082942_7318022d 仅含 preferences.json 和 backup-manifest.json。载荷 1371 bytes，独立 SHA256 58e1c8cc33f43243cd1dee8db4a45da712106e44c29c1ad6a0f97288843e3c85 与清单一致且 JSON 有效；这是管理器偏好备份，不含完整面板配置。只保存只读校验报告，不复制偏好内容 / 原始清单，不执行恢复。

原始日志按字节 gzip 留存；source-provenance.json 记录原始大小与解压 SHA256；checksums.sha256 校验证据。上述为局部候选验证，未闭合真实安装升级、默认根、完整原生、性能 / 周期、真实更新 / 事件或正式签名，releaseAuthorized / releaseAccepted 均 false。
