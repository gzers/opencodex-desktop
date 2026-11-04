# 43c6253b5 生产闭环通过：真实 prepare、新默认与本地路由

**本次用户指定的 owned production_prepare 验证通过，非候选手工 owner spike。** 真实 save_with_context → prepare_saved_rename_restart → 新官方 Codex 无 model 覆盖 → 实际 Krill provider / 原始型号 / 本机 Responses 响应全部取得；pending 保留。安全复制态的 sync_cache_after_save(true) 只在模拟完整成功时清理副本，真实 pending 没有被确认。

冻结完整提交 **43c6253b582c3fe19ba67c8d1ccfeb4adabd7068**。[26 项证据核对](production-prepare-43-result.json)，[同次真实运行及独立 copy gate 组合结果](run-20261104-162210/combined-production-and-copy-result.json)。

## 来源和实际操作

真实运行 **162210** 只构建一次、save 一次、生产 prepare 一次、模型 POST 一次。432 份 Rust/UI 产品及构建输入与指定提交完全一致，213 份 Rust/构建输入从编译前到运行后没有漂移；后续 a9c747ee8 测试专用提交前进以产品 hash 继续确认，不冒称新产品构建。

真实 helper SHA256：2a2ac4f7c4c90e03941a40b66d57ef3ccfd31bcb19714ca0c3dca662d9cb6778。原二进制保留在本次 owned 临时根 original-real-prepare-helper，原 source 逐字节快照位于 run/original-run-source/。copy-only 修正后的 helper SHA256：4b78be1d52e2dbc13c40ad41806254d9f5c753ea02ab70333c3f92588d23ab8f。两者链接的同一生产 rlib 摘要始终为 1e260580e2ee65db5ddfd7a8a6c5aefd60290fd756f67bf35a403663eb163601。

OpenCodex 来自已缓存 @bitkyc08/opencodex 2.84.0，官方 Codex 来自 @openai/codex 0.166.0 darwin-arm64，配套 Bun 1.4.2；registry.npmjs.org tarball URL/完整性和包源摘要仍保存在 provenance、cached-package-source-manifest.json。未下载或通过用户渠道认证。

## 真实生产部分

| 检查 | 结果与原始证据 |
| --- | --- |
| 旧安装、当前 model | 改名前仅旧侧 fixture seed；官方 producer 建立真实旧集成。真实 config/read 加载 Krill-1/owned-routing-model，[旧配置](run-20261104-162210/old-effective-config.json)。|
| 真实绑定 | actual Management 使用 native-integrations 唯一 Codex 行 installed=true/state=current；路径、版本与 root route 归属由生产验证，未替换返回值，[binding](run-20261104-162210/official-binding-before-rename.stdout)。|
| 生产保存 | [save_with_context](run-20261104-162210/production-save.stdout) changed=true，稳定 channel.id/型号保持，真实 ClientEffects 写 pending。|
| 生产编排 | [prepare_saved_rename_restart](run-20261104-162210/actual-production-prepare.stdout) success=true，返回实际 pending id；测试没有自行 reload/PUT/CAS，没有改名后手工改 config/catalog。|
| 磁盘 no-op 与 live | [文件身份](run-20261104-162210/saved-disk-identity-through-production-prepare.json) 保存后的 config.json 字节/SHA/inode/mtime 均未改变；[生产 Management 读回](run-20261104-162210/official-binding-after-prepare.stdout) 仅 provider=Krill、default=Krill，旧 live 已删除。|
| 新客户端当前默认 | [config/read](run-20261104-162210/effective-config-after-prepare.json) model=Krill/owned-routing-model，user 根 origin；[新 thread/start](run-20261104-162210/default-thread-after-prepare.json) 不传 model/config 覆盖仍为新标识。|
| 新目录 | [model/list](run-20261104-162210/models-after-prepare.json) 两行 Krill/owned-routing-model、Krill/owned-second-model，无 Krill-1。|
| 实际路由 | official Codex exec 无 --model；唯一 [本机 POST](run-20261104-162210/fixture-requests.json) 为 /v1/responses，model=owned-routing-model，合成 Key 匹配。|
| 实际身份 | [官方 usage](run-20261104-162210/actual-production-prepare-routing.json) provider=Krill、resolvedModel=owned-routing-model、routeKind=explicit-provider，不是旧默认渠道 fallback。|
| 响应消费 | [exec](run-20261104-162210/production-prepare-default-local-turn.stdout) turn.completed、exit0；[最终文本](run-20261104-162210/production-prepare-last-message.txt) 为 OCX021_LOCAL_FIXTURE_OK。|

