# 0.2.24 npm代理隧道、远端DNS与原生字典验证

revision: 1；2026-11-12。归IMP-34 P24-03，承接[npm静态系统代理](npm-system-static-r1.md)。代码main 4a7980904549c64c16511092707b7f14a8450004；accepted=false、非冻结候选。本批补已开放传输协议的实际证据，不新增PAC或分协议路由能力，不宣称完整System完成。

## 为什么增加这批检查

前批固定npm实际五场景只使用HTTP明文registry，尚不能证明已准入的HTTPS及socks5h实际工作，或代理场景仍保持证书校验。因此本批只验证新的相关协议，不重跑前批plain HTTP五场景、完整服务启停或Windows验收。

新增npm_proxy_transport_tests通过生产SystemNpmRunner/base_command和冻结系统快照fixture，运行原样Node26.8.1/npm11.19.0。目标为registry.invalid，代理与合成registry只监听loopback。所有env清空/私有HOME/cache及原用户配置不变检查保持；没有真实registry/package下载或pack/install。

## 四种真实传输结果

| 场景 | 实际观察及断言 |
| --- | --- |
| HTTPS经HTTP代理 | 代理收到CONNECT registry.invalid:443；将原TCP字节转给自有TLS registry，npm获取合成2.99.0，exit0。 |
| HTTP经SOCKS5h | 代理收到SOCKS5 DOMAIN类型及registry.invalid:80；没有把本地解析IP冒充远端DNS，npm读取成功。 |
| HTTPS经SOCKS5h | 原SOCKS5 DOMAIN携带registry.invalid:443，隧道内仍走TLS，npm读取成功。 |
| 相同HTTPS隧道、不信任测试CA | strict-ssl仍true，未设置禁用校验变量；npm exit1并明确UNABLE_TO_VERIFY_LEAF_SIGNATURE。代理建立连接不等于TLS被接受，未获取registry响应。 |

正例使用一次性合成CA签发serverAuth叶证书，SAN为registry.invalid，仅以本次npm --cafile加入测试根。负例不传cafile，并保持原用户npmrc strict-ssl=true。没有改系统钥匙串、全局CA或npm全局设置。server私钥/CA私钥均为本轮生成，未作为证据留存，结束后已删除；仅保留公共证书及摘要用于审查。

HTTP代理和SOCKS握手均是明确测试服务器，不是新生产forward proxy；生产仍由npm原代理库处理。SOCKS remote-DNS和CONNECT记录来自真实协议字节断言，不从配置字符串或env存在推断通过。TLS拒绝代码由npm原stderr提取并限制字符后输出，不留任意secret或上游body。

## Mac原生配置解析的直接验证

原SystemConfiguration读取仍调用同一解析算法；将该算法提取为macos_map，便于使用真实CoreFoundation字典/数字/字符串/数组检查类型，不变更运行语义。2项平台检查覆盖：HTTP/HTTPS共同代理、IPv6 SOCKS5h回退、真实CFArray例外与保存bypass合并；错误host/port/flag类型与值、PAC/WPAD、简单主机排除、错误数组/元素、CIDR、不同两协议代理仍拒绝；明确空字典无代理。测试只建立内存CF对象，不写系统dynamic store，也不伪称本机原系统启用了代理。

该验证不替代Windows WinHTTP实际调用及本机真实已配置代理的端到端验收；Windows原生运行仍待最终对应检查。完整PAC/分协议/复杂bypass仍按原计划继续，不能用这些静态正例扩大支持声明。

## 来源及检查

最终准确源码下3个去重测试函数（2字典＋1传输，传输内部4场景）通过；初次传输已通过，后继提取Mac解析、加强负例证书分类及增加明确npm错误码输出后，用最终同一binary再次验证受影响4场景，未重复其它已过范围。首Clippy发现测试read未处理字节数，改为read_exact、不禁lint，原stderr保留；最终编译/格式/全目标严格Clippy通过。

.adg/evidence/0.2.24/npm-proxy-transports-r1/含原假设、两次传输结果、最终CF检查、准确source/binary、原lint、公共证书/生成配置/摘要、固定工具来源与manifest。初次binary在Mac解析提取和增强负例前，最终完整源SHA单独记录，不能把初次运行冒充最终源码执行。ignored入口由本轮显式OS宿主执行成功，不把默认ignored当通过。

OS sandbox实证拒绝外网与日常HOME，仅允许loopback及自有fixture写入；自有Node/npm/测试/代理/registry进程为空，复制工具、profile/npmrc/cache/binary、CA/叶私钥和逐字节核对的重复证据根均清理，原工具和日常配置/安装不变。未改官方2.99或系统代理、无模型/WebDAV请求，未启动OpenCodex服务或构建新.app/DMG，不发布。

实际客户端恢复/完整进程集合监听写者、停止后revision4首错、WindowsW0 HOME/Profile/W1/W2、NAME、System复杂路由、官方新发布验证与固定候选仍保持原范围；本批只补静态代理传输证据，目标未关闭。
