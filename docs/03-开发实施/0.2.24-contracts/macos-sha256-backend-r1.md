# 0.2.24 Mac ARM64的SHA-256硬件后端

revision: 1；2026-11-12。归IMP-34 P24-04，承接[停止校验成本](stop-admission-cost-r1.md)。代码main17ac170c4792ecd6ef35aec1f7d821589dac80a6；accepted=false、非冻结候选，生产MutationPermit/apply仍未开放。

## 原因与独立反证

当前锁定sha2 0.10.9的sha256.rs只在aarch64且启用asm特性时编入上游aarch64 dispatcher，否则使用soft::compress。前批release虽优化整树重复读取，仍按原10秒超时，不能拿debug的一次9997ms当预算解决。

先写假设，使用独立临时Rust宿主，对同一版本sha2/HMAC、release opt-level=s/codegen-units1/LTO，只切换上游asm特性。256MiB ASCII a的连续分块摘要：软件947ms、硬件98ms；两者均为b4a0226ee3f9b159ac06a86332dca0d90a04adef7f88934aa2a75be2a011d504，与Python hashlib独立值相同。空串/abc标准向量、HMAC RFC4231也一致。本机通过Rust标准库真实检测sha2 CPU指令=true。单轮吞吐不是整版性能保证。

独立宿主自有锁解析到cc1.11.1/libc0.2.191，不冒称完全相同的产品依赖图；产品锁不升级原依赖，仅新增sha2-asm0.6.4，复用已有cc1.6.0。真实产品binary另做下述检查。

## 产品行为与边界

只在Cargo的cfg(all(target_os=macos,target_arch=aarch64))依赖项为原sha2启用asm；上游cpufeatures运行时检测后选择硬件SHA指令，缺硬件仍走同库软件回退。不写自定义摘要、不强制全局target-feature，不升级sha2、不改官方2.99或运行包字节。Windows目标cargo tree特性解析明确无sha2/asm；此项不是Windows原生执行证据。

程序全目录inventory、所有角色摘要、独立清单/入口/链接/权限、阶段原件及HMAC验签等原调用保持，所有字节仍读取，没有以摘要缓存/mtime/跳过对象换速度。原30/5/10秒及未知拒绝、无强杀均不变；普通启停仍因完整客户端补偿未完成而保持恢复待处理。

## 验证

产品debug共45项相关检查通过：hash4、程序选择11、完整树3、官方协调6、原生协调10、认证状态9、生命周期签名2；2个显式原包手动入口ignored不计通过。新增256MiB独立分块向量和HMAC RFC4231/修改认证字节及payload拒绝，编译/格式/全目标严格Clippy通过。

新供应的原样官方2.99、全新合成Profile，准确build回执的binary114f658f摘要68a8b7f2225c34c1f6255b9e525b493ba693811d92066ebd175052bdae43fc68与上述45项及运行副本一致。Start8623ms、Status230ms、Stop4775ms（内部4772ms）；正常原PID exit0、4个捕获TCP端口全缺席。停止后协调2794ms、索引RestorePrepared/revision6；四schema4帧/原件/plan及stage/35项内容引用匹配。新collector353ms Stopped+Required、无PID/port且三门槛false。实际catalog/cache/audit仍未补偿，原完整断言保留exit101/accepted=false。

真实release优化级构建和同组45项通过，binary b66c44cd摘要5027f867039818e523d06dbe2505a312c3cb94ae1d726917f6df8e615ba2cb0c与实际副本匹配。第二个全新Profile完整重核同一原样官方镜像：Start5546ms、Status222ms、Stop3559ms（内部3555ms），停止后协调2063ms。原30/5/10秒保持；原PID正常exit0、4项捕获TCP均缺席、索引RestorePrepared/revision6及四frame/plan/stage/35内容引用核对。新collector376ms Stopped+Required、三门槛false。catalog/cache/audit仍未补偿，原完整断言exit101/accepted=false。

此实际样本为后续恢复在同10秒内留下6441ms；证明目前窄夹具的停止/计划准备性能改善，不能外推一般大树/所有客户端图或最终完整恢复已过预算。旧软件后端超时证据原目录不改，成功不覆盖失败。

## 留证、清理与未完

证据归.adg/evidence/0.2.24/macos-sha256-backend-r1/，保留上游源摘要/真实CPU特性、两种后端原输出/独立宿主锁和源码、产品Cargo差异/平台特性、准确build/source/binary、失败原件/索引/frame/plan/stage及内容摘要。运行记录只留PID/port，不保存runtime secret/admin token。

两个原主端点经完整Darwin TCP表framing核对均缺席，自有服务/Node/Bun/launcher/test/probe镜像为空；6个精确新外部协调DB各取/释放EXCLUSIVE后删除，双Profile/程序副本/原件blob/index/stage/测试副本及独立探针Cargo target/软件和硬件binary/CPU helper与重复临时回执均清理。证据逐文件SHA/长度manifest核对后保留。额外TCP缺席来自准确产品trace，不以未导出次要端口列表伪造独立清理回放。Grok合成代理占位值另与原官方发行的5处固定常量来源核对，不是运行secret/admin token。本批不构建新.app/DMG，不发版。完整受控进程集合/持续写者许可、实际恢复、显式恢复及通用图/客户端全范围、旧revision4首错、WindowsW0/W1/W2、NAME、复杂System代理、官方交付发布验证与固定候选继续；局部性能改善不能关闭整版目标。
