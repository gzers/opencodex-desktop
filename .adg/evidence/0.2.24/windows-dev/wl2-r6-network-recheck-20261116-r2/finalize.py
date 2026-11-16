from pathlib import Path
import subprocess,json,hashlib
ROOT=Path(__file__).resolve().parent
DOCS=ROOT.parents[4]
result=json.loads((ROOT/'fetch-result.json').read_text(encoding='utf-8-sig'));assert result['gitRawExitCode']==0 and result['latestRemoteInputsConfirmed']
inputs=[]
for path,marker in [('docs/03-开发实施/TEAM-0.2.24.md','TEAM16'),('docs/03-开发实施/IMP-OPENCODEX-DESKTOP-34.md','revision: 142'),('docs/03-开发实施/0.2.24-contracts/windows-claim-anchored-custody-r6.md','revision: 6')]:
    raw=subprocess.check_output(['git','-C',str(DOCS),'show',f'origin/docs/governance-main:{path}']);assert marker in raw.decode('utf-8')
    inputs.append(dict(ref=result['governanceMain'],path=path,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
with (ROOT/'verified-inputs.json').open('x',encoding='utf-8') as out:json.dump(dict(fetch=result,actualRemoteContractInputs=inputs,sourceBeforeFollowup=result['sourceHead'],publicStartOpen=False),out,ensure_ascii=False,indent=2)
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write('''# HTTPS传输复核恢复

2026-11-16 18:36 Asia/Shanghai。本目标第二次涉及同一传输前提的自动续轮，仅原配置下一次git fetch，原生退出码0/1247ms；main fac8573e55441fe514919c63c7f1d65181ff3577、docs/governance-main 5782307174f0d1b011b5415c6c1e8d60414e6b36，实际读取远端TEAM16/IMP142/r6原blob并核摘要。本次没有不安全TLS/永久配置/网络策略更改，也不推断先前故障根因。

原fetch stdout/stderr/真实git退出码由fetch-once.ps1直接捕获；上轮efd8bfcd3的失败证据按原时点保留，不追溯改成功。复核前源码仍d20103fee且仅原先untracked schema；未为网络复核重跑native/产品构建/清理。传输阻塞解除，先补推上轮本地证据及本次复核，再继续跨动作历史/生产资格实现；目标未完成，Start保持拒绝，不W2/双SID/发布。
''')
previous=ROOT.parent/'wl2-r6-network-preflight-20261116-r1'
for name in ['verify_evidence.py','verify_git_payload.py']:
    with (ROOT/name).open('xb') as out:out.write((previous/name).read_bytes())
print(json.dumps(dict(fetchRawExit=0,remoteInputsVerified=len(inputs),transportBlockResolved=True,sourceBeforeFollowup=result['sourceHead'])))
