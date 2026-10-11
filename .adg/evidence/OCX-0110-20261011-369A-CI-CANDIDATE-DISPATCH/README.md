# 369ab12d 完整 CI 与单次 Windows 候选触发

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182。完整源码 SHA 369ab12da55448fbfdc4d0c9bf03970b56fd6c27。

[CI 38124355301](https://github.com/gzers/opencodex-desktop/actions/runs/38124355301) completed / success：release-tools、frontend、backend、backend-windows、macOS build 五个 job 全部成功。ci-final.json 为可读 JSON，ci-final.stdout.txt / stderr.txt 为命令输出。

远端 main 与源码一致，查询最近 50 个候选无同 SHA run 后，于 2026-10-11 09:04:20—09:04:23 UTC 仅触发一次 windows-candidate.yml。新 [候选 38127059287](https://github.com/gzers/opencodex-desktop/actions/runs/38127059287) createdAt 09:04:22Z、headSha 一致；dispatch-receipt.json 保留参数、时间、查重与触发次数。candidates-before / after 和 remote-main 原件保留。

candidate-status-2.json 为后续快照：Windows regression with the package lock 成功；Build NSIS and MSI candidates 正在执行，整体 in_progress。尚未完成安装 smoke、上传或本地独立核验，不能写候选通过。早期状态不覆盖；无需重复 dispatch。macOS 同源码候选局部验证另见 ../OCX-0110-20261011-369A-MACOS-TREE-CANDIDATE/。

该工作流只生成候选 artifact，不创建 Release / tag，权限 contents:read。releaseAuthorized / releaseAccepted 均 false；设备双权限 / 物理 DPI / 默认根 / 原生、性能 / 真实周期、真实更新 / 事件、正式签名仍待验。checksums.sha256 覆盖本目录证据。
