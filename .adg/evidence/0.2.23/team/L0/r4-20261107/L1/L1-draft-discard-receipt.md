L1 / 明确放弃后端草稿最终回执

提交e857da05dd7a32c2f8a78e875eb08309318b0be1；基线851948bd4fc281982c69dfaaedcfccf2052deb99；工作区/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes，codex/0.2.23-model-matching，工作区干净。

仅新config_file_v2_discard.rs/tests、parent末尾注册及pub use discard，以及history::with_state的一行pub(super)复用。生产接口config_file::v2::discard(root:&Path,home:&Path,expected:ReadBaseV2)->Result<FileSnapshotV2>；由L0注册冻结命令model_v2_file_discard({expected})，root/home不从UI来。

用户明确放弃整份server draft（含recovery/copy/connection/Key意图）时，先在同pending/config/Store门下核完整draft.base/generation、当前source/root/library/savedPolicy及observedBaseline；构造actual saved事实成功后，精确草稿原子rename到私有discard tombstone，不把recoveryId置null冒充普通草稿，不readExisting/rematch/registry/config/library/Policy/ledger/version写。返回FileSnapshot.draft=null。

Unix刷新源/目标父目录、Windows使用MoveFileExW WRITE_THROUGH，确认元数据失效后才删私有vault材料；异常保留或还原原草稿且不进GC。失效后再次核source/Store metadata，新generation不清理。GC仅读取这份已证明discarded草稿中的typed Replace UUID和当前imported copy ticket，按digest/schema/channelId/source核对固定路径；凡已被savedPolicy/actual raw引用的材料保护。无扫描vault、旧目录或删除actual provider账户。GC失败保归属明确0600 tombstone（只含opaque refs/typed draft，无Key值）供后续受控定向清理；草稿仍已安全失效，可回普通编辑，不把私有清理延后冒充业务discard失败。

真实定向覆盖：精确普通/key/copy草稿丢弃；source/root/lib/policy/generation任一CAS失败所有原件/slots/tickets保留；实际源URL被外部修改不会被吞；pending拒且不recover；saved copy已用的pending slot不删；GC失败留私有证明；接受恢复→明确discard→新draftId→普通edit/save成功，旧history accept不能覆盖新代次。

97项配置边界回归全过（原90+7discard），最后补持久失效后7项定向再过；完整cargo clippy --lib --tests --no-deps -- -D warnings、cargo fmt --all -- --check及git diff --check过。注入FakeDraftCredentials，不访问真实keyring；没有日常配置、真实推理/同步或发布。临时D/H与fake slots随TempDir/测试进程清理，保留target日志。Windows真实原生仍L4负责。

L3已获明确消费合同：确认后传当前server expected，成功才清本地queue/keyBuffer，accept返回saved事实；继续编辑用现begin_draft得到新身份。失败保当前输入，不用loadChannels(true)或手动清recoveryId假discard。公命令注册及UI由L0/L3接线。

日志draft-discard-regression.log、draft-discard-durable-tests.log、draft-discard-durable-clippy.log，完整源SHA/日志SHA见L1-draft-discard-manifest.json。此前复制/历史/队列已分别提交并被总控集成，113项consumer最终回执L1-final-consumer-receipt.md保持原样，不覆盖不同源码证据。本地未上传。
