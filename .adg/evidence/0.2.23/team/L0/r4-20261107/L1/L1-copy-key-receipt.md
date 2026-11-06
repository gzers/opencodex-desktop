L1 / 显式复制Key回执

提交d34775a37d78c10c719dd178844248805caf1e99。原型和旧UI均默认未勾选，新增CopyChannelV2.reuseSourceKey bool，serde默认false，UI不传真实Key。copy source receipt固定后端reuse/pending credential intent；不复用时副本credentialRef=None+Clear，seed全部PRIVATE/runtime credential metadata剥离且票据不存源pending credential ref；复用pending replace则引用后端immutable槽，剥离旧pool/header等后由原save vault/统一事务取值；pending clear仍空。来源/其它draft不变。旧save原行为、draft导出边界保持。

取消在UI确认前不调用copy，不产生持久业务效果；确认只落server draft和私有copy source票据，不写原配置或复制Key到新vault。stale guards拒绝不留票据；缺vault保存失败保留原intent/引用，补回后同intent重试。已有Rust tests仅授权各补一bool字段，新copy tests6项。

70项原边界回归与最后6项定向全过；完整严格Clippy/rustfmt/diff通过。日志copy-key-first-tests.log、copy-key-final-tests.log、copy-key-clippy.log，源码/日志SHA见L1-copy-key-manifest.json。L0已确认集成1b0f78bba，L3获reuseSourceKey明确bool合同。无UI/命令/发布修改。本地未上传。
