L1 / AC-P22-02 同制品真实名称链验收

提交85c773fda8e9f23a5f510752fc82d44022bf1676，仅新增tests/runtime023_manager_name_chain.rs与test/smoke/runtime023-manager-name-chain.mjs。正常主线基线2b7d4586/cce22已合；后继cb6真实backup consumer用于严格gate。无产品/版本/锁/registry新改，分支codex/0.2.23-model-matching工作区干净。

最终实际验收1/1通过，四label、每label两个raw模型，耗时166.53秒。链路为临时D/H真实config_file::v2::snapshot→begin_draft(实际existing draft.base)→SaveRequestV2 typed Rename scoped save→真正complete bundle provision临时I一次→随包Node26.8.1调用随包npm11.19及Bun1.4.4→安装目录原始config/gather/build/retained-sync模块→临时CODEX_HOME实际catalog.json读回。没有复制matcher/编译器、mockUI、造module结果或替换包内源码。

通过名称：RenamedEnglish、中文渠道、Team Space、中文 / Team。同channelUUID始终60d51f41-de3b-4cea-aea5-8e2564e5cd0b；英文key改为RenamedEnglish，非法key文字由实际manager分配channel-UUID。raw ID model-1与vendor/original-model保持，实际routedSlug/decode codec roundtrip，display_name精确为name/raw（包括中文/空格/斜杠）。未选untouched provider、root/provider unknown字段保留；catalog未知root字段保留；消费者前后manager config字节一致。

固定真实制品：完整archive SHA256 982580a65408d5325be3b707288a31a427583e9589b0057ae11be57e7fa11899，237732299字节；descriptor SHA256 88a96e98e3070baa05c7d2ccb402ba70f491f25ca4a93505f8927b8eb0b8b152；runtime包SHA256 afb72a4c792c1ca70a0fcd9209d212c78249281d2d7bc42f8eee63aa4464b26b，版本2.97.0-desktop023.names.1。完成后重新hash archive/descriptor原输入仍一致，无同版本不同字节。原路径来自环境输入，不硬编码私人路径进源码。

隔离边界：I/D/H/CODEX_HOME都在同TempDir；OS sandbox deny全部network和外部exec，只允许固定I内Node/Bun，PATH空。authMode=none，无APIKey，liveModels=false，catalog entries accountSelectors=[]，不启动服务/推理/日常Codex。沙箱明确deny I程序写，并比对实际导入6个package/module文件前后SHA不变。

真实runtime K锁固定系统/tmp/opencodex-runtime-v1-UID/catalog-write-locks/SHA256(canonicalCODEX_HOME).sqlite，不服从TMPDIR。没有mock该resolver；实际导出resolver核对精确随机测试home hash，只放行该sqlite/-wal/-shm/-journal及必要创建私有父目录，非整namespace写权限。RAII清理这些本测试新对象，日志CLEANUP_OWN_CATALOG_LOCK filesRemaining=0；完整TempDir在测试结束销毁。日常home读/写在沙箱拒绝，没有日常配置或进程重启。

完整strict integration Clippy（--features integration-test --test runtime023_manager_name_chain --no-deps -- -D warnings）通过，cargo fmt --all --check及git diff --check通过。实际full run产物基线dcea75426cdea9628c7717d440c8a061d3651d5a，strict基线3a63c40b46e35d85a2160d54cd44bcb98a31d032；二者实际manager/编译/provider导入/bundled/install源码逐字节同，后继仅新增真实backup消费者消除共享deadcode，未以假引用/allow掩盖。精确源码hash在manifest记录。

首两失败保留：manager-name-chain-first.log（初sandbox拒硬编码K锁SQLite）；manager-name-chain-own-lock.log（英文完整成功后fixture漏已有draftguard，被真实CAS拒）；另manager-name-chain-clippy-first.log保共享基线deadcode失败。修复仅测试真实环境/接口消费，不降低显示/路由/保留原字段断言。旧包回归复用既有runtime023-baseline-regression.log，已复制为manager-name-chain-original-package-regression-reused.log，证明旧包selector display Krill model-1仍为krill/model-1；未重复跑旧包。

运行方式（由调用者显式设置归档/描述/可信pin）：

~~~sh
OCXD_TEST_BUNDLE_ARCHIVE=/absolute/platform-bundle.tar.gz \
OCXD_TEST_BUNDLE_DESCRIPTOR=/absolute/platform-bundle.manifest.json \
OCXD_TEST_BUNDLE_DESCRIPTOR_SHA256=88a96e98e3070baa05c7d2ccb402ba70f491f25ca4a93505f8927b8eb0b8b152 \
cargo test --offline --locked --manifest-path apps/desktop/tauri/Cargo.toml \
  --features integration-test --test runtime023_manager_name_chain -- --ignored --nocapture
~~~

兼容已有OCXD_RUNTIME_BUNDLE_ARCHIVE/MANIFEST/MANIFEST_SHA256环境名；默认#[ignore]，缺输入不能假通过。程序内只使用provision后的固定Node/npm/Bun路径。最终日志manager-name-chain-readonly-final.log，结果四条MANAGER_RUNTIME_CATALOG及MANAGER_NAME_CHAIN_RESULT可机器收取；完整源码/日志/制品SHA与每label的config/catalog摘要见L1-manager-name-chain-manifest.json。

本项是AC-P22-02真实同制品链验收；不代表软件UI、Windows原生/两SID、安装制品或发布全版通过。相应工作由总控继续。本地产物未上传。
