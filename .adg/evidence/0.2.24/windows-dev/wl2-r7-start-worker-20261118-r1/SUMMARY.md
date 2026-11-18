# r7 Windows 首启原工作线程

源码 7ca43ca47bb642a91e8c69444afaad673582a213。在 TEAM46／IMP185／r7 已同步的基线上，首启在首次原生获取前登记唯一 Owner 与专用线程。失败先撤销 Start 权限，再由原线程保留已取得的句柄和范围。Status 走独立的有界投影，记录阶段、已知原始码与 Resume 状态。

原生范围 5/5、raw0，144149 ms 含编译，测试 0.95 秒。四种真实 SQLite 冲突均见 raw33，19 个本轮夹具已核实缺席。首启控制层 2/2、raw0，139262 ms 含编译。检查 Status 竞争／失联、等待端断连后保留首错、拒绝迟到成功。真实持锁 worker 与显式同范围恢复仍待验证。两个原始运行目录及执行源码摘要均保留。执行副本留在代码工作树的测试输出目录。

控制层测试 EXE 为 102178304 字节，SHA-256 11dd88f897ab5cc1fe141080c380d4bbeddc4283e56b65659bd3f1483ac9df70。两批复用制品路径，第一批在记录摘要前被第二批覆盖。artifact.json 仅确认第二批制品。两批都没有测试断言失败。

r2 本轮只比对十份已归档原始回执的长度和摘要，共 28777603 字节。原归档 10701 文件、1664 目录，SHA-256 c3906f7814f48c00f88ce16f6de6fbdc55b6bdffcaf2f2c61f3af9a67c6c0a。保留原 command-lineage/raw64 及后续映像／参数一致记录。原 Job 仍有 24 项未知，派生零权限复制 raw50。executionReady=false，保留原件、现场与既有审批拒绝项。

仓库代理 127.0.0.1:3060 的 HTTP CONNECT 成功，Git fetch 分别在 OpenSSL 和 Schannel 的 TLS 握手失败。源码本地提交，远端状态待确认。完整状态见 status.json。公开 Start 持续拒绝。

产品 MSVC --bins 在 a5ba9aee338ed69841a836e4a5be639115fa8e1c 完成，raw0，113102 ms。包含派发早期失败时原 Owner 留槽及普通动作拒绝。三个二进制摘要见 product/artifacts.json，尚待运行验收。
