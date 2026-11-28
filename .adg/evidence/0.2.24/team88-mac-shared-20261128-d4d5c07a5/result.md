# TEAM88 Mac 共享修复回执

代码已推送至 feature/024-team86-shared，SHA d4d5c07a56b7610e0de0ecde907d2f09a2680114。整版 accepted:false。[源码](https://github.com/gzers/opencodex-desktop/commit/d4d5c07a56b7610e0de0ecde907d2f09a2680114) · [本机验证](verification.json)

- Desktop 数据根和 cache 校验通过后，缺少 OpenCodex HOME 也能生成本地报告。本地检查未启动官方 Doctor。主动联网时共用 5 秒进程退出和输出预算，分别报告等待退出或读取输出超时，以及受控进程清理结果。历史 7.5 秒故障仍无阶段取证。
- 缺包回归使用 mock transport。确认前零请求，确认后校验固定归档并记录同一 attemptId 的终态。真实 macOS 随包准备通过，安装锁已释放，测试配置保持。Windows 原生 GET 待实机验证。
- macOS 后端 2,430 通过、52 跳过，严格 Clippy 通过。前端 913 通过，生产构建成功，生产入口以模拟 IPC 验证 4 项。测试使用临时目录，原验证记录保留。[本提交 CI](https://github.com/gzers/opencodex-desktop/actions/runs/46132958033)
- v0.2.24 的 tag 和归档文件名与 Release 上传一致。正式 Release 未公开时，公开归档 URL 返回 404 符合预期。

## Windows 复验输入

从 Actions 制品 15224462563 提取运行归档和配套描述文件，核对验证记录中的摘要。原候选已作废，该归档仅用作隔离下载夹具。通过受控同源输入检查确认前零原生 GET、确认后固定摘要以及同一 attemptId 终态，记录实际 GET 数量和夹具清理。首次可信空闲由 Windows 生命周期提交及对应回归另行证明。[Actions 制品](https://github.com/gzers/opencodex-desktop/actions/runs/46049421165/artifacts/15224462563) · [原归档核验](../team85-candidate-20261128-9a452c37d/verification.json)
