# 86 产品 freeze：default npm 完整生产 prepare 验证

**本 worker 的 default-npm 生产组合门通过，35/35 证据核对。** 真实保存 Krill-1→Krill、生产 prepare、新官方客户端无型号覆盖的默认线程、实际 Krill/原始型号/本机 Responses 回包和 pending/copy ack 均在同一次 fresh run 成功。

[原始运行结果](run-20261104-181831/result.json) · [35 项机器核对](production-prepare-86-result.json) · [交付脚本及摘要](DELIVERY-86-MANIFEST.json) · [证据文件清单](EVIDENCE-86-MANIFEST.json) · [验证入口](verification-86.json)

产品 freeze 为 86eee1dbff23823eead733cb65308cf02fd95f9e；实际 HEAD 为 6e2197fd0b8a36dcb8d893b62693c5a2a77151cc。构建前后、运行前后共五次核验，433 份产品输入均同 freeze；tree listing SHA256 为 4b33faefea87076c15ebc54a2302fbd901a99362df117f0ea4c06005c6b7bbf6。允许 test-only HEAD 前进，没有把新产物冒充旧 SHA。

| 实际步骤 | 原始证据及结果 |
| --- | --- |
| 官方默认 npm 入口 | 初次实际 resolver source=path、无 persisted runtime；标准 .bin/codex symlink 指向 ../@openai/codex/bin/codex.js，未设置 CODEX_CLI_PATH 等 forced native 变量 |
| 旧安装与默认 | 改名前真实官方 syncModelsToCodex 建立旧集成/catalog；config/read 确认 user 根 model=Krill-1/owned-routing-model，无活动 profile |
| 真实保存 | production-save.stdout：config_file::save_with_context success=true，稳定 channel/model 身份保留并写 pending |
| 生产 prepare | actual-production-prepare.stdout：cache_refresh::prepare_saved_rename_restart success=true；仅调用生产入口，没有测试补救 owner |
| 全 live/catalog 采用 | official-binding-after-prepare.stdout 仅 Krill/default Krill；catalog-after-production-prepare.json 仅新命名；provider config.json 字节、SHA、inode、mtime 全部不变 |
| 当前选择与新客户端 | effective-config-after-prepare.json、models-after-prepare.json、default-thread-after-prepare.json：当前默认/目录一致为 Krill/owned-routing-model，thread/start 无 model/config 覆盖 |
| 实际有效路由 | npm launcher 执行 exec，无 --model 或 --config；仅一次 POST http://127.0.0.1:57964/v1/responses，端点收到 rawModel=owned-routing-model；实际 usage provider=Krill、routeKind=explicit-provider、HTTP200 |
| 客户端消费响应 | production-prepare-default-local-turn.stdout 与 production-prepare-last-message.txt：OCX021_LOCAL_FIXTURE_OK，exec exit0 |
| pending 与副本提交 | 真实 pending 29da2212-a4b0-447e-acb0-f7cf37f46dd2 保留；生产 sync_cache_after_save(true) 只在独立副本连接 synthetic attested HTTP，缺重启完成保留副本、成功回执仅 ack 副本 |
| 清理 | allOwnedChildrenReaped=true，两个端口均 ECONNREFUSED；仅本临时 CODEX_HOME 摘要命名的协调 SQLite 文件清理，未枚举/终止其它 Codex |

Codex 原生 thread.modelProvider 的真实值为 openai：官方集成把内置 openai transport 指向 owned OpenCodex proxy。Krill 是实际 OpenCodex 路由 channel/provider，由 actual-production-prepare-routing.json 的真实 usage 单独证明，不混淆两个字段。

最终 settings 返回的真实路径是 /private/var/folders/fq/l0j4lwrs5ld6gpfx2pppv2640000gn/T/ocx021-owned-routing-blg8n9la/tools/npm-client/node_modules/.bin/codex，version=0.166.0、source=configured。官方真实旧安装 producer 将初次 PATH 发现自然持久化，persisted origin=discovered；driver 没有设置 pin、删除记录或替换 settings。observer/exec 一直运行原 npm JS selector，production owner 自行通过同包 native resolver 执行客户端配置 owner。

