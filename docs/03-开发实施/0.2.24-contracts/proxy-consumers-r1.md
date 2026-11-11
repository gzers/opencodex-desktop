# 0.2.24 更新与npm消费者的免代理接线

revision: 1；2026-11-11。归IMP-34 P24-03 / POST-06，承接network-notes-feedback-r1。实现main80afa4418af7d255c8823e5cdecb026fbb03412c，定向检查通过；不当完整代理平台矩阵或最终集成候选验收。

## 更新器检查与下载

先前说明查询/连接诊断已经使用保存的免代理项，应用更新器仍只调用proxy/no_proxy而遗漏networkNoProxy。本批从同一保存偏好快照取policy和bypass，使用更新插件configure_client钩子，插件版本检查和已签名制品下载都会调用同一钩子，地址/通道/签名验证和安装预约仍归原更新器。Direct明确清除环境代理，Manual只用已验证http/socks5h地址加免代理项，不继承环境代理或失败后隐式直连；System不写入no_proxy，继续交库的系统/平台选择。

tauri-plugin-updater精确固定2.13.1，其configure_client类型使用reqwest0.13，显式同版本reqwest_updater=0.13.5并启用socks；没有新增另一个网络栈版本或升级传递依赖。rustls测试依赖使用插件已有0.23.45/ring，独立传输夹具复现插件首查前原有的crypto provider初始化，生产插件自身逻辑不变。地址无主机、凭据、其它协议、路径/查询/片段明确拒绝，不退回直连。

## npm安装与元数据

SystemNpmRunner持有本次命令边界捕获的no_proxy，base_command在清空继承环境后设置NO_PROXY及npm_config_noproxy；原用户HOME/USERPROFILE、显式缓存、受控PATH保持。联网pack与候选依赖install消费同一base_command，手动代理/临时凭据userconfig仍按原请求处理，凭据不进入argv。versioned与旧受控安装编排都传入当前保存的免代理项；不写用户/global npmrc、系统代理或用户环境。

npm view的受控元数据查询继续使用EnvironmentPolicy的HTTP(S)_PROXY/NO_PROXY，同时明确提供npm_config_noproxy，避免npm自己的配置层忽略免代理值。本次改动只闭合保存的Manual/Direct免代理传递，不声称npm能自行读取Windows/Mac OS的System自动代理；该平台能力若需要新的原生解析仍属待闭合。WebDAV端点代理、模型上游代理不混入此设置。

## 实际验证与限制

证据 .adg/evidence/0.2.24/proxy-consumers-r1/：31项去重相关后端检查通过（updaterProxy3/npmEnv1/versionedInstall4/metadata7/真实NodeNpm1/managerUpdate15），格式及全目标严格Clippy通过。新增更新器传输测试使用真实reqwest0.13钩子策略，以loopback HTTP伪代理收到metadata/artifact两种实际绝对URI，并在保存免代理命中时越过不可用代理直连真实loopback服务；Direct清除预置代理、System保留平台客户端选择，非法Manual不触发回退。它不下载GitHub真实更新或绕过签名进行安装，不当完整插件GUI验收。

真实固定Node26.8.1/npm11.19.0只对loopback合成registry执行view读取，原SystemNpmRunner构造环境，分别证明代理实际收到请求与免代理直连，读取到合成2.99.0，不执行pack/install/lifecycle或访问真实registry。私有合成npmrc保持原字节，HOME及缓存独立，输入工具复制并按摘要保留身份后清理，不改原工具或日常配置。

首轮独立传输夹具未初始化插件自己会安装的TLS provider，保留panic后补齐真实前置；Clippy大枚举通过Box缩小不抑制检查。旧npm超时回归在OS sandbox中无法spawn系统ps，原失败保留；该单项仅用已审阅临时shell输入/私有HOME/TMP和PID限定ps读回，在无OS sandbox的独立进程中执行，原超时/退出断言保持，未知不算通过。不放宽全组隔离或改产品退出判定。

生产无界面布局变化，未重复页面测试。原管理器更新签名/通道/候选/在途所有权回归与版本化安装保护回归保持。WindowsW0/W1/W2、NAME、System代理平台解析、正式完整服务/客户端恢复及新官方交付发布验证仍按IMP-34继续，当前工程Mac包早于本提交，不冒称最新候选。


最终Direct清除既有代理、System不重写客户端选择、Manual保存例外在两种实际请求路径均验证通过。命令元数据夹具实际检查NO_PROXY与npm_config_noproxy相同且用户HOME/缓存保持；签名/通道/候选/后台写者与版本化安装保护无放宽。测试工具复制品、所有私有程序/配置/缓存根和重复临时回执已清理，原失败按目录保留。没有前端布局变更，无理由重复共享页面验证；最新Mac工程.app/DMG仍是本提交前输入，最终候选需要重构。
