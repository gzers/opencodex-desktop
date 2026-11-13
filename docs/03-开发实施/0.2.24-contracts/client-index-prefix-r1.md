# 索引scope绑定的同次认证帧链消费 r1

revision: 1；2026-11-13；IMP-34 P24-04；main 0f0e85fb3；accepted=false。

root_binding.verify_capture原open_profile_origin已用同一refs的独立Original/phase摘要验证完整帧链，随后丢弃frames，再由verify_index_scope重新读取Original+Ready。现在改用原已有open_profile_origin_prefix返回的同次(originals,frames)，立即传给原verify_index_scope_prefix；无写入、外部callback、修改refs或known，也不在Originals保存frames。归档路径早已使用此接口，未新增公开证明或跨调用cache。

完整prefix仍每帧核source/provenance/namespace、hash/pred/schema/root/权限/Original/tree/Grok archive，scope保持严格hash/保护/schema/attempt/Original/UID/data/profile/client路径identity/HOME及identity；live保持hash/schema/scopehash/Ready原字节hash/process/parent字段。helper不独立接受任意外来frames作为证明；其私有caller保证来自同次完整opener。当前program匹配/活体Ready仍由原scope_binding/source/native/签名Ready承担，本接口从不授予进程或写入权限。

同binary实际Index.open_existing正反顺序参考与新路径：首代frame8→4，参考约201.85–211.43ms/当前137.21–143.65ms；第二代frame24→20，参考413.41–418.73ms/当前361.67–369.39ms。完整historybegin/finish次数不变，两次取得锁过程各少读Original+Ready。debug合成数据仅支持局部观测，不证明原Stop10或通用收益。testreference是cfgtest TLS/RAII，无产品env/serde入口。

两个新例：首/后继根取得返回同ReadyHead，计数少4frame；HOME/client/ReadySHA/generation错误采用合法新内容hash并恢复400，scope变化同步live依赖，先拒绝后只恢复目标字段及必要hash关系，必须成功，以免其它过期引用掩盖语义校验。初case存在live.scopehash未同步问题，review后加强，最终语义例单独执行；产品代码字节未变，两次源码/binary身份分别记录。

98个唯一函数通过（prefix筛选8项含2新例及6项既有根状态、其余Session34、索引56；7ignore不计，最终semantic加强单独重测且去重），strict all-targetClippy/fmt/diff通过。原Session/普通canonicalclient/非Ready构造/来源/索引/中断测试保持。帧返回只在当前观察内使用，去掉第二次读取不代表所有不合作外部瞬态修改的采样时刻完全等价；保留的是独立来源字段与完整认证帧绑定，后续调用重新读取。

证据.adg/evidence/0.2.24/client-index-prefix-r1/保存假设、3源和两次testbuild身份、对照、语义加强/检查与清理。无官方runtime/service/全局协调DB、日常配置/Shell/安装app或真实WebDAV模型请求。

原ready-native-r1第二Stop10729ms/deadline耗尽仍失败。下一步累计已测修复后冻结release-source，单轮原包两个完整循环确认normalStop10；若仍失败按trace定位，不增加/重置deadline或跳过内容/权限/OSproof。其它根中断/动作回执/缺Ready/客户端图路径/完整进程集合与刷新/NAME/代理/WindowsW1W2/发行README与固定候选等IMP34完整目标保持，未发布。