本机 cached 发布物为 @bitkyc08/opencodex 2.84.0、@openai/codex 0.166.0 与 darwin-arm64 optional native、Bun 1.4.2。npm 两包原样复制到 fresh 工具根，Node 单独复制且 PATH 只包含这两个新工具目录和系统目录。46 个复制包文件、2065 个原 cached 发布源文件全部摘要一致；registry URL/integrity 来源继续见 provenance/ 与 cached-package-source-manifest.json。没有本轮下载或渠道认证。

生产 helper SHA256：0b828742999f3804ad6d54435aa29a2cb2ba378b3c49090c67fc7779049a2536。生产 rlib SHA256：33e2a00438ba7de0989f76ec3f1cccb24749015805da95f69d642914244921bb。九个实际运行脚本在 run-20261104-181831/original-run-source/ 与 delivered-smoke-86/ 原样归档，均匹配原 result.json 中的运行时摘要。

重放入口在代码根 /Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes；默认无 --run 不启动 runtime。需要 fresh 根验证时使用新的完整、已授权产品 freeze；本次实际命令为：

    python3 test/smoke/ocx021-owned-routing.production.py --run --default-npm-client --source-freeze-head 86eee1dbff23823eead733cb65308cf02fd95f9e --cache-target /private/var/folders/fq/l0j4lwrs5ld6gpfx2pppv2640000gn/T/ocx021-owned-routing-5yh6z3c0/target

仅核对 freeze、不启动客户端或编译器：

    python3 test/smoke/ocx021-owned-routing.production.py --check-source-freeze-only --source-freeze-head 86eee1dbff23823eead733cb65308cf02fd95f9e

先前 default npm 尝试 [172919 原始拒绝](run-20261104-172919/result.json) 与 [原因](run-20261104-172919/binding-refusal-analysis.json) 保持原样：实际 save 成功，prepare 以 rename_client_binding/exit23 拒绝，pending 保留。夹具把 npm 安在临时中间 /home/npm-client/ 下，上游非 root-anchored 用户名正则把 npm-client 脱敏成 [USER]；产品拒绝猜测任意中间片段正确。没有 shim takeover，原 symlink/selector 未变。只把下一 fresh 根的工具前缀移到 tools/npm-client，未改产品、未替换服务 DTO、未修改名后配置。这次完整成功链没有复用失败根状态。

OS 隔离拒绝日常 home、Keychain IPC、外网，仅开放两个 exact owned loopback 端口；已通过文件/网络 canary。旧安装 fixture 仅生成临时 service-state/plist 文件，不注册 launchd。实际 save/prepare/官方 producer/app-server/exec/Responses 均真实；copy ack 的 HTTP/重启回执明确为模拟。未调用实际全局 restart 或日常客户端枚举/kill，未做真实 GUI quit/relaunch，真实 pending 未 ack。本项证明应与主线生产 true 路由守卫测试组合；不声明 false 分支应写配置，不扩充 sync-cache 合同，不替代跨平台构建/发布门或真实 GUI 重启。

脚本写集已稳定，仅本 worker 的三份 default-npm driver 文件相对上一交付发生变化，其余六份保持原样。具体 SHA 见 DELIVERY-86-MANIFEST.json；未改产品/平台 smoke/CI，未 git add、commit 或 push。父级可统一提交测试与证据并继续其发布流程；本 worker 无剩余运行阻断。

旧 43 原始结果、机器结果、DELIVERY-MANIFEST.json、EVIDENCE-FINAL-MANIFEST.json、verification.json 均不重写。共享 README 增加最新入口前，136 个旧清单文件连同原 manifest 已逐个核验并完整快照到 [archive-43-final-before-86/](archive-43-final-before-86/EVIDENCE-FINAL-MANIFEST.json)，该目录可作为旧 manifest 的原始相对路径基准。更早所有失败与候选 spike 也继续保留。
