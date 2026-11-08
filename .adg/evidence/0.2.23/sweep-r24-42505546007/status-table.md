# 第24轮完整诊断（不构成候选验收）

固定源码6b44bf84f；25通过／3失败，accepted:false。原始ZIP已核对GitHub大小及SHA256，272成员清单逐项一致。

|步骤|内容|结果|首行错误|
|---|---|---|---|
|01|original-inputs|pass||
|02|current-binary|pass||
|03|server-msi-prepare|pass||
|04|msi-default|pass||
|05|msi-custom|pass||
|06|nsis-default|pass||
|07|nsis-custom|pass||
|08|path-snapshot-and-copy|pass||
|09|native-install-recovery|fail|missing-version-retains-original: Missing real enabled action: 安装程序|
|10|query-and-update-recovery|pass||
|11|two-real-sids|pass||
|12|private-bundle-supply|pass||
|13|runtime-readiness|fail|user1: no unique successful actual runtime start receipt; environment-ready is not service-ready|
|14|runtime-ports|fail|user1: runtime_start_not_healthy; stage=wait-owned-health|
|15|two-user-paths|pass||
|16|same-user-ipc|pass||
|17|cross-user-ipc|pass||
|18|account-and-process-cleanup|pass||
|19|uninstall-and-retention|pass||
|20|server-msi-restore|pass||
|21|query-failure|pass||
|22|update-failure-recovery|pass||
|23|native-process-cleanup|pass||
|24|windows-bundle-root-regression|pass||
|25|backend-state-observer-build|pass||
|26|attested-status-regression|pass||
|27|actual-runtime-fixture-schema|pass||
|28|runtime-port-save-regression|pass||
