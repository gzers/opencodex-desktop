# 0.2.24 单应用包装与官方资源装配

revision: 1；2026-11-11。代码 main318b75e53（装配）、17f608590（发布门禁拒绝旧口径）、2bfca20d1（结尾格式）、09392b38e（发布测试直接入口）；实际进度归IMP-34。不是正式产品候选或发布通过。

## 发行物结构

package_macos_024.py只接受显式已密封单个OpenCodeX Desktop.app，生成新的DMG；根内容只允许该.app、指向/Applications的Applications链接和安装说明，加文件系统自有元数据。用户可直接拖入/Applications，或自己复制到~/Applications；不再要求一层OpenCodeX Desktop目录。Mac运行程序仍由program_layout的当前用户Library/Application Support/OpenCodeX Desktop/runtime供应，不写.app或其父目录，配置路径策略保持。

打包前先检查Info.plist目标版本、codesign严格验证和原.app全树字节/模式/链接/xattrs；核对.app内部固定descriptor pin、完整平台归档大小/SHA和原样official-runtime.tgz的tagged官方SHA。新DMG核验后只读挂载到本事务唯一mountpoint，再次验证布局、codesign、全树及官方资源身份；成功后复核源未变化，以不覆盖发布方式产生新文件。输出/元数据不可在.app中、不可已存在；未知链接、硬链接资源、移动后内容变化或可写挂载拒绝，失败保留日志并清理自己的卷/临时目录，不签补源.app、不去隔离属性或启动应用。

新版辅助工具macos_package_common.py只拥有不可变树/版本/路径/命令记录等共用纯原语；旧023脚本和历史验收原件保持，不把024单.app行为改写成历史已成立。

## 工作流与发布边界

Release装配改用stage_runtime_bundle_024.py与固定official-npm2.99资源，保存平台descriptor/pin/staging身份；不再生成或附加旧023维护运行包。Mac包装使用024脚本，保留原.app/updater签名字节，只替换新DMG。Windows原安装器验证/Server MSI策略准备还原步骤保持，不因Mac成功跳过；Wix当前旧资源名需W1在其独占写集修改，不能声称双端流水线已通。

verify_release_inputs_024.py在资源/网络装配前检查tag只能v0.2.24，实际Tauri/UI/Cargo/runtime policy版本全部一致且official2.99，不按tag猜已改版本。目前实际项目版本仍0.2.23，预检明确拒绝；冻结时由主线统一版本/README/发布说明后再构建，不为包装测试偷改版本或发tag。

现有publish_release.py原先对>=0.2.22都强制旧maintained-runtime，不能让024继续用旧asset过关。当前>=024明确拒绝，直到新的官方集成交付制品验证完成再由主线替换；这不是已完成的发布功能或授权发布。签名/资源验收只补新合同，不能通过删除强度放宽旧门槛。没有触发Actions、tag、Release上传或公开发布。

## 已验证与未完

.adg/evidence/0.2.24/single-app-packaging-r1/：19项包装/024stage/原包装回归及1项禁止套用旧维护发布门禁反例通过，YAML和Python AST解析通过。加入预检时曾生成错误换行正则导致SyntaxError，原失败保留后修正；最终current preflight拒绝已确认是版本未冻结，而不是语法错误。

实际macOS hdiutil/codesign验证使用未执行的简短C签名.app夹具，带当前完整固定官方2.99资源；真实制作267890286字节DMG、只读挂载并核对单.app/Applications/说明、原签名字节和原官方SHA，约23.3秒，通过自己的卷卸载/临时目录删除。它没有编译真实Desktop，不证明真实单.app启动/安装、生命周期或整版签名候选；元数据显式fixtureOnly/productAcceptance=false。夹具.app/DMG在证据摘要保留后删除，无日常安装/HOME/配置变化。

实际产品单.app构建/安装与只读应用供应、Windows安装器资源和W0/W1/W2、NAME、完整原生集成、README/版本冻结及官方新发布验证继续归IMP-34，不关闭P24-01或整版目标。
