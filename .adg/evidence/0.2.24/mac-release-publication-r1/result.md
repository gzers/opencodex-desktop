# 0.2.24 发布入口离线验证

固定候选源码 [2a4c51f60](https://github.com/gzers/opencodex-desktop/commit/2a4c51f6040bbe6e419c9ace7e3d7849a2e6de9d)。使用生产装配回执写入器、模拟 GitHub 请求与临时 Minisign 密钥。真实构建与正式签名仍待验证。整版 `accepted:false`。

- 发布前绑定完整源码 SHA、数字 Release ID、装配 Run ID 和 Attempt，核对标签、工作流、回执 ZIP、制品字节与签名。保留 Latest 和晋升同一已公开版本两条路径均通过。只读草稿入口拒绝已公开版本。
- 14 种装配错配、7 种发布前异常和 3 种发布后异常均按预期拒绝。发布前重查标签、Release 和 Latest，单次修改已核验的数字 ID。发布后验证 Release 身份、Latest 及匿名访问的 `latest.json` 长度、SHA-256 和内容。
- 最终 Python 回归 105 项、两份工作流的 actionlint、Git 差异和版本预检通过。预检显示四处版本均为 0.2.24，分发方式为 `official-npm`。
- 首次组合回归缺少 `PYTHONPATH=test`，四个测试模块未加载，原输出另存。工作流脚本演练也发现 Bash 3 在 `set -u` 下展开空可选参数数组失败。改用位置参数后复核，发布测试覆盖输入缺失与令牌隔离。
- 装配和发布演练的 GitHub 远程调用数为 0。真实草稿、正式签名、Actions 制品、公开后匿名读回均仍待验证。
