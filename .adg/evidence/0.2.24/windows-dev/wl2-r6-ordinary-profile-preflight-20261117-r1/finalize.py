from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
DOCS=ROOT.parents[4]
BASE=ROOT.parent/'wl2-r6-successor-predecessors-20261117-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
def write(n,v):
    with (ROOT/n).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=subprocess.check_output(['git','-C',str(CODE),'rev-parse','HEAD']).decode().strip()
tree=subprocess.check_output(['git','-C',str(CODE),'rev-parse','HEAD^{tree}']).decode().strip()
assert head==load(ROOT/'source-commit.json')['head']
remote=subprocess.check_output(['git','-C',str(CODE),'ls-remote','origin','refs/heads/codex/024-windows-native']).decode().split()[0];assert remote==head
write('source-push.json',dict(head=head,remoteHead=remote,branch='codex/024-windows-native'))
stage=load(ROOT/'staged-input.json');public=Path(stage['public'])
for row in stage['stagedFiles']:
    raw=Path(row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
assert not (public/'out01/launch-result.json').exists()
for name,args in [('public-acl',['C:/Windows/System32/icacls.exe',str(public)]),('session',['quser'])]:
    r=subprocess.run(args,capture_output=True)
    (ROOT/f'{name}.stdout.log').write_bytes(r.stdout);(ROOT/f'{name}.stderr.log').write_bytes(r.stderr)
    write(f'{name}.json',dict(args=args,rawExit=r.returncode));assert r.returncode==0
assert 'marenben' in (ROOT/'session.stdout.log').read_text(encoding='utf-8',errors='replace')
for row in load(ROOT/'library01/source-manifest.json'):
    raw=(ROOT/'library01/executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
    if row['path'].endswith('.rs'):assert raw.replace(b'\r\n',b'\n')==(CODE/row['path']).read_bytes().replace(b'\r\n',b'\n')
test=stage['files'][0];assert test['name']=='r6-profile-tests.exe'
status=load(BASE/'final-status.json')
status.update(sourceHead=head,sourceTree=tree,productionSourceHead='4452d583d16f33865a27d8a8518e3a330bd57d2a',productionSourceTree=load(BASE/'final-status.json')['sourceTree'],productionArtifactReference='../'+BASE.name+'/product02-artifacts.json',preparationTestArtifact=test,preparationNativeRustProducer='library01',preparationBuild=load(ROOT/'library01/result.json'),preparationDirectedDataPassed=1,preparationOrdinaryDesktopIgnoredPendingUser=True,ordinaryExistingBaselinePrepared=False,ordinaryAccount='marenben',ordinarySession=7,desktopLauncher=str(public/'launch-r6-profile-preflight.cmd'),actualProductionExecuted=False,startOpen=False)
status['matrix'].append(dict(id='OrdinaryDesktopExistingBaselinePreparation',result='prepared-compiled-and-staged/pending-actual-marenben-desktop',fact='New ignored native preflight only original SID1003/session7/KnownFolder and no-outerJob. Collect all independent states before setup; both clients and task base must be Absent, any Present/Unknown refuses. Create-new credential-free synthetic baseline records exact IDs/all8/bytes and prior absence, no official initialization/lease/restore/Closed/Start. Private same-SID bounded log export included; native Rust code unchanged by separately parsed PowerShell export helpers.'))
status['cleanupPreparation']=dict(publicTaskRetained=True,accountAndProfileUnchangedSoFar=True,clientRootsTouched=False,reason='Ordinary desktop invocation pending; no preparation native test executed in controller context.',historicalRejectedDeletionRetried=False,overallCleanupComplete=False)
status['firstErrorThisPreparation']=dict(nativeProductOrTestError=None,toolError='First WindowsPowerShell5.1 parser invocation passed script path after -Command but $args[0] was absent: EmptyScriptExtent/FileReadError. This was checker argument handling, not a script/native test failure. Corrected to encoded literal ParseFile command; both launcher/export scripts parse in WindowsPowerShell5.1 and pwsh.',ordinaryNativeError=None)
write('final-status.json',status)
write('preparation-errors.json',status['firstErrorThisPreparation'])
write('artifact.json',dict(**test,sourceHead=head,sourceRustProducer='library01',ordinaryDesktopExecutionPending=True,actualProductionExecuted=False))
report=f'''# r6普通桌面已有根基线准备入口

源码{head}；生产实现与制品仍归4452d583d/上一轮product02，本轮仅6Windows测试文件，未改生产Start门。MSVCdebug custom-protocol库raw0/1纯DATA反例通过，1真实普通桌面用例ignored待用户显式运行，不冒充普通令牌验收。

入口{public}/launch-r6-profile-preflight.cmd已准备，测试EXE SHA256 {test['sha256']}；脚本/EXE逐份摘要核，Public本任务目录显式赋SID1003 Modify且原ACL保留，不改账户/日常策略。原SID1003/session7交互Active。只在真实KnownFolder Profile/noJob/固定8投影和精确全部根Absent成立时create-new无凭据已有根测试基线；原缺席事实单列，不将其当官方初始化/安全退根证明。任一已有/未知路径拒绝，不覆盖。所有setup前缀/原ID/all8/bytes记录保留，非canonical恢复事实源，不授NativeOwner/lease/Closed许可。

测试Rust精确归library01；终态后只新增单独PowerShell原SID日志导出和launcher调用，不改已编Rust/断言，源差异摘要明列source-commit.json。两脚本在实际WindowsPowerShell5.1及pwsh解析通过，首次checker的-Command参数未进入args[0] FileReadError原工具首错单列，不冒产品失败或重跑native。

当前尚未执行此入口，未创建marenben客户端根/测试Profile基线、未启动官方服务，所有生产原包/真实Closed/后继/Status/H-S-D/首次初始化/原缺席根/全容量及普通MSI/CAP-trust/历史清理未完继承。Start/allAccepted/productionExecuted=false，不旧raw5/空GUI/W2/双SID/发布；无官方/日常配置/PATH/策略修改或业务/模型请求。
'''
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write(report)
p=DOCS/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
heading='## 2026-11-17 r6普通桌面原件基线准备入口'
assert heading not in p.read_text(encoding='utf-8-sig')
with p.open('a',encoding='utf-8') as f:f.write(f'\n\n{heading}\n\n源码{head}已推送，仅6Windows测试文件；生产仍4452d583d/证据1836d28ef。证据.adg/evidence/0.2.24/windows-dev/{ROOT.name}/。MSVCcustom-protocol库raw0/1DATA反例过/1真实普通桌面ignored待用户，本轮不重跑原v3/空GUI。精确marenben SID1003/session7/KnownFolder/noJob和全部客户端/任务根Absent才create-new无凭据existing测试基线，原缺席事实准确保留，不当官方初始化/退根证明。固定8投影/原ID/bytes/setup前缀只作准备DATA，不是恢复journal或许可。原SID可读日志导出独立脚本两PowerShell解析通过；首次checker FileReadError仅args传递错误保留，未native重跑。入口{public}/launch-r6-profile-preflight.cmd及EXE摘要{test["sha256"]}已核；Public本任务ACL局部赋测试SID Modify，未动日常策略/账户。\n\n普通桌面用例尚未执行，真实Profile客户端根尚未修改/服务未启动；当前需用户在原桌面运行，控制器/Administrator不代验收。公开Start与实际Closed/生产/后继仍false；上一轮全状态/首错/234scene中6纯DATA清228保留/609raw等继承，所有普通MSI/CAP-trust/原缺席Codex安全退根/初始化/原包副作用/容量/真实H-S-D/历史清理未完不关闭。无官方/日常配置/PATH/策略/业务/模型/旧raw5/空GUI/W2/双SID/发布。\n')
print(json.dumps(dict(head=head,testSHA256=test['sha256'],ordinaryDesktopPending=True,actualProductionExecuted=False)))
