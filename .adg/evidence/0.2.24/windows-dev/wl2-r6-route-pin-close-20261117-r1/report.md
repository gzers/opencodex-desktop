# r6原路由pin准确所有权与显式关闭原语

源码b8e06be268182c517a19706dd974961b7e26e57a/tree53f60cff29526bc65034e7cd9f97ed98cd80e915；承接fcc7a688d/e3daa9956，TEAM16/IMP142/r6及共同源不变。

| 运行 | raw | 过/败/忽略 | ms |
| --- | --- | --- | --- |
| library01 | 101 | 47/1/0 | 139002 |
| library02 | 0 | 48/0/0 | 99162 |
| library03 | 0 | 49/0/0 | 114250 |
| product01 | 0 | —/—/— | 62017 |

首轮raw101/47过1败为真实原生反例：fault81先关闭primary config后，余下READ_CONTROL-only观察句柄不能继续denywrite，原raw32断言遇Ok。保留原断言及失败source/EXE/60scene，修独立expected/actual关闭顺序primary最后。首失败用例在probe panic前未输出细trace，此缺口明确保留，不编造/重跑旧前提。末轮加attempt-before-assert trace及ADS veto，49过0败。source verifier早选错new函数，修准确NativeHandleBatch impl anchor；原20函数/文件逐字保持。

RetainedRouteClose先无IO移原RoutePin到retained候选，再核原Token/H/config ID/all8/主流字节及streams，独立fullinventory包含主config、BoundedAccess hidden security文件和两条完整祖先链。准确实际File移入原NativeHandleBatch，所有权错误不drop；close前fresh全部ID/主流/8/ADS/原end。成功所有ConfirmedClosed及原窗completion后允许后继独立config写，旧facts只核immutable路由摘要与自己的完成/原token，不采用新文件字节。

实际partial1、protectedUnknown1、allclosed-before-private-completion、wrongdeadline/data/inventory和ADS场景均保留准确Held/ConfirmedClosed/Unknown前缀，未知不重试/重建File/清保护。ADS确实在移交后创建，close在任何native关闭前InvalidData拒绝，所有Held，原流另导出原字节/hash并原位保留。readonly observation实际不等同主句柄sharing保证。

本轮只是own-read-handle原语，不接production current RoutePin，不产actual Stop/同链许可/typedClosed/后继放行；candidate data始终nativeAuthority=false。下一步同一canonical route-close intent/result＋原actual Source/Owner/action/content/normal/supervisor＋final parent lineage，不能Drop作完整release。原包生产副作用/初始化/ownerreceipt/原缺席Codex安全退根/最大batch容量/普通MSI/CAP-trust/历史清理保持未完，Start拒绝。

185scene私有ZIP/hash/membership核验后全部保留（含首失败/protectedUnknown/nativepending/ADS），66份canonical/快照/history另收存，本轮无删除/历史拒绝重试；ZIP非NTFS身份安全或ADS克隆，ADS原字节另导出。最终源对工作/Git明确CRLF等价，产品192输入/最终test146Rust，预存schema未手改/暂存。不改官方/日常/PATH/政策/账户，不旧raw5/空GUI/W2/双SID/发布。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77809152 | d2133ec56a509ac277ba08375223036b26a67d75cfaa46117839e424a67c322e |
| ocxd | 5398528 | f610028184290d236f2300c1a90c799b84bbe46fd049fbcf810539851ce068ce |
| ocx | 306176 | bc14a410faea55ae3638d9cf76b80a1e051c1129ea07a605b600c100476fd4d4 |
