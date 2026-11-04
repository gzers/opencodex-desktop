# 0.2.20 → 0.2.21 macOS 相邻 OTA

当前仅准备。没有父级明确通知最终 v0.2.21 已发布到既有 beta 并给出 source SHA、原包及公开发布回执时，不执行升级。0.2.19→0.2.20 的 REL23 / final-candidate-updates-20261104 只作为方法来源，不计为本轮验收。

~~~sh
python3 test/smoke/ocx021-macos-ota.py --mode prepare
~~~

准备模式不查询 beta、不启动 app，只验证缓存原封 0.2.20 发布资产、旧公钥签名、完整 app seal、嵌入源码、复制后逐文件身份；编译既有 native AX/capwin 与独立只读 PID/environment monitor，验证权限和系统网络边界，并生成执行合同。每次创建新的 /private/tmp/ocx021-mac-ota-UUID/root，保留失败原件。准备成功的 result 是 prepared-awaiting-parent-publication，不能写成 OTA pass。

固定旧来源为 b1750f5cf78b294b2edb34b9674d8c707ef8f810 / 0.2.20。旧 archive SHA-256 是 8fe307fe2f5daaa87c41abbcccc38ef7db346cd5e88ee1456d3285bfb1d8d0f9；旧 EXE SHA-256 是 544de246de1a498b8c681c0bd6ce254c7043ffe4b5fe754d67d3477a8fe4370b。默认复用 /private/tmp/ocx020-final-updates-b1750f5c/assets/ 下缓存的 archive 和 sidecar，可用 --baseline-archive/--baseline-signature 指定同字节原件。

父级发布通知后，提供实际发布回执 JSON（示例中的占位符必须替换）：

~~~json
{
  "schemaVersion": 1,
  "parentNotified": true,
  "betaPublished": true,
  "tag": "v0.2.21",
  "sourceCommit": "<完整40位最终源码SHA>",
  "releaseId": 0,
  "archiveSha256": "<最终上载Mac updater archive的64位SHA256>",
  "betaManifestUrl": "https://github.com/gzers/opencodex-desktop/releases/download/beta/latest.json"
}
~~~

~~~sh
python3 test/smoke/ocx021-macos-ota.py --mode execute \
  --parent-beta-published \
  --source-sha '<完整40位最终源码SHA>' \
  --publication-receipt '/absolute/path/to/parent-publication-receipt.json' \
  --target-archive '/absolute/path/to/original/OpenCodeX.Desktop_aarch64.app.tar.gz' \
  --target-signature '/absolute/path/to/original/OpenCodeX.Desktop_aarch64.app.tar.gz.sig'
~~~

只有 execute 会匿名读取公开 tag/release/beta。要求 tag 精确对应父级 source SHA，release 已非 draft、ID吻合，beta 精确提供 0.2.21 和原 tag 资产 URL，本地目标字节吻合 GitHub asset digest 与父级摘要，sidecar 与 beta manifest 相同，并通过旧版公钥 minisign 验签。只读取公开公钥，不读取签名 secret。目标 app 的 seal、source、整包文件也独立核验。

执行顺序：

1. 将原封 0.2.20 app 放到本轮 installed/，仅有一次脚本启动调用。进程级 HOME/CFFIXED_USER_HOME/CODEX_HOME/OPENCODEX_HOME/XDG/TMP/ROOT/BOUNDARY 都固定在本轮，传 -ApplePersistenceIgnoreState YES 仅抑制 AppKit 旧恢复提示，不改系统设置。
2. 真实旧 UI 在合成渠道上两次保存启用/停用，每次要求新增恰好一份精确前像备份；没有系统凭据。通过真实设置 UI 选择测试通道，冻结业务 config、布局、偏好、备份前像和合成客户端/Skill哨兵摘要。
3. 通过旧版“检查更新→查看详情→确认更新”触发生产 updater 的 beta 下载、旧信任验签、原位安装。pending handoff 和“重启并应用更新”必须由生产流程生成；重启前已安装整包逐文件等同公开原始签名载荷。
4. 仅点击生产“重启并应用更新”。不手动补起 app，不重开第二个 readback 进程。精确安装路径发现新 PID，检查旧 PID 退出、新 PID 启动时刻及继承的隔离环境，在同一个自动新进程挂载真实 WKWebView 后读回原渠道；startup source/new version、整包 seal/字节、全部业务摘要与 handoff 消费必须通过。
5. 清理旧、新及观察到的自有子进程组。daily80916 永不进入候选集合，不按名称隐藏/关闭/杀 app，不全局操作 LaunchServices。

