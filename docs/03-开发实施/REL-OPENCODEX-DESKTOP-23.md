---
id: REL-OPENCODEX-DESKTOP-23
object_kind: release
state: planned
title: 0.2.20 原型一致性与默认字体修复发布
summary: bda7c92d为私有候选；完整CI、Mac包、独立四安装及双平台实际OTA仍待收口，未公开stable。
---

# 0.2.20 原型一致性与默认字体修复发布

用户已授权从治理到stable连续推进。本记录保留候选发布决定与实际结果。范围DMD-OPENCODEX-UI-020、IMP-OPENCODEX-DESKTOP-28及原0.2.20-bugs的S20-01～20；原型cfbd90c4b+1da4cabca叠加默认字体增量，排除并行未来模块。

产品源码bda7c92da4e1f2e02e6bc2c641e0747748b98dd5、私有草稿432929365、tag v0.2.20；公开stable仍0.2.19。Windows实际资产已按tag公钥验签，原始EXE SHA为f864de162ba28b4f2c386fce4bf06d3b959bfc4475c959b7c557a8c0d5b84158；独立四安装41675807159进行中。Mac首轮因GitHub ARM容量不足未分配runner，保留cancelled/failure；同Release41671938225只重跑Mac，attempt2已打包，Windows原包不重建。

完整CI41671607927待Windows后续原生矩阵。额外官方合同41672846917在test-only诊断源644da8f7完整2项通过，产品代码和断言与bda相同，仅增加失败回执日志；前次失败不删除。字体、Mac100/200%与全路由已有证据，最终原包/安装/OTA仍单独验收。

不改日常配置、凭据、服务或真实WebDAV。本机OTA基线为独立0.2.19副本，已有两次真实UI保存、合成Key与精确前像备份；实际升级未执行。先在既有beta通道验收同一签名候选，再晋升stable。原资产、匿名下载、实际OTA、兼容与回退结论取得回执后回填；当前为保持私有、继续验收，不是发布完成证明。

详细证据见.adg/evidence/0.2.20/font-final-candidate-bda7c92d、integration-20261104和原问题清单。保留0.2.19原包/签名作为回退来源；回退不移动公开tag，不把自动降级当作已有能力。


Windows四独立安装回执已回收：41675807159的MSI/NSIS×默认/自定义全部通过，恰为4格；每格sourceCommit=bda7c92d、原始构建SHA=f864de162ba28b4f2c386fce4bf06d3b959bfc4475c959b7c557a8c0d5b84158，定制目录包内EXE摘要与预期类型补丁一致，默认目录无路径覆盖。原生首屏/卸载及数据保留证据归档于font-final-candidate-bda7c92d/four-installers-41675807159；不把四安装当作实际OTA通过。Mac新包及最终完整CI仍进行，当前公开stable保持0.2.19。
