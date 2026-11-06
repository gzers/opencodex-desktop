# 正常生产 crate 复验

本批最终证据来自真实仓库应用Cargo.toml及其锁定依赖，不需要临时选择器/模型/接收替身driver。取receipt记录的共同基线及尚未采用的贡献提交，在实际checkout中执行：

```sh
CARGO_TARGET_DIR=/tmp/ocxd-service-own-recheck cargo test --offline --manifest-path apps/desktop/tauri/Cargo.toml --lib modules::model_config::sync_service_v2::tests:: -- --nocapture
CARGO_TARGET_DIR=/tmp/ocxd-service-own-recheck cargo clippy --offline --manifest-path apps/desktop/tauri/Cargo.toml --lib --tests --no-deps -- -D warnings
```

目标目录必须是复验者独占目录，避免与其它对话的build并发混用。当前源码比14项聚合基线追加两项生产保护/导出及一项秘密diff测试，分别有正常crate日志；报告不把重复执行的原首装上下文回归累计成新测试。

实际网络用可注入确定性transport，以PUT计数与远端文件全字节保留断言验证只读边界；capture、模型v6、overlay/wrapper、identity/local/ReceiveFileHost和模型事务均为已提交真实生产实现。不是全业务WebDAV验收或L0其它过滤测试全跑。

完成后清理自己的临时target并保留原始日志；此批L2私有target与开发driver会在归档后移除，不依赖/tmp目录继续存在。