网络只开放真实更新：候选 Seatbelt 拒绝直连，仅可访问本轮 loopback tunnel 端口；driver 的 CONNECT 隧道只转发 GitHub 发布域名的443，不做 TLS 终止、响应替换或缓存。TLS证书和 updater 签名由真实生产栈验证。配置页使用本轮手动代理偏好，继承到自动新进程。没有 fake IPC、endpoint 覆盖、二进制修改、重签、真实供应商/同步/服务操作。代理只记录主机、字节量、时间和错误类型，不读取 TLS 内容或秘密。

证据默认写入 docs 工作树 .adg/evidence/0.2.21/macos-adjacent-ota-20261104/新UUID子轮，含 run-context、准备/执行报告、签名日志、bundle inventories、AX/单窗口截图、两次真实保存前像、发布回执、生产隧道日志、自动进程身份与选择性环境、数据前后摘要及全部 owned 进程清理。首次准备中 macOS Python 3.9 tar filter 兼容和 Seatbelt localhost 语法失败保留，不改写成通过。

--mode execute 缺发布输入在任何根/app/network操作前退出2。实际运行中 fail退出1，blocked退出2；只有完整实际 OTA通过才 report pass。父级的最终021 all/overview验收由父级负责，本 driver 不重复也不替代该验收。

匿名 GitHub API release/tag 元数据因403/429明确限流时，仅这三个公开API类别可经 host 现有 gh api GET 读取：021 release、021 tag ref及其 annotated tag。记录 metadataSource=authenticated gh public metadata，并保存原匿名错误；不冒称匿名API通过。使用正常凭据适配，不读取/打印token，不向candidate环境传凭据。beta manifest和产品tag资产下载没有认证或缓存替代，仍为匿名真实HTTPS/TLS/签名流程；所有原packet/source/旧公钥签名/自动重启/同进程读回断言不变。c6a首轮403原件保留，其旧app未启动、otaExecuted=false，不是产品OTA失败。

5089轮已完成旧UI两次保存及beta选择，但原driver错误地读取扁平偏好字段：真实0.2.20保存的是schema2的updates.app_update_channel。按显式schema1/2读取实际字段，仍必须等于beta，未知schema拒绝，不改写或推断通道。该轮otaExecuted=false，KeyError原件保留。只读monitor对直接启动app可为nil的NSRunningApplication.launchDate，使用同一精确已准入PID的真实OS ps lstart读回；不以观察时刻伪造启动时间，自动新PID仍必须满足同路径、新启动时刻与继承根，并且禁止手动补起。

f7轮真实下载并原位安装后，旧进程AX只返回空根，候选单窗口截图仍为“安装就绪，等待重启”；该阻塞原件保留，未手动启动新版本。对此仅在真实签名handoff存在、旧PID仍运行、原位包逐文件等同目标的条件下，允许现有capwin/Vision对精确PID单窗口证明旧/新版本、等待重启及唯一“重启并应用更新”按钮。重启时再次截屏/OCR定位，monitor严格核对active、未锁屏、最前方命中窗口ID/ownerPID相同，向该PID发送一次真实鼠标点击。没有AX/DOM替代事件、盲坐标或全局点击。自动新PID发现、继承根、原包seal/source和同进程原生业务读回要求全部不变；该截图输入路径不能绕过任何发布/签名/数据断言。

307轮完整下载/安装后，Vision把副标题“等待重启”识别为“等待重后”，语言校正读取同图仍相同。原始图文保留，不改写OCR输出。手接版本只取消副标题冗余短语逐字匹配；仍必须识别0.2.20、0.2.21、“安装就绪”和唯一精确按钮“重启并应用更新”，且按钮点击前原签名handoff/目标整包逐文件相同/active命中窗口准入全部不变。已有307 controller仍运行原已加载逻辑，不热改、不再次启动新根；正常结束后由父级长期session接手。

307已按原360秒观察窗正常结束并清理，未发SIGTERM。该轮后来候选失去active，手接版本在观察/命中测试前仅activate精确已准入PID，再要求其active与唯一窗口命中通过；不隐藏/关闭其它应用。f7和d836的SIGTERM来自assistant显式os.kill自有controller以清理已记录阻塞/失败，不是父级interrupt或工具lifetime；controller-signal-provenance.json保留归因。307已签名安装但无自动新PID，实际OTA仍blocked。手接必须fresh完整执行，不允许启动307已安装021来替代自动重启验收。
