# Windows可取用补丁（当前入口）

- 仓库gzers/opencodex-desktop，分支codex/post021-runtime-patch-handoff。
- 提交d5a6aaf991262ef6205e7dd68cefae69d62033d8。
- 文件test/smoke/post021-windows-final/patches/runtime-owner-dev-0a30c551.patch；35,703字节。
- SHA256：0d8eaaec8c361d914fb901db2c8ef6a36702ce3c7ec209cb3f255817f8e32bb7。
- manifest.json同目录含原/后像摘要、上游固定dev和未完项；不在管理器仓库应用补丁，须在隔离上游checkout核对基线后使用。

在Windows现有仓库fetch该分支后，用git archive --format=zip --output=runtime-patch.zip origin/codex/post021-runtime-patch-handoff test/smoke/post021-windows-final/patches，再用Expand-Archive解压及Get-FileHash核验；避免PowerShell文本重定向改写patch编码。Git blob与ZIP提取均已复验原SHA。用户仅授权可取用贡献分支，本次没有创建上游fork/PR/issue或发布。旧/private/tmp路径仅保留本机溯源，不再是唯一交付位置。

---

# 本地运行包补丁预备结果

私有补丁路径、SHA及十个变更文件摘要取runtime-owner-dev-prep-review.json；不把补丁源码复制为本项目实现，不表示它已提交上游或已成为正式依赖。补丁在隔离目录准备，未创建fork/PR/issue，未发布或改用户运行包来源。

八个修改前像逐一匹配固定dev提交0a30c551的Git blob，两个新增路径不存在；十文件git apply检查/回放及后像摘要通过。dev完整归档下载unexpected EOF后未解包、未重启；本地使用完整main快照cab7eb7的未变周边，涉及文件与dev逐一核对，expected-layout基准换成精确dev原件保留其它新增条目。Bun lock/tsconfig相同，package仅版本字段不同，不能据此宣称完整dev分支CI通过。

Bun1.4.4类型检查2306文件；三项定向套件109通过/3 Windows专属未执行，兼容启动套件12通过/2 Windows专属未执行。新query构造器保留base64命令行上下文，真正嵌入源码；禁止物理源码目录读取与网络后，编译小程序和解释执行输出摘要一致。该验证仅是构造器，不是完整运行包/Windows原生。10变更文件的上游scanText无发现，Git索引依赖的整库privacy/structure/layout和完整上游CI仍未执行。

原5项失败有压缩原件：旧查询字符串断言和将任何CreationDate视为起始时间查询的夹具分流。现按生成的真实native SID/path/time查询及未知/空语义调整；Windows合成成员测试替换原生身份类型、仍执行完整PowerShell选择策略，不枚举或控制真实进程。该平台测试在Mac未执行，不能写成Windows通过。旧2.84运行包候选的Windows通过继续独立，不能代替这个移植版本。

接续：用户授权对外贡献后，以最新dev建立真实上游分支，采用/重放可审阅补丁，完成所需原生/索引及上游检查，按上游模板提交草稿PR并由维护者集成发布运行包。当前没有对外授权，不自动发消息或提交PR。正式运行包可安装后仍须本批真实默认重启/发行原包门。
