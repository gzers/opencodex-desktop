from pathlib import Path
import hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parent;DOCS=ROOT.parents[4]
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(value,f,ensure_ascii=False,indent=2)
commit=load(ROOT/'source-commit.json');head=git('rev-parse','HEAD').decode().strip();assert head==commit['head']
remote=git('ls-remote','origin','refs/heads/codex/024-windows-native').decode().split()[0];assert remote==head
write('source-push.json',dict(head=head,remoteHead=remote,verified=True))
for row in commit['files']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
    assert raw.replace(bytes([13,10]),bytes([10]))==git('show',head+':'+row['path']).replace(bytes([13,10]),bytes([10]))
assert all(p.startswith('test/windows024-native/') for p in git('diff','--name-only','625101e7b',head).decode().splitlines())
stage=load(ROOT/'staged-input.json');public=Path(stage['public'])
for row in stage['files']:
    raw=(public/row['name']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
failures=[]
for name in ['r6-archive-proof-r3-01','r6-archive-proof-r3-02']:
    v=load(ROOT/name/'result.json');assert not v['passed'] and not v['processAction'] and not v['sourceDeleted']
    with zipfile.ZipFile(ROOT/name/'proof.zip') as z:assert len(z.namelist())==69
    failures.append(dict(run=name,firstError=v['firstError'],zipMembers=69,nativeSceneUnchanged=True,retained=True))
success=load(ROOT/'r6-archive-proof-r3-03/result.json');assert success['passed'] and success['powershell'].startswith('5.1')
a=success['archive'];assert a['archiveVerified'] and len(a['files'])==70 and a['totalSourceBytes']==1049255 and len(success['negatives'])==4
with zipfile.ZipFile(ROOT/'r6-archive-proof-r3-03/proof.zip') as z:
    assert sorted(z.namelist())==sorted(r['relative'] for r in a['files'])
    for row in a['files']:
        raw=z.read(row['relative']);assert len(raw)==row['bytes'] and sha(raw)==row['sha256'] and raw==Path(row['source']).read_bytes()
assert success['helperSha256']==sha((CODE/'test/windows024-native/archive-r6-content.ps1').read_bytes())
assert load(ROOT/'r6-archive-proof-r3-03/python-verification.json')['passed']
status=load(ROOT.parent/'wl2-r6-official-source-paths-20261117-r1/final-status.json')
status.update(sourceHead=head,sourceTree=commit['tree'],productionSourceHead='3afd4e59dc61c01a9d63053b498a222b91597634',productionArtifactReference='../wl2-r6-official-source-paths-20261117-r1/product01-artifacts.json',archiveDesktopPending=True,archiveDesktopLauncher=str(public/'archive-r6-failed-preresume.cmd'),activeCodeRoot=str(CODE),activeDocsRoot=str(DOCS),actualProductionExecuted=False,startOpen=False,allAccepted=False)
status['firstErrorThisArchiveRound']=dict(ordinaryR2='PS51 Measure-Object cannot resolve bytes property of Hashtable rows; firstArchiveError line59, archiveVerifiedfalse/filesempty, no ZIP created. Original native image/time checks passed.',localProbe01=failures[0],localProbe02=failures[1],cause='Open stable log directoryentry length can be stale despite flushed data. Inventory now obtains read-only filehandle Length; same original512MiB/2GiB/20k bounds and all byte/hash/beforeafter/member assertions retained. Probe02 added failure details only before actual fix.',environment='Both earlier active Git checkout paths lost their tracked files during this turn, leaving ignored targets. Recovered clean managed worktrees from already-pushed625101e7b and3b3f222ed. Older D feature branch left untouched; new uncommitted evidence reconstructed from original Public raw2 logs. Existing marenben failures/objects/processes not moved or deleted.',localCommandPreparation='One multiline Python patch script failed parsing and one patch found missing old-worktree file; no native test or source mutation from those commands. Recovered worktrees then applied exact changes.')
status['matrix'].append(dict(id='ArchivePS51EndToEndR3',result='actualPS51-shared-helper-pass70files/Python-independent ZIP bytespass/ordinaryr3pending',fact='Explicit boundedInt64 sum replaces PS51Hashtable Measure-Object. Source lengths use readonly handle, not stale directorymetadata for open logs. Same helper in ordinary r3 exercised full inventory/compression/allmember hash/exact beforeafter dir+file membership and fresh source hash;70files1049255bytes incl Unicode/nestedempty/zerobyteSQLite/heldopenlog/random1MiB. ExistingZIP/duplicate/relativeescape/missing4cases refuse before overwrite. Two old failed local partialZIPs and r1/r2 logs retained, no assertion weakening or production retry.'))
status['cleanupThisArchiveRound']=dict(newLocalScenes=3,allLocalFixtureSourcesRetained=True,partialAndSuccessfulLocalZipRetained=True,sourceDeleted=False,priorDeletionRejectionNotRetried=True,ordinaryFailedBroker54692AndSuspendedNode50580Retained=True,oldBroker40588Untouched=True,ordinaryPrivateArchiveR3Pending=True,NTFSIdentitySecurityCloneClaimed=False,overallCleanupComplete=False)
status['nextRequiredDecision']='Read actual r3 archive receipt first. Loaded original c870 host cannot consume new native cleanup commands; new process cannot recover originalJob/Owner. Any forced cleanup or new independent fixture plan must keep explicit authorization/unknown/forced distinctions; no publicStart/normalClosed qualification from archive.'
write('final-status.json',status)
report=['# r3只读归档：PS51完整验证与原失败保留','',
 '源码'+head+'；仅4Windows测试脚本，生产Rust及制品仍3afd4e59d，不冒重编。r2真实普通桌面已验证broker54692与挂起Node50580同原FILETIME，但在Hashtable输入的Measure-Object bytes统计处失败，ZIP未创建，原目录与进程保留。原完整617148字节receipt及548字节错误日志收存。',
 '', '归档逻辑抽为同一archive-r6-content.ps1供实际入口与本地定向脚本调用；显式Int64累加沿原每文件512MiB/整树2GiB/20000文件上限。完整测试又揭示open稳定日志的目录项length滞后，改为只读原文件handle Length；前后长度/字节摘要/全目录文件membership和ZIP逐member校验保持，不把大小不一致忽略。',
 '', '实际WindowsPowerShell5.1完整通过70文件/1049255源字节，包含嵌套/空目录、Unicode路径、零字节SQLite、稳定但持写handle日志及随机1MiB文件；独立Python逐member与源bytes/SHA核。原ZIP不可覆盖、重复relative/越界relative/缺失源四反例拒绝。前两本地partial69member失败日志/执行脚本/ZIP留，原marenben进程从未被localproof操作。三个测试scene与ZIP均留，无删除/旧拒绝重试。',
 '', '批处理现保留PS exitcode，明确成功或失败，旧r1/r2/Public内容不改。r3入口'+str(public/'archive-r6-failed-preresume.cmd')+'及全部4文件SHA已核；测试helper hash与staged相同，pwsh/PS51解析通过。普通r3尚待实际桌面，不能当归档已完成。',
 '', '原工作树tracked checkout在执行期间消失，已通过managed工具从remote625101e7b/治理3b3f222ed恢复新独立code/docs检出，旧D feature未改；当前根及此事件明确记录。之前删除拒绝不重试，原失败Public/Profile/旧40588/本轮54692与50580均留。后续完整原包/Closed/Stop/后继/GUI交接/初始化/ownerreceipt/原缺席安全退根/全容量/普通MSI/CAP-trust/历史清理未完，Start/allAccepted=false，不业务/模型/W2/双SID/发布。']
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write(chr(10).join(report)+chr(10))
p=DOCS/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md';heading='## 2026-11-17 r3只读归档PS51完整验证'
assert heading not in p.read_text(encoding='utf-8-sig')
paragraph=heading+chr(10)+chr(10)+'源码'+head+'已推送，仅4test脚本；生产Rust/制品仍3afd4e59d，证据.adg/evidence/0.2.24/windows-dev/'+ROOT.name+'/。r2原普通session7读回broker/Node同FILETIME，但Measure-Object无法解析Hashtable bytes统计，ZIP未建，原617148bytes receipt/548bytes错误保留。共享readonly archive helper显式Int64累加原20k/2GiB/512MiB限；实际PS51完整测试发现openlog目录项length滞后后改readonly handle.Length，全部原字节/hash/前后membership断言保持。测试70文件1049255源字节完整压缩/逐member/源hash过，独立Python核；existingZIP/duplicate/escape/missing四反例拒绝，前两localpartial69member源码/ZIP/错日志保留。三localscene未删、不重试旧删除拒绝。'+chr(10)+chr(10)+'旧工作树tracked检出执行中消失，managed新code/docs从已推送6251/3b3恢复；当前根详final-status，不改旧D feature。r3入口'+str(public/'archive-r6-failed-preresume.cmd')+'待用户原桌面，改批处理准确exit0/1不再统一finished，4input摘要核。普通全归档尚未完成，原54692/挂起50580/旧40588与失败全树留，未kill/权限/配置/PATH/官方/业务/模型/W2/双SID/发布。原宿主无后续控制channel，归档后仍需明确受控清理/新独立scene方案，不从PID/JSON构建旧Owner/NormalStop；Start/allAcceptedfalse，全版其余未完保持。'+chr(10)
with p.open('a',encoding='utf-8') as f:f.write(chr(10)*2+paragraph)
print(json.dumps(dict(head=head,localPS51Pass=True,ZIPMembers=70,sourceBytes=1049255,ordinaryArchivePending=True)))
