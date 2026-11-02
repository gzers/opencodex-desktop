# 0.2.19 正式发布证据（2026-11-02）

正式版本v0.2.19；source cb4c9c9b7f342fe582ea6b6c95e3cf957611403c；Release 430987382；2026-11-02北京时间10:19:48公开stable/Latest。发布决定与边界唯一记录在[REL-22](../../../docs/03-开发实施/REL-OPENCODEX-DESKTOP-22.md)。

- main-ci-final.json / main-ci-final-jobs.json：41372541021六job通过。
- verification.json、三份minisign日志、macOS签名/DMG日志：八原资产digest、三包验签、公钥保持、macOS封装通过。二进制与安装包留GitHub Release和本机/tmp，不重复入库。
- release-build-result.json / release-first-failure.log / release-first-installer：首次Windows串行安装失败保留。MSI记忆目录覆盖自定义路径，不把该轮改记为成功。
- installers-run.json / installers-jobs.json / installers-summary.json / installers：41373902280四独立runner MSI/NSIS×default/custom全通过，严格匹配原始EXE SHA256和资产digest，无重建。
- publish-run.json / publish.log / release-final.json / anonymous-public.json：41377101811公开stable，八资产匿名200、大小/hash匹配、两份清单一致，公开前后原资产不变。
- ota-run.json / ota-jobs.json / ota-summary.json / ota：41377311243两种安装器实际0.2.18→0.2.19更新、验签、退出与重启、OS自动重开、沙箱配置及合成凭据保持、pendingRestart清除；第二轮触发重试41377322313亦成功，不重复计入覆盖。
- formal-macos-native.log / macos-formal-app.log / macos-formal-*.png / macos-formal-ax.txt：正式原包显式沙箱首屏和设置页。无日常配置写入、无真实WebDAV/跨设备配置同步。
- publish-tools.log / etw-tools.log / retention-tools.log：29+13+6项离线检查通过。

verify-release.py与verify-public.py保留本次只读核验方法，前者需Python3.11+；路径指向本次工作树，复用应明确改路径。安装器和OTA仅在一次性Windows runner执行；不在本机安装/卸载日常软件。macOS沙箱进程已停止；/tmp/ocx0219-*留作可选复核，不属发布制品。

SHA256SUMS校验归档字节；日志去行末空白与ANSI控制符，目录关闭Git行尾转换。历史失败输出保留，不据成功轮次覆盖失败。报告中的忽略/跳过和未验环境不折算为通过。
