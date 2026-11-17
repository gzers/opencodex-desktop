# 独立v2原生DATA附件持久化

代码c3b99c0fa后继完整SHA见status.json，main fcf04caec / docs857fbd672，TEAM28/IMP160/r7；仅新增Windows V2 storage/相关原生测试与模块注册，未改变旧v1实现。完整103项本线状态由status.json延续，局部通过不等整版验收。

每次append先用严格V2 decoder/replay完整验证previous＋candidate，再CREATE_NEW独立v2-record文件；实际file/ID/所含8/bytes/ADS/完整directory membership前后核。目录和每个创建文件在后续fallible检查前登记，所有失败保prefix及原native对象、不重试、不从既有目录重建能力。既有V1 native durability作为实现模式来源，独立V2不调用旧parser、不转换记录。

product01真实完整Tauri库MSVC raw0/145459ms，6过0败0ignored，native测试0.98s，共26个实际新目录：完整canonical DATA链；7个beforewrite拒绝；5写入中断前缀；4截止/非空拒绝；6个成员/安全/ADS及晚变化；3种准备DATA。完整prepare DATA例的身份字段为合成声明，只证明文件持久化严格性，不冒原产品Owner/nativeOriginalSet/恢复完成。未知准备终态拒绝，Pending可原样持久化。

最终完整产品库TEST EXE SHA256 4b5e372f7c50b3b4d3be904a725d2d0b2bb371ee890bb3475d7fb0690ccdebea。4份最终执行源逐Git归一行尾核；全部执行参数/原stdout-stderr/快照保留。无意外product首错；本次续接工具调用/未发布文档merge操作错误另留operational-errors.json及修正结论。112编译warning原样留stderr，不以warning当首错或独立安全审计。

26个新自有fixture根缺席；schema生成raw已归档并还原tracked/移出untracked。旧自动删除拒绝两根、driver失败两根原样保留；synthetic27960身份/处置清理缺口保持，未force或采用旧授权。

完整原生V2 producer/原安全与内容证明、pre-manifest/部分树恢复仍待；原产品Owner returned/proof独立slot复验、过期后只读history/status、版本化可达privatehost/fulltrait及真实固定官方普通Profile全预算仍待。当前r2原Job24raw50仍Unknown；21500/75744/原件/work/custody/旧seq9及40588不动，当前无恢复执行授权。原件未还不重跑官方，publicStart/production/allAcceptedfalse，原预算/未知拒绝/普通token保持，不W2/双SID/发布，目标active。