本次 raw-file 路由证明与 native CAS token 由生产 apply_with_file_snapshot 分域处理；smoke 没有生成或替换 expectedVersion。先前 double-prefix 和 raw/native 不等同失败保留，未去 guard。

## pending 与独立安全 commit

真实 pending id：**f4b4c2e5-e029-4202-99f8-5daaf6293bff**。save、prepare 回执与 [所有 gate 后真实 pending](run-20261104-162210/actual-pending-after-all-gates.json) 逐值一致；原文件 SHA256=165514394b15fc221232d04ed2fe384dc7076a3e138e4af812bdfc119ba21b3e。没有真实桌面重启，因此原 pending 正确保留，不能因 prepare/回包成功而 ack。

初次同 run 的最后 copy stage 退出25，原因属于 smoke 夹具：复制后 homeBinding 仅 raw hex，没有生产合同的 sha256:。纠正为生产 store::sha256 后，独立 copy 检查又发现模拟 catalogRefresh 缺 degraded/notices，按生产完整合同补齐。两次初始 driver 错误原始 stdout/源码保留，原 run/result.json 仍是整体 driver failed，**没有篡改为原脚本全绿**。

用户允许仅重跑 copy gate。最终 [copy-gate-only-20261104-162726](run-20261104-162210/copy-gate-only-20261104-162726/result.json) 直接使用同次 Cargo 记录的 .rlib extern/native paths，rustc 只重链 smoke helper；不执行 Cargo、不重编生产库，也不再跑真实 save/prepare/client/model turn。

在复制的 manager/home 上调用实际 production sync_cache_after_save(true)：

- synthetic attested owner 返回 desktop test_environment：生产结果 incomplete，匹配 pending 副本保留。
- 同 fixture 返回完整合成成功回执：实际生产 acknowledge CAS 清除匹配 pending 副本。
- 两次 restart HTTP 仅发给本机新建 mock server；无 OS 枚举、kill 或 GUI 拉起。原 pending 始终逐字节未变。

生产 prepare/live/client 部分是实际官方 owners；仅 restart commit gate 的副本 HTTP/重启结果是明确模拟，不能把它称为真实 GUI 重启。

## 运行与冻结约束

最终可复用入口 [生产 smoke 脚本](/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/test/smoke/ocx021-owned-routing.production.py)（代码根下运行）：

    python3 test/smoke/ocx021-owned-routing.production.py --check-source-freeze-only --source-freeze-head 43c6253b582c3fe19ba67c8d1ccfeb4adabd7068
    python3 test/smoke/ocx021-owned-routing.production.py --run --source-freeze-head 43c6253b582c3fe19ba67c8d1ccfeb4adabd7068 --cache-target /private/var/folders/fq/l0j4lwrs5ld6gpfx2pppv2640000gn/T/ocx021-owned-routing-dhann258/target

需要仅复验副本时：

    python3 test/smoke/ocx021-owned-routing.production.py --copy-gate-only ../governance-main/.adg/evidence/0.2.21/owned-client-routing-20261104/run-20261104-162210 --source-freeze-head 43c6253b582c3fe19ba67c8d1ccfeb4adabd7068

完整 SHA、commit 解析、即使同 HEAD 也检查工作区、untracked product input 拒绝、构建前后/运行前后产品摘要复核。test-only/CI/native/UI tests 前进不进入产品 identity。新产品变更必须重新明确冻结；不自动重试。历史手工/候选 modes 保留独立脚本，不参与本次产品通过。

## 收尾与边界

owned 官方进程全部退出，49668/49667 均 ECONNREFUSED；新临时 CODEX_HOME 专属两个 SQLite 锁文件已精确清理，其它数据库未动。全部业务配置与凭据是临时合成，只放行本次 loopback，真实用户树与 keychain IPC/外网被 OS 沙箱拒绝。无真实供应商/WebDAV、无日常 Codex 读取或重启、无官方 global restart。

本次通过范围为用户指定的生产 prepare、当前 user 根默认、目录加载、实际路由/响应和 copy ack gate；review_model/profile/multi-scope/拒绝/补偿分支由 parent 另有源绑定测试，不以本单一夹具外推。桌面实际退出/拉起、整版 CI/平台/发布由主线汇总，本 worker 不签发整版发布证书。

own scripts 文件与摘要见 DELIVERY-MANIFEST.json，最终证据摘要见 EVIDENCE-FINAL-MANIFEST.json。未改产品代码、未 git add/commit/push，交给 parent 统一提交。原失败及其 pending 证据全部保留。
