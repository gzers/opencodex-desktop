# 0.2.20默认字号与验收补正


### 2026-11-03 默认字号收紧与原型验收依赖补正

用户明确100%缩放正常，标准字体偏大；直接校准共享字体，不再等待旧版本号。采用正文/导航13px、普通控件12px、副标题15px、弹窗标题18px、页标题22px；保留micro/caption/label10/11/12px与点击尺寸、缩放语义。概览主状态改用同一标题令牌。此为cfbd90c4b+1da4cabca上的用户明确字体增量，不引入当前并行未来模块。软件f1579c6f，含QA依赖检查head3e950b12，PR63。

179文件1122项UI、typecheck/build通过；两引擎各64项/16截图（1180/900、亮暗、100/150/200%）实际计算字号与缩放比例正确，弹窗边界与关闭可达。Mac独立身份custom-protocol实屏100%正常，200%主操作可见，取消回原页；已恢复100%并停止自有PID19090，业务config SHA保持，无配置包输出、真实同步或生成。全路由428/428。证据compact-typography-20261103。

原隔离原型只抽取“原型/”，漏了上级glass/overview/proto-controls共享文件。旧报告保留，但其原型截图不能证明完整视觉一致。本轮从cfbd90c4b完整支撑原型目录重建，叠加1da4cabca与字体增量，并补冻结品牌SVG和Logo iframe依赖。同步矩阵记录实际基线/URL并拒绝关键资源404；最初缺品牌图的两轮虽各648项几何断言通过，HTTP404仍使整体失败，不记通过。

完整依赖下WebKit同步648/648、0页面/资源错误；Chromium一轮522项后原型events/stamp/visible未定义并超时，不能记全量通过。单独12轮加载无JS/请求错误，仍未代替失败整轮，正在继续查加载问题。

PR61完整CI41653396143通过并合并ec822e20807a9285fb9f42393ed28e608d1d21cf；PR62完整CI41654730935通过，按原head合并。旧c691草稿虽签名/四安装/部分Mac原生通过，字体产品源码已变，必须重建最终候选。实际OTA未执行。

独立官方合同CI41662358604仍失败，实际官方回执为success=true/catalogRefresh=skipped/catalog-unavailable；所用upstream-models只是9行摘要，0条base_instructions模板。第一次sync不保证后续管理写入有完整catalog。管理器正确拒绝按成功继续，不放宽门。已在空HOME安装固定Codex0.166.0，只读version/debug models --bundled得到12条含基础指令的真实catalog，继续补夹具前置条件，尚未重跑通过；日常服务与配置不参与。
