from pathlib import Path
from datetime import datetime
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
facts=load(ROOT/'local-facts.json');network=load(ROOT/'observed-tool-results.json')
previous=load(ROOT.parent/'wl2-r6-handle-handoff-20261116-r1/final-status.json')
assert facts['observations']['source-head']['stdout']==previous['sourceHead']
assert facts['artifacts']==previous['artifacts']
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=previous['sourceHead'],sourceTree=previous['sourceTree'],
    lastSuccessfullyPushedSource=previous['sourceHead'],lastSuccessfullyPushedEvidence='7d0d567e19b7b3cbc799dcd85ec3a43ad4fca0f4',
    latestRemoteInputsConfirmed=False,contractLocal='TEAM16/IMP142/r6',remoteTransport=network,
    matrix=previous['matrix'],artifacts=facts['artifacts'],priorRunsReusedUnchanged=previous['runs'],
    newBuildsOrNativeTestsThisTurn=False,sourceModified=False,localFacts=facts,
    firstError=dict(command='git fetch origin main docs/governance-main',reason='schannel TLS handshake failure',gitInternalExitCodeCaptured=False,
        caveat='First combined shell continued and returned0; this is not fetch success. Guarded fetch retry/alternate backend command returned1; connector returned isError transport errors.'),
    localDocumentationCommitPendingPush=True,pushAttempted=False,currentPushStatus='not attempted after failed fetch; not remotely verified',
    environmentRequirement='Working verified HTTPS GitHub Git transport for main/docs-governance fetch and eventual code/evidence push; connector HTTPS also failed, underlying root cause unconfirmed.',
    currentRemaining=load(ROOT/'readonly-gap-audit.json'),
    startOpen=False,productionExecuted=False,typedClosedQualified=False,allAccepted=False,overallGoalComplete=False,
    W2=False,dualSID=False,published=False,cleanup=dict(newNativeScenesCreated=0,newScenesRemoved=0,historicalRejectedDeletionRetried=False,
        previousCleanup=previous['cleanup'],overallCleanupComplete=False),networkPolicyOrDailyConfigChanged=False)
with (ROOT/'final-status.json').open('x',encoding='utf-8') as out:json.dump(status,out,ensure_ascii=False,indent=2)
report='''# Windows r6 远端传输前提失败与本地状态

2026-11-16 Asia/Shanghai；上一目标轮已完成句柄交接组件及代码/证据推送，属于有效进展。本轮继续前先fetch。当前不能确认最新main/治理合同，源实现保持原提交，Start无条件拒绝。

| 独立项 | 本轮结果 |
| --- | --- |
| 原Git/Schannel fetch | 两次TLS握手失败；不是成功fetch |
| 仅本命令OpenSSL后端fetch | TLS unexpected EOF；证书验证未关闭，不写永久配置 |
| GitHub连接器查main | backend-api/ps/mcp传输发送失败，没有分支数据 |
| GitHub连接器查治理main | 同传输错误，没有分支数据 |
| 本地源码 | d20103fee084a3e63f728bf7c6664af65405e456，tree5e54b88a8cca03f9c00adf0630d2025deffa61c2；仅原先untracked生成schema |
| 三制品 | 实际文件大小/SHA与上一已推送证据全部一致；未新编译 |
| 新测试/清理 | 没有新native场景或测试，没有旧删除拒绝重试；历史清理缺口不关闭 |
| 推送 | fetch未成功后未发送新push，本轮证据只提交本地；不能称远端已接收 |

首错为fetch的Schannel握手失败，随后普通重试/保留证书验证的单命令alternate backend及独立GitHub connector均未成功。根因尚未确定，不推断所有互联网服务故障，也不关闭TLS验证/改代理/DNS/机器策略/日常配置。observed-tool-results是实际工具返回的转录及明确标注的connector错误摘要，不冒充未采集的git子进程原stderr文件。首次组合shell后续继续读取而exit0不等于fetch成功；内部git退出码未采集，后两次带throw的shell原返回1，connector为isError。

缓存origin/main fac8573e55441fe514919c63c7f1d65181ff3577及origin/docs/governance-main 5782307174f0d1b011b5415c6c1e8d60414e6b36只能称本地已知输入，不能声明本轮远端最新。上一成功推送源码d20103fee和证据7d0d567e1及其测试结果保持原时点，本轮三制品逐字hash复核但没有新构建/新测试证据。完整继承矩阵和所有缺口归final-status.json。

独立只读审计定位了后继具体缺口：Journal::check_inner仍要求原authorized_end且拒绝expiry，cfg(test)旧history helper也调用原check；NativeHandleBatch.completed_at只证明native closes时点，未涵盖canonical结果/current及最终后验完成。未来应在这些全部成功、原Stop截止前创建私有非serde完成receipt，再由新的Status5只读入口核原token/peer/current ID/tip/source/history/归档与retired对象；不得把新end写回旧mutation deadline。exact inventory角色/数量及actual Source/Owner/typedStop/原process/supervisor历史仍须接合，不从文本Closed/数量/Drop推断资格。

因当前无法消费最新远端合同并完成后续推送，停止新增源码改动；不在相同TLS前提下重复构建、native测试或空跑fetch。网络HTTPS传输恢复后先fetch、核最新TEAM/IMP/合同与本地分叉，再从已有缺口继续实现、定向验证并分别推送源码和本轮/后续证据。目标未完成，当前为首次报告的环境阻塞，未标记完成或擅自暂停目标。不W2/双SID/发布。
'''
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write(report)
prior=ROOT.parent/'wl2-r6-handle-handoff-20261116-r1'
for name in ['verify_evidence.py','verify_git_payload.py']:
    with (ROOT/name).open('xb') as out:out.write((prior/name).read_bytes())
print(json.dumps(dict(sourceUnchanged=status['sourceHead'],remoteFreshnessConfirmed=False,newNativeTests=False,sourceChanged=False,pushAttempted=False,goalComplete=False)))
