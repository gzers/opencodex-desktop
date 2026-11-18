# r7 只读共享修订的候选摘要配对

输入 Windows 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7、main 040f411e6d48d97bde6e7b2e6c22860e547b6173、治理 534e392dd54295d760c28a33a837ef8ec7975e73（TEAM67／IMP206／r7）。复用 70c6d7f4f972ff93edd3d8b35ace4e5aad25cfe5 的合成 V2 整对读回及负例。

待证伪判断是 launcher 可把兼容读共享限制在 CheckAttachment 的单一调用点，计时入口只须换该 launcher 的摘要，原 CMD pin 及拒绝行为继续适用。脚本先校验三个固定文件的原始字节。对内存草稿只替换 ExportExact 的参数和只读 File.Open 的共享参数，以及 CheckAttachment 的调用；其余调用仍通过默认 Read share。计时脚本只替换一个摘要。候选仍待 Mac 固定。

| 文件 | 候选字节数 | 候选 SHA-256 |
|---|---:|---|
| launch-r7-partial-owner.ps1 | 30158 | 6b553204560d59ba7a292f64c322404028eceb1a0899661f5f9dc9d76953e990 |
| observe-r7-readback-timing.ps1 | 6394 | f3a0c5ed9f8ba698d0b5d3e37d1b1209b4b88fa5ee4714a5fe06c1ff000aba5f |

PowerShell 5.1 解析四份原脚本与候选脚本的文本，从原观察脚本和候选观察脚本提取准确摘要门。两套同版通过；两种跨版及修改一字节的 CMD 被原错误文字拒绝。整个离线检查 raw0，stderr 为空。草案未物化，固定文件和真实回执目录未写入。此证据仅覆盖摘要绑定与改动范围，真实 Host、原 worker、V2 持久化和 5 秒读回仍待现场。

复现时以 Windows PowerShell 5.1 执行本目录 check-pins.ps1，参数为 -Fixture C:/Users/Public/ocx-r7-isolated-owner-range-r1。stdout 中给出变更的准确字面量及每一项判断。
