# 最终发布前接受输入（未关闭）

final-gates.pre-publish.json / evidence-map.pre-publish.json 已按真实同原包6e及父级 installation-applicable-scope.json整理。10/13技术门就绪，7/8AC和15/17checks按范围通过；仅final-ci、public-stable、artifacts-signatures最终public状态保留pending。AC08、full-regression、release-and-ota等待同三门。没有运行最终prepare或关闭。

pre-publish-accepted-summary.json统一绑定所有已验证gate的testedCommit、testSourceCommit、releaseId、EXE、facts及原件摘要。gate使用完整claims指针；AC/check使用现有validate_map接口支持的精确JSON assertions（source/release/id/status），全部含实际路径/SHA。原件链与原失败仍单列，不能把来源层的历史pending/fail误读为当前accepted结果。

本次补证：Mac实际020→021 OTA同一自动进程44591（旧44138）原生读回，8files精确保持、pending消费、无manual relaunch，旧020公钥验证新原包。Windows默认两格按原测试的安装/首屏/资源/默认路径范围接受，不补不存在的移除/数据断言；custom两格仍严格断言EXE移除和数据保留。Windows NSIS default为any/提权，其他三格ordinary/非提权；WindowsOTA为OS自动新进程观察后用同installedEXE隔离重开读回，区别Mac同进程路径。86provider仍无global restart、真实pending保留、copy ack为mock。

## 父级最后只需三个真实报告

1. CI41817249379的gh JSON，明确completed/success，head20d13334a，必需frontend/backend/backend-windows/release-tools/windows-smoke-evidence五job全部success。main-only build若skipped则明确排除，不计pass。CI20d→6e的产品输入用实际Git比较，不靠标题推定。
2. 父级verify-uploaded-assets.py以stage=public生成的asset-verification.json：同release433749817、source6e、八资产ID/字节/SHA与原draft相同、三签名五平台、draft=false/prerelease=false。
3. 父级verify-anonymous-public.py产生的anonymous-public-verification.json及同目录八下载原件：tag/Latest均0.2.21，同资产身份，无凭据头。填充脚本再次读本地下载原件校验摘要。

采集CI命令（在本目录；此命令只读，running/failure报告不能填成success）：

    gh run view 41817249379 --repo gzers/opencodex-desktop --json databaseId,headSha,status,conclusion,jobs,url > ci-final.actual.json

三个实际文件路径齐备后，只需一次填充（PUBLIC_ASSET_REPORT和ANONYMOUS_REPORT替换为父级实际输出路径）：

    python3 -B fill-pre-publish.py --ci ci-final.actual.json --public-assets PUBLIC_ASSET_REPORT --anonymous-public ANONYMOUS_REPORT

成功只新建final-gates.json、evidence-map.json、final-input-merge-receipt.json，拒绝覆盖。尚在运行、缺必需job、public仍prerelease、资产身份改变、匿名下载不一致均拒绝。此脚本无网络、无发布、无Git写入、无Markdown/runtime修改、无kernel关闭调用。

父级最后人类事实仍按README两阶段回写。之后使用原有formal-closeout.py对final-gates.json/evidence-map.json显式prepare、plan、再由父级明确commit；本次未执行任何一步，不提前改变TASK/DMD/IMP状态。
