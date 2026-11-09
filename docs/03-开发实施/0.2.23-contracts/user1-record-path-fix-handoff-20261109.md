# 第13项：lifecycle.3 路径修复单用户验证

2026-11-09，Asia/Shanghai。用户已采用直接修维护包路径校验、构建推送并交Windows一次user1验证。本交接覆盖[输出诊断交接](user1-output-diagnostic-handoff-20261109.md)中继续使用lifecycle.2的下一动作；诊断accepted:false，尚未关闭启动、停止或版本验收。

## 修复事实与固定来源

新代码：codex/0.2.23-integration@d3b34a234e32b63365672052772c2c72c719f97c。文档：docs/governance-main，以含本交接的实际远端提交为准。

Windows最新回交4712139d6／ocxu1-20261109-205318已核对73条manifest记录。新Desktop确实运行，stdout/stderr读完无读取错误，首个失败为私有记录模块公共refusal；实际HOME为Windows扩展本地盘符路径。原校验直接拒绝此前缀，已用相同归档函数和实际路径复现。此证据证明一个确定阻断；尚不代表修后正常启动与Stop已完成。

本批只在维护包私有运行记录校验中允许扩展前缀后接合法盘符根的路径。原路径不改写；设备命名空间、扩展UNC、驱动器相对路径、穿越、ADS流、尾随空格／点、保留名等原拒绝条件保留。原ancestor pin、owner/DACL、CREATE_NEW、文件身份和原子发布函数保持原字节。既有普通路径行为不扩大／缩小，不修改旧文件权限，不加旧目录兼容。

- 运行包：2.97.0-desktop023.lifecycle.3。
- 归档：opencodex-2.97.0-desktop023.lifecycle.3.tgz，15413397字节。
- SHA256：ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739。
- Git可取原件：[新运行包及证据目录](../../../.adg/evidence/0.2.23/lifecycle3-record-path-20261109/manifest.json)，归档位于同目录。不能把Mac临时目录当Windows交付地址。
- 构建入口仍为.github/scripts/build_runtime_lifecycle_023.py，从固定ownerquery.2逐层重建；双平台runtime policy、包锁定完整性及单用户采集摘要已同批更新。旧lifecycle.2的原件、摘要和失败批次均保留。

## Windows只执行这一轮

1. fetch代码和文档，核对完整SHA。准备新源码、lifecycle.3的完整Desktop程序；现有windows-candidate.yml只选择purpose=core-build，核对实际run.head_sha，封存新run/artifact ID、ZIP大小／SHA256、core-build.json全成员、实际运行包摘要。**不能使用上轮42619237040／13676565494或任何lifecycle.2 Desktop包。** 构建失败原样回交，不改判据或自行追加产品修复。
2. 新鲜批次目录、一个临时普通user1、真实Profile和可用桌面，记录实际Windows版本／SID／令牌／RDP或控制台环境。test/smoke/windows023-user1-start-stop.ps1显式传-ExpectedSource d3b34a234e32b63365672052772c2c72c719f97c和-ExpectedBuildRun本次真实构建run。只调用一次实际Desktop Start，成功确证后立即正常Stop，再精确清理。
3. 回交仍为四项：启动原回执／数量／唯一性／PID端口一致性；启动trace／startup_output及首错；正常停止trace和真实退出码；账户／Profile／用户目录清理结果，残留时列PID／镜像／SID。启动未成功则正常Stop未执行，不能以强制清理冒充Stop通过。
4. 原30秒启动／10秒停止／5秒查询以及归属、健康、实际还原、清理判据不变。若有新失败，保留原批次并一次回交所有已观察事实，不自动重跑；具体阶段未知保持未知。
5. 用最新windows023-user1-evidence.py采集，传本轮--artifact-id、--artifact-zip、--document-ref；environment应显示lifecycle.3和上面新摘要。代码与证据沿各自Git分支回交，独立目录.adg/evidence/0.2.23/user1-start-stop-20261109/<新批次>/，原失败不覆盖。

不跑core-sweep、双SID、安装器、全量、旧失败轮次、真实业务同步或发布；不碰日常配置。第18项不先独立修，也不因本次清理通过关闭历史失败。Mac没有派Windows原生执行；收到新四项证据后再判断后续修复和收口。

## 派发前判断及本机验证

待证伪判断：原来的扩展本地HOME拒绝将被解除，启动应能进入运行记录创建与就绪阶段；最终是否在原预算内通过Start／Stop，以真实Windows回执为准。如果仍未生成记录或出现新失败，本轮判断不完整，不能重复同一轮或写成已解决。

本机已完成实际旧／新归档比较：只一处源码路径guard变化，另两处为package及兼容摘要；9个路径用例、漂移／重复补丁拒绝、5个装配测试、2个采集器测试和PowerShell7 AST通过。两次独立规范打包同字节，所有兼容清单现存成员及双平台policy／锁定摘要一致。原继承缺失bun.lock与scripts/model-metadata.source.json保持原报告，不算验证成功。没有改变Rust/Desktop源，不因此重跑Rust／UI全量。证据见[verification.json](../../../.adg/evidence/0.2.23/lifecycle3-record-path-20261109/verification.json)。
