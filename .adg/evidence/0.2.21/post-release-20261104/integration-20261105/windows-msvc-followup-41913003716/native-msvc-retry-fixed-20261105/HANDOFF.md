# 新MSVC候选重试修复原生回交（2026-11-05）

代码6a3a64f02792e0756340bd3fbb5d817f67888a43已推送codex/post021-windows-final；产品修复4e6614ee316bf5391dfe5be38ae0b490e1330fa3，其后仅test/smoke驱动变更。草稿PR70不合并、不发布。性能与正式客户端默认重启仍未完成，TASK保持in_progress、RCP总体failed，无CERT。

## 精确新候选

run41944607861 / artifact13380187718。ZIP SHA-256 5fb674d9219bf08cc120ac509e5b391c0e2ce4d1d95d4fb73f40acbe93005288；EXE SHA-256 fa9df83a55153f764673781e85167093d20602fd2e34742c69f6b2f5b37f9c68，15,293,952字节，PE32+ AMD64。构建d9de5034dad798c89faabff6230ecd8a9b388ead，requestedHead e9e7fcaa401fc14b2890bb9a3df5bbaff291dfbe；apps树a8bb18f9ff2781dbbfe67f91473a298f3cd74ae6与当前代码一致。Git对象、下载/复制SHA、实际startup内嵌SHA均已核验，未重建冒充该候选。

新隔离根post021-windows-final-msvc-retry-41944607861/fixed，真实官方2.84服务仅在自有loopback60444/60446；客户端集成/自动同步关闭，正式重启钩子直接拒绝。管理器仍显式指向status-only测试状态探针，非日常CLI/服务。

## 根因与有效修复前后

旧MSVC候选41913003716在全新panel根跑严格反例：真实/web响应头被测试服务器延迟60秒，/healthz仍健康；首载约8001ms按原8000ms超时。恢复服务器gate后点击生产“重试加载”，8秒内未恢复，服务器只有1次/web请求。native-retry-negative-20261105包含脚本失败、实际请求、原生失败截图和日志，不把旧请求60秒后自动完成当通过。

WebView2在首个导航尚未收到响应头时，对同一视图Reload不能摆脱原请求等待。修复在显式reload且原生ready=false时关闭旧视图并重建，保留视图epoch/generation与origin过滤；新建视图不再重复Reload。已就绪面板正常复用，未动启停/同步/字号/布局/焦点与浮层退出守卫。

| 新MSVC原生检查 | 结果/证据 |
| --- | --- |
| 同一60秒初始导航延迟，8秒超时后恢复gate并真实点击重试 | retry/panel-retry-navigation.json通过，约645.586ms恢复，低于原8秒限；新HTTP/web请求在retry后实际出现 |
| 视图退役与来源 | native-app.log有retry; retired pending navigation、第二次creating embedded view、page ready；内嵌commit d9de5034d |
| 不重启服务、不改业务配置 | retry前后同一服务PID/port，config字节一致 |
| 旧60秒导航迟到 | retired-navigation-readback.json通过，越过原60秒后仍同一新trace/visible-ready；原脚本复制的GNU-only标签已纠正，原JSON另留retired-navigation-original-label.json |
| 60444正常及快速重开 | reopens60444/panel.json通过，2次正常重开及10/30ms早离开重开 |
| 60444浮层Hide未返回即关闭 | overlay/panel-hide-race.json5次通过，真实传输提交→public close→native Hide结果→Show visible-ready，不改回包或加延迟 |
| 60444→60446切端口后快速与正常重开 | reopens60446/panel.json通过；config观察前后字节一致 |
| 60446同类浮层恢复 | overlay60446/panel-hide-race.json5次通过 |
| 原生画面 | native-retry-restored-msvc.png为已恢复官方首用说明弹窗；native-dashboard60444-msvc.png、native-dashboard60446-msvc.png为收起说明后的实际面板。未改变官方子代理设置 |

所有有效新候选结果在native-msvc-retry-fixed-20261105。旧MSVC、GNU及调试失败保留为不同目录，不能拼成一个候选或认证。

## GNU诊断边界与更正

为了不空等CI，临时git archive副本只将crate-type改rlib避免GNU DLL导出258649超过65535；Rust212个文件与提交blob按Git canonical换行核对，Windows archive CRLF转换明确列出。最终Common Controls manifest另适配，内嵌commit=unknown；因此始终不是正式MSVC替代。

初始GNU临时组装复制了未重新构建的UI dist：Rust重试约424.728ms通过，新HTTP请求/服务PID/config证明具调试意义，但浮层5轮没有Show，不能归于最终产品。后重新typecheck/Vite、复制24份新产物再重链，原生前台未准入（用户其它应用在前台），保留拒绝记录并清理，未冒充通过。最终MSVC本身全部6项严格检查通过，才采用上述正式结论。生成物/manifest/GUI影像的身份局限均保留。

原录制器只观察postMessage，后来增加custom-protocol fetch观察；原空trace/缺回包/未就绪准入错误保持，最新工具不允许loading状态开始5轮已加载面板恢复场景。重试严格脚本已经旧候选真实拒绝、新MSVC真实通过，不靠伪造IPC或弱化断言。

## CI真实状态与未完

候选所在run41944607861的Mac后端、前端197文件1291测试、release-tools、Windows完整后端及安装恢复/Node22/普通账号均实际成功，原始日志已保存。windows-smoke-evidence最初Controlled parent did not become ready失败；父run结束后仅重跑该job请求201受理，新job126995179947已completed/success。该候选轮最新attempt五项适用job均success，build按原条件skipped；ci-jobs-after-smoke-rerun.json及ci-smoke-rerun-success.log为终态，初次失败原件继续保留。最新工具头1d40的run41945970332其Mac库1215通过/1失败/4忽略/1已单跑排除，失败accepted_panel_query_commits_success_or_failure_after_observer_abort（0≠1），原日志留存；该轮Windows仍执行。不得把旧同源通过抹掉这个失败，待终态按原检查补验。

性能仍原预算失败，没有可接受生产修复：静止/滚动/弹窗约181.2/142.2/174ms，high/真实玻璃及B=1000/60、p95≤2B、over6B=0不变。新增OpenGL请求实际回退Microsoft Basic Render Driver，约166.7/136.3ms仍失败，不采用；仅取消帧率限制的尝试在Esc取消后前台准入失败，未测得有效结果。先前滤镜/合成/驱动对照与原始trace均在connected-native-20261105；它们只有GPU/compositor等待定位，不能认证根因或修复完成。

正式Codex默认adapter隔离重启仍缺一次性当前用户包实例/Windows会话，只有日常同包实例，不执行；不以CODEX_HOME、OwnedClient、GNU或官方代理面板替代。云端首次独有对象精确并集、后续新增不扩选和明确子集保持已有原MSVC证据，产品仅panel.rs变更，未改同步业务。

所有新测试服务/管理器按绝对路径、PID、创建时间、session/owner及原生句柄精度核对后停止；cleanup-final.json确认测试根无残留，日常MSIX38560及管理器24140仍存活。只认证本轮受控清理，不抹除历史日常Bun30196误停事件，不自行补偿重启或改日常配置。
