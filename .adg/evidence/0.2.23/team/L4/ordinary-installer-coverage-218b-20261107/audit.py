import datetime,hashlib,json,os,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parent
code=pathlib.Path(r"C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code")
source="218b2262bdbdcddaaa79c8fc240872ff2b596ae8"
base="cbd02dc9d8f411033c8757ef727e609dba947df5"
ref="refs/heads/docs/0.2.23-windows"
prefix=".adg/evidence/0.2.23/team/L4/ordinary-installer-coverage-218b-20261107/"
def git(*args,data=None,env=None):return subprocess.check_output(["git",*args],cwd=code,input=data,env=env)
assert not git("status","--porcelain")
assert git("ls-remote","origin",ref).decode().split()[0]==base
paths=[".github/workflows/windows-candidate.yml","test/smoke/windows023-installer-retention.ps1","test/smoke/windows-installer.ps1"]
needles=[['kind: nsis, scenario:','kind: msi, scenario:','windows023-installer-retention.ps1 -AdmissionPath','windows-installer.ps1 -BundleDirectory'],['ordinaryAccountAcceptanceClaimed=$false','function Install-RetentionPackage','function Invoke-RetentionInstaller','Start-Process','GetCurrent().User','SeSecurityPrivilege'],['function Invoke-Installer','Start-Process -FilePath','windows-ordinary-startup.ps1']]
facts=[]
for path,terms in zip(paths,needles):
    b=git("show",source+":"+path);text=b.decode()
    facts.append({"path":path,"sha256":hashlib.sha256(b).hexdigest(),"selectedLines":[{"line":i,"text":line} for i,line in enumerate(text.splitlines(),1) if any(term in line for term in terms)]})
retention=git("show",source+":"+paths[1]).decode()
assert 'ordinaryAccountAcceptanceClaimed=$false' in retention
assert 'Start-Process' in retention and 'SeSecurityPrivilege' in retention
assert 'scenario: fresh023' in git("show",source+":"+paths[0]).decode()
result={"observedUtc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"sourceCommit":source,"instructionCommit":"af1d18dd6a5804c67f3f7c976461ddaab5d397b3","scope":"read-only installer acceptance coverage audit","nativeExecuted":False,"sourceFrozen":True,"ordinaryInstallerAcceptanceProven":False,"retentionHarnessExplicitlyDeclinesOrdinaryAccountClaim":True,"ordinaryStartupDoesNotProveInstallerToken":True,"sourceFacts":facts,"currentCandidate":{"runId":42333262869,"jobId":128284347930,"lastObservedStatus":"in_progress","lastObservedStep":"Windows regression with the package lock"}}
(root/"verification.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n",encoding="utf-8",newline="\n")
report="""# L4：普通用户安装器最终验收覆盖缺口

2026-11-07。用户继续要求跟进Windows普通用户默认安装/升级/保留最终回执；源码冻结，L4只读核对，不改实现或派发重复候选。明确观察源码218b2262bdbdcddaaa79c8fc240872ff2b596ae8，当前候选42333262869/job128284347930仍为Windows regression with the package lock运行中。

关键结论：当前windows023-installer-retention.ps1的结果结构明确ordinaryAccountAcceptanceClaimed=false。五场景即使全部pass，也不能单独签作普通用户安装器最终验收。脚本经当前runner的Start-Process执行安装器，含提权runner合成owner处理和SeSecurityPrivilege审计；没有在该入口把安装/升级/卸载全链交真实普通Token执行。此为源码覆盖审计，未伪称取得原生失败或已运行安装器。

同源windows-installer.ps1确有windows-ordinary-startup.ps1，覆盖安装后的EXE普通身份启动；安装器本身仍由Invoke-Installer/Start-Process在调用者身份执行。普通用户运行应用与普通用户执行安装器是两项证据，不能以普通EXE启动或前置普通Rust回归替代安装进程Token事实。

主对话后继需要的最小最终回执：精确候选run/attempt/SHA和NSIS/MSI资产ID/bytes/SHA256；安装/升级/卸载/重装每个入口的实际TokenUser、elevation及受控Profile/LocalAppData；无目录覆盖参数的默认I=LocalAppData/Programs/OpenCodeX Desktop；实际注册上下文、退出码与安装日志；U/D/H和合成业务文件前后哈希、旧根不读取/不迁移；普通身份失败/UAC要求如实记录；临时进程、账户/Profile及ACL清理回执。宿主可提权准备受控环境，但不能让提权安装结果冒普通安装通过。

矩阵保留既有五场景：NSIS fresh023/adjacent；MSI fresh023/adjacent/explicit-conversion。MSI旧per-machine到新per-user的预期阻断应记pass-expected-block，不能记自动升级通过；若需显式卸载旧MSI的管理员步骤，作为单独有权步骤记录，随后新per-user安装及默认卸载/保留需保留普通用户Token事实，不把跨上下文权限限制改成静默迁移。原文合同和断言不由L4修改。

当前没有可验收的新安装器，也没有普通用户安装器最终pass回执。L0继续现候选及后继验证；本回交不触发源码解冻、重新构建、签名或发布。用户目前无需提供密码；若后继确需人工受控通道，再回交确切入口与待执行步骤。
"""
(root/"L4-普通用户安装器覆盖缺口.md").write_text(report,encoding="utf-8",newline="\n")
files=[root/"verification.json",root/"L4-普通用户安装器覆盖缺口.md",root/"audit.py"]
manifest={"schemaVersion":1,"sourceCommit":source,"nativeExecuted":False,"files":[{"path":p.name,"role":"audit-script" if p.suffix==".py" else "audit-report" if p.suffix==".md" else "source-audit","sha256":hashlib.sha256(p.read_bytes()).hexdigest(),"size_bytes":p.stat().st_size,"media_type":"text/x-python" if p.suffix==".py" else "text/markdown" if p.suffix==".md" else "application/json"} for p in files]}
(root/"manifest.json").write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+"\n",encoding="utf-8",newline="\n")
files.append(root/"manifest.json")
index=root/"docs-coverage.index";assert not index.exists()
env=os.environ.copy();env["GIT_INDEX_FILE"]=str(index);env["GIT_NO_LAZY_FETCH"]="1"
git("read-tree",base,env=env)
for p in files:
    blob=git("hash-object","-w","--stdin",data=p.read_bytes()).decode().strip()
    git("update-index","--index-info",data=("100644 "+blob+"\t"+prefix+p.name+"\n").encode(),env=env)
tree=git("write-tree","--missing-ok",env=env).decode().strip()
git("diff-tree","--check",base,tree)
changed=git("diff-tree","--no-commit-id","--name-only","-r",base,tree).decode().splitlines()
assert len(changed)==4
commit=git("commit-tree",tree,"-p",base,data="docs(windows): 明确普通用户安装器最终验收覆盖缺口\n\n保留五场景原断言，区分安装器Token与普通EXE启动证据；源码冻结不变。\n".encode()).decode().strip()
git("update-ref",ref,commit,base)
for p in files:assert git("show",commit+":"+prefix+p.name)==p.read_bytes()
assert not git("status","--porcelain")
(root/"commit-verification.json").write_text(json.dumps({"commit":commit,"base":base,"files":4,"bytesVerified":True,"codeCheckoutClean":True},indent=2)+"\n",encoding="utf-8",newline="\n")
print(json.dumps({"commit":commit,"files":4,"ordinaryInstallerAcceptanceProven":False,"bytesVerified":True,"codeCheckoutClean":True}))
