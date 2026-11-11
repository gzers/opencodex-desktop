# 0.2.24 npm系统静态代理与直连配置覆盖

revision: 1；2026-11-12。归IMP-34 P24-03，承接[代理消费者](proxy-consumers-r1.md)。代码main 52600795246b596af49dad29fd2c41e47ddd2ada。本批接入可等价映射的静态系统代理，accepted=false、非冻结候选；完整System路由、Windows实机和其余整版范围保持未完。

## 当前行为与消费范围

新增network_defaults::npm_proxy::Snapshot，在安装worker和面板版本查询worker各自捕获一次已保存策略与当时原生系统配置。None/Manual不读取系统代理，System才查询；不继承外部HTTP_PROXY/ALL_PROXY或用户Shell配置，不写系统代理、npmrc、全局npm配置或用户环境。

Mac使用原生SystemConfiguration dynamic store，直接消费HTTP/HTTPS/SOCKS启用、主机/端口和例外；复用已锁定system-configuration0.7.0为直接依赖，未升级传递版本。Windows使用WinHttpGetIEProxyConfigForCurrentUser读取当前用户，包括AutoDetect/PAC、静态路由和bypass，原生字符串有界UTF16解析，RAII GlobalFree；不从其它用户注册表或HKLM猜值，不读取系统代理凭据。

Snapshot只准入两协议相同的静态HTTP或socks5h代理，或两者明确不使用代理。原生值先校验scheme、host/port、路径/查询/凭据字段，非法或不可读直接返回npm_proxy_unavailable，不回退直连。系统例外的可映射域名后缀规则与保存的应用noProxy合并。PAC/WPAD、HTTP与HTTPS不同或只有一侧代理、CIDR/任意通配符/排除简单主机名/Windows <local>等无法用当前npm规则等价表达时，明确拒绝本次npm请求。它们仍是需要后继完成的System功能范围，不把拒绝当支持完成。

联网pack和候选install共用同一受控base_command；离线来源不触发系统代理解析，原--offline保持。显式一次性安装代理优先于保存策略；带凭据仍只进原0600临时userconfig，不进argv。应用快照的代理环境在此显式请求时移除，避免覆盖原临时凭据文件；无凭据请求同时指定--proxy和--https-proxy。HOME、受控PATH和独立cache保持。安装结果proxy_used沿既有含义记录配置了代理，不声称每个命中bypass的请求也走代理。

面板npm view使用同一快照投影到原EnvironmentPolicy，再设置明确npm配置；原远端版本/完整性、超时、受控Job/查询日志和候选准入不放宽。WebDAV及模型上游代理不混入该设置。运行程序恢复/内置离线供应仍按其原离线策略，不为System额外联网。

## 真实npm暴露并修正的问题

原包npm-registry-fetch实际把httpsProxy || proxy用于registry请求；@npmcli/agent对HTTP还优先取HTTPS_PROXY，noproxy只做域名分段后缀比较。不能仅设置两组环境变量就声称已支持不同协议的系统代理，也不能把CIDR/PAC逐字传过去就算保留了语义。本批据此显式拒绝不能表示的配置，后继需完整路由方案，不能通过默认直连或抹掉系统例外凑通过。

真实固定npm11.19.0验证中，又发现空npm_config_proxy/https_proxy被npm环境加载器忽略，继续采用合成用户npmrc中的旧代理，Direct实际连接127.0.0.1:9失败。原失败stdout/stderr保留。生产改为npm认可的proxy=false、https-proxy=null，同时清空HTTP(S)_PROXY；空noproxy使用可解析为空字符串、但不会被env加载器丢掉的单个空格，覆盖旧npmrc bypass。真实最终反例包含旧proxy/https-proxy及noproxy=registry.invalid；五种请求均按本次策略路由，原npmrc保持原字节。不是只断言command.get_envs就宣称直连成立。

## 验证及边界

18项去重相关检查通过：3项策略/拒绝/Windows字符串解析、1项npm环境、1项凭据argv、1项临时配置保护、6项元数据、4项版本化安装事务、1项真实npm（内部5场景）及1项实际Mac原生只读查询分类。Windows native API另经x86_64-pc-windows-msvc独立cargo check，直接引用最终共享模块，但错误/偏好/环境类型为结构stub；它只证明WinHTTP调用及类型可编译，不是整个Desktop构建，更不是Windows运行测试。初次versioned筛选实际0项不计通过，改为正确versioned::tests后4项通过；已有显式真包安装/远端live tests未运行，不计验收。

真实固定Node26.8.1/npm11.19.0由既有不可变运行归档复制到隔离工具目录，按原SHA核对，不修改原工具。只向loopback合成registry执行view读取，五场景为Manual代理、Manual bypass、System快照代理、System快照bypass、Direct覆盖旧npmrc；未执行pack/install或访问真实registry，没有模型/WebDAV请求。System有代理场景注入的是明确合成的原生快照，不改变Mac系统代理，也不冒充该OS真实配置下的路由测试。

Mac实际SystemConfiguration只读检查成功，当前无HTTP/HTTPS代理；这只证明当前原生读取及分类，不证明所有配置字段组合。尚缺可配置原生快照的Mac路由实测与Windows当前用户实际执行。新CLI/原生网络错误反馈及最终候选仍需对应范围验收；本轮未改界面布局，不重复共享页面测试。

首编CFArray泛型不满足ConcreteCFType、Clippy多余借用原错误保留；改用正确未定型CFArray及逐项CFType安全类型判断，不禁lint。首真实npm两项Manual通过后，System场景测试服务器在非阻塞accept所得stream读取时WouldBlock，保留首错，将连接明确切回blocking，再只补未过System/Direct；随后Direct真实产品故障按上节修复。最后因产品Direct/noproxy逻辑变化及新旧bypass反例，重验完整五场景，全部通过。早期zsh通配重定向及两次launcher相对binary路径错误仅原工具输出/分类，未捕获独立raw stderr文件，不能声称所有工具首错原字节已归档。

最终编译/格式/全目标严格Clippy通过，准确源码/两个阶段binary与原失败按目录保留，末轮源码和提交10件逐件SHA一致；跨编译清单/结构宿主单独标注，不作产品事实源。

## 证据与清理

.adg/evidence/0.2.24/npm-system-static-r1/保留原假设及后继限制判断、相关官方npm源摘要/合同片段、失败/最终路由输出、原生读取分类、Windows仅编译记录、源/binary SHA、manifest和cleanup。失败临时npm配置自身随TempDir作用域清理，原字节输入由最终源码固定；输出日志保留，不伪称已归档每次临时cache原件。

OS sandbox实际拒绝外网/日常HOME读取，只许loopback和自有fixture写入；自己的Node/npm测试及server进程为空，复制Node/npm/工具编译target、私有profile/npmrc/cache/测试binary和已核对的重复回执根均清理，工具原摘要未变。未改日常配置、Shell、正式安装、官方2.99或系统代理，未构建新.app/DMG或发布。

完整System的PAC/分协议/复杂例外与Windows实机仍待处理，不能将P24-03整项关闭。Windows贡献仍b2e89665d/0eef59a01；W0 HOME/Profile/W1/W2、NAME、实际客户端恢复/完整进程集合/监听/写者、revision4停止登记首错及官方交付发布验证/固定候选继续，目标保持active。
