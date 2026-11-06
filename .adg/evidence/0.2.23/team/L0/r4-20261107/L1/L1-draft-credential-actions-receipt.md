# L1 草稿凭据动作元数据回交

2026-11-06，Asia/Shanghai。提交b4c38f22cccb3884aaa04b183f39f02dd5e17a89，父f76bcbc1b7c8fd763020314e22e9a10da37cd102，分支codex/0.2.23-model-matching。仅store_v2.rs与consumer_v2_tests.rs；工作树干净，不改L0 gate/UI/命令/公共注册。

DraftV2新增 #[serde(default)] pub credential_edits:BTreeMap<String,DraftCredentialEditV2>，wire名credentialEdits。公开enum Replace{credential_ref:String}/Clear，wire为{kind:"replace",credentialRef:"UUID"}或{kind:"clear"}，只有引用/动作，没有秘密值字段。只由可信后端stage更新，UI只读；saved Policy、portable、ledger没有新增这个字段。

Store验证map key是规范管理UUID且渠道在draft.policy中；Replace引用是规范UUID且精确等于该渠道connection.credential_ref；Clear要求credential_ref=None，但不改authMode。Clear不是auth:none，保留key模式合法。错误引用/未知渠道/秘密字符串冒充ref拒绝。

CAS保持原接口：整Draft Eq包含actions；仅改凭据action而不增generation也拒绝。正常递增持久仅草稿，Policy/ledger零写，baseline不被改成人工source。两个既有fixture初始化空map，缺字段默认空仅为未发布schema2 fixture，不引入v1迁移。

严格反序列化有具体必要实现：Serde internally-tagged unit Clear即使derive Deserialize+deny_unknown_fields也会接受附加字段。新增拒secret回归实际抓到20通过1失败。最终Serialize继续derive，Deserialize委托私有Strict enum的Clear{}空struct variant再转回公开unit Clear；保留完全相同公开Rust API和wire，同时使额外apiKey/secret/value/credentialRef（含null）真正拒绝。原失败日志保留，不压测试或改公共API。

正常完整应用crate受控快照21消费者测试通过（原18+新增3），0失败/忽略，1305无关项过滤。新增验证Replace/Clear真实临时根往返、无secret、错key/ref/不规范引用/unknown字段拒绝、Clear不改变authMode、动作-only CAS、Policy/ledger零写及fixture缺字段默认。正常crate Clippy --lib --tests --no-deps -- -D warnings通过；rustfmt/diff检查通过；两文件与受测快照逐字节核对。

证据同目录：draft-credential-actions-tests.log（最初Serde unit variant回归失败）、draft-credential-actions-final-tests.log（最终21过）、draft-credential-actions-clippy.log。测试临时根自动清理；无真实配置/上游/网络/模型调用，无服务或端口。未push/发布，未写共享docs/.adg。

L0后继：生产Draft初始化明确保留/填空credential_edits，server stage-key-only/clear命令写动作；明确动作才删除APIKey/池，不从apiKey:null/缺字段猜clear；再次Read保留动作，scopedSave仅清理已保存scope动作并保留其它草稿动作。实际Key查看/秘密存储、保存门/各分组凭据应用、读回与恢复由L0完成，本批不声称其实际链路通过。

