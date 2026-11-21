# Mac 私有客户端目录消费原型

若私有 CODEX_HOME 可承载官方 2.99.0 写入并供 ChatGPT 随包的 Codex 后端消费，则在正常 Start 后，每次官方刷新后新建的 stdio app-server 应报告该私有 HOME，且 model/list 对同一 slug 依次返回 A、B、回退 A，人工名称保持。默认目录禁止读写，完整快照四阶段一致，随后原预算内正常 Stop 并清理。

任何阶段读错 HOME、漏模型、名字不符、默认树变化或正常 Stop 超时均推翻本轮判断。本轮限定为后端组件诊断，整版 accepted:false。
