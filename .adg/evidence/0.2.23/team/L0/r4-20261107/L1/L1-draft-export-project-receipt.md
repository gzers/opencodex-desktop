L1 / 可信草稿导出与非写投影回执

完成提交：3ef3c30b498108f63f6ef55b7d3cd8e28387d4e5
分支：codex/0.2.23-model-matching；工作区：/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes；工作区干净。
依赖正式提交23dd95538（真实file_transfer trait）、87c830c7b（child注册及四个pub(crate)纯helper）、1a544921a（owner lint清理）均已normal merge。未复制integration dirty文件作为编译依赖。

写集三文件：config_file_v2_export.rs、config_file_v2_export_tests.rs，以及追加窄授权parent config_file_v2.rs的scope/ModelEdit+ChannelIntent+seed/rename/public connection共享变换提取。原save guards、credential获取/处理、业务提交顺序保持；现有save/receive的55项回归通过。

实际接口：

- public unit struct ConfigDraftFileCaptureV2，实现file_transfer_v2::DraftFileCaptureV2。
- 注入路径crate::modules::model_config::config_file::v2::export::ConfigDraftFileCaptureV2；owner.with_draft_capture(Arc::new(ConfigDraftFileCaptureV2))。
- capture(&self,d,h,&ReadBaseV2,&CaptureRequestV2,&dyn CaptureCredentialsV2)->ModelResult<CapturedSnapshotV2>。
- config_file::v2::export::project(d,h,SaveRequestV2)->schema_v2::Result<DraftProjectionV2>。
- DraftProjectionV2仅Serialize，字段expected:ReadBaseV2、effective:Vec<EffectiveModelV2>、intended:PolicyV2、digest:String。无raw config或Key；digest只对应脱敏投影，不能当保存授权。

只读边界：全局writer admission、4个既有pending目标门锁、config→Store锁，pending只拒绝，不调用带recover的file-save/channel-rename admission。校验实际draft.base全值、当前D/H root/source/library/saved-policy摘要、draft ID/generation、已存observed_baseline与只读actual provider重算。用explicit existing bindings，不调用会写registry的bindings。结束前再核source/root/Store文件摘要。无snapshot/read_existing/save/自动rematch；不生成临时业务配置重读，不写registry/config/draft/version/WAL。仅锁sidecar可创建。

project复用共享ModelEdit/ChannelIntent/seed/rename、公有connection投影与真实compiler_v2/协议分组/认证方向检查。ModelNaming::validate及真实logical channel alias/raw ID/slash归一和actual native/combo冲突核对均实际执行。scope只约束本次意图，返回DTO仍保留其它server draft未存编辑；不递增backend generation、不提交本地/远端内容。L3需用本地编辑代次抑制过期异步响应，失败保留原队列与base；不能回传intended充当UI policy。

显式draft capture仅LocalFile渠道及精确template refs，按选中model/family缩小可信server draft后纯编译。支持具备template/source/explicit协议的全新staged connection及只读copied_seed，Public provider由真实内存编译结果经过既有public_source/scoper捕获；不猜adapter。未解析的可执行协议会按实际compiler拒绝，不提供虚构底值。源字段与人工override分层保留，v6仅精确revision依赖、无默认revision自动采用。无Key/授权/其它manager/assets/global default scope；credentials接口未调用（真实计数0）。saved-only入口保持跳过bad draft。

验证：normal生产crate 64/64（9个新child测试+55个既有parent/receive回归），完整cargo clippy --lib --tests --no-deps -- -D warnings、三文件rustfmt --check、git diff --check通过。owner真实注入后调用prepare_draft_export并v6 encode/decode通过；原业务文件逐字节不变（含未选draft及registry/版本），没有预先save。新建/复制、source与explicit分层、exact依赖、bad baseline、stale各guard、四pending不恢复、alias错误/碰撞和单scope保留其它draft都覆盖。初轮编译/Clippy失败保留并已修正，未allow或假引用屏蔽。

复验命令：

~~~sh
cargo test --offline --locked --manifest-path /Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes/apps/desktop/tauri/Cargo.toml --target-dir /Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97/batch2-crate-target --lib modules::model_config::config_file::v2
cargo clippy --offline --locked --manifest-path /Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes/apps/desktop/tauri/Cargo.toml --target-dir /Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97/batch2-crate-target --lib --tests --no-deps -- -D warnings
~~~

日志draft-export-final-tests.log、draft-export-final-clippy.log及初轮日志/完整源码SHA256见L1-draft-export-project-manifest.json。已给L0具体实现名/注入和model_v2_file_project注册需求，给L3DTO与异步队列消费建议；公开入口及UI整合由对应线继续。本批仅macOS临时D/H验证，没有真实推理/同步、日常配置或发布；TempDir析构清理，保留target与受控日志。Windows原生未在本批验证。产物本地未上传。
