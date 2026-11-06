# L2 最终 G0 合同包 v1

本地受控回交产物，供 L0 按本包摘要冻结/归档；尚未提交/推送，不是 R1 放行或生产实现。以本次总控裁决为依据，不修改最初 G0 回交。禁止真实 WebDAV、Key、远端清空。

| 文件 | 用途 |
| --- | --- |
| PROTOCOL.md | 正式协议、状态、关联不变量、业务签名、L0网络trait精确需求 |
| dto.schema.json | JSON Schema 2020-12，33个DTO定义；请求使用#/$defs/类型名验证 |
| error-policy.json | 26个稳定错误的恢复动作允许表及动态准入要求 |
| budgets.json | v1硬预算；先耗尽者阻断，不静默截断或清理 |
| examples.json | 30个序列化实例、7个文件字节和10个结构化摘要向量 |
| verify_contract.py | 仅合同检查，无应用导入/业务配置/网络调用 |
| validation.json | 结构、摘要、图关联、预算及27个拒绝检查的结果 |
| manifest.json | 文件SHA256及整个合同材料摘要，供总控入库时核对 |

结构检查与图/摘要关联不变量分层：JSON Schema不能表达跨节点SHA256、头选择成员关系等约束，正文和验证脚本明确覆盖；不能只通过Schema就声称满足并发合同。例子中的payload是合成文本，不是可导入OCXDCONF文件；其分层版本值仅示范现行格式，不是L1新版版本表的采用。

复验需要Python3和jsonschema（版本记在manifest）。安装依赖至隔离环境后执行 `python3 verify_contract.py`，只会重建validation.json；不会联网或运行应用。生产Rust/前端测试须获R1和实际接口后另做。

总控只需冻结该包摘要并发布执行代码SHA、L2独占checkout/逐文件写集、L0网络接口基线和L1 PayloadCodec支持版本注册。实现中的路径/凭据/保存/错误注册继续归L0，Depth1网络文件没有转给L2；生产模型Schema继续归L1。
