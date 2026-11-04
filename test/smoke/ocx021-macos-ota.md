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
