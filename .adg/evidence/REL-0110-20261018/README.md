# 0.1.10发布证据

授权及范围：authorization.json、accepted-exception-disposition.json；原TASK-182和旧失败保持，不将未验项改为passed。本目录不是完整AC完成证书。

最终源码与事实回写：source-binding.json、core-writeback-verification.json、demand-writeback.json。main/tag为2578e11b，应用源码树与已验8e相同，额外只含CHANGELOG和共享README。

ci/：main首次CI39097338564，六job原日志及实际checkout核对；release-ci/：首次双平台Release39097612293；mac-asset-check/与prepublish-assets/：真实资产/API字节摘要、Mac包内版本/源码、DMG校验和三包旧公钥验签。原始日志保留CRLF及空格，不重排/归一求diff通过。

publish-ci/：单次stable派发、唯一39098939911及成功原日志。public/：公开Release与Latest元数据、匿名固定/stable清单及四资产下载摘要。postrelease-native/：实际发布Mach-O的独立测试身份副本首屏/关于、精确启动源码、正常退出及日常保护读回；不是生产身份Gatekeeper/旧客户端OTA或Windows重新实测。

发布事实以docs/03-开发实施/REL-OPENCODEX-DESKTOP-10.md为权威，短观察归RUN-OPENCODEX-DESKTOP-20261018-01。release-scope-completion-audit.json先记录所有发布项与“最终文档尚待提交”的时点，随后Git提交/远端回读补足文档交付，不改写该时点原件。manifest.json封存本目录所有其它文件。
