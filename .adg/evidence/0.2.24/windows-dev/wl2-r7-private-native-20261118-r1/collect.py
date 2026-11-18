from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
def sha(b):return hashlib.sha256(b).hexdigest()
runs={name:read(ROOT/name/'result.json') for name in ['product01','product03','binary02','binary04',*[f'regression{i:02}' for i in range(5,9)]]}
assert runs['product01']['raw']==101 and all(r['raw']==0 for n,r in runs.items() if n!='product01')
assert 'test result: ok. 8 passed; 0 failed; 0 ignored;' in (ROOT/'product03/stdout.log').read_text(encoding='utf-8-sig')
for name,count in dict(regression05=2,regression06=1,regression07=1,regression08=1).items():assert f'test result: ok. {count} passed; 0 failed;' in (ROOT/name/'stdout.log').read_text(encoding='utf-8-sig'),name
sources=read(ROOT/'binary04/source.json')['sources'];prior=read(ROOT/'product03/source.json')['sources']
changes=[r['path'] for r in sources if r['sha256']!=next(x['sha256'] for x in prior if x['path']==r['path'])]
assert set(changes)=={'apps/desktop/tauri/src/infrastructure/windows_owner_host024/recovery_control/client.rs','apps/desktop/tauri/src/infrastructure/windows_owner_host024/recovery_control/server.rs'}
for row in sources:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256'] and raw==(ROOT/'binary04/executed-source'/row['path']).read_bytes(),row['path']
launcher=read(ROOT/'launcher-ps51-check.json');assert launcher['raw']==0 and launcher['helperRootAbsent'] and not Path(launcher['helperRoot']).exists()
for name in ['launch-r7-private-native-empty.ps1','launch-r7-private-native-empty.cmd']:
    relative='test/windows024-native/'+name;raw=(CODE/relative).read_bytes()
    target=ROOT/'launcher-source'/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
    sources.append(dict(path=relative,bytes=len(raw),sha256=sha(raw),verification='PS51 actual AST/helpers for ps1; exact command dispatch wrapper for cmd'))
assert next(r['sha256'] for r in sources if r['path'].endswith('.ps1'))==launcher['scriptSha256']
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE)
write('source-manifest.json',dict(codeBefore=git('rev-parse','HEAD').decode().strip(),mainInput=git('rev-parse','origin/main').decode().strip(),docsInput=git('rev-parse','origin/docs/governance-main').decode().strip(),authority='TEAM31/IMP168/r7 unchanged',files=sources,finalBinaryRustSources=8,launcherSources=2,onlyPostUnitRustChanges=changes,postUnitChanges='server nativepeer DATA logging + expected terminalACK disconnect no authority race; action native pre/post auth fences unchanged',unitCodecSourceUnchanged=True))
binary=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/ocxd.exe'
test=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe'
artifacts=[]
for label,path in [('actualOcxdBinary',binary),('actualProductLibraryTestExe',test)]:
    raw=path.read_bytes();artifacts.append(dict(label=label,path=str(path),bytes=len(raw),sha256=sha(raw),releaseCandidate=False))
assert artifacts[0]['sha256']=='55cd6ef08653edc8bf2b5c754006473fba7b3fc8f4a0a04c486ea89222181d3e'
assert artifacts[1]['sha256']==read(ROOT/'regression05/result.json')['artifactSha256']
assert read(ROOT/'binary-argv-check.json')['artifactSha256']==artifacts[0]['sha256']
write('artifacts.json',dict(files=artifacts,nativeOrdinaryPeerDesktopAcceptance=False,actualOriginalProductOwnerRestorationAccepted=False))
staged=read(ROOT/'desktop-staged.json');public=Path(staged['publicRoot'])
for row in staged['files']:
    raw=(public/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
assert not list((public/'out01').iterdir())
write('desktop-preflight-readback.json',dict(publicRoot=str(public),inputFilesVerified=len(staged['files']),outDirectoryEmpty=True,privateExecutionPerformed=False,expectedSession=7,account='marenben',latestSessionObservation='quser shows session7 disconnected, real ordinary interactive desktop required',sourceOfNoJobRequirement='r7/TEAM24B native endpoint peer gate, no controller inheritedJob bypass',currentR2Touched=False))
priorclean=read(ROOT.parent/'wl2-r7-private-wire-20261118-r1/cleanup-readback.json')
write('cleanup-readback.json',dict(helperRoot=launcher['helperRoot'],helperRootAbsent=True,newRuntimeOwnerCreated=False,newNativeEndpointExecuted=False,publicPreparedInput=str(public),publicOutputEmpty=True,failedAndRejectedRoots=[dict(path=r['path'],present=Path(r['path']).exists()) for r in priorclean['failedAndRejectedRoots']],oldDeletionRetried=False,currentR2Touched=False,synthetic27960='previous native readonly identity/plan only; no disposition',historicalCleanupComplete=False))
base=CODE/'apps/desktop/tauri/src/infrastructure/windows_owner_host024'
server=(base/'recovery_control/server.rs').read_text(encoding='utf-8');client=(base/'recovery_control/client.rs').read_text(encoding='utf-8');original=(base/'existing_qualification.rs').read_text(encoding='utf-8')
assert original.index('recovery_control::install_before_capture(&c)?')<original.index('ExistingQualification::prepare()?')
assert original.index('recovery_control.bind_runtime(runtime.clone())?')<original.index('lifecycle::qualify_existing(')
assert server.count('.no_outer_job()?')>=3 and server.count('.verify_image(')>=3
assert 'qualification_failure_cleanup' not in server and 'force_owned_cleanup' not in server+client
assert server.index('let facts = self.dispatch(')<server.index('self.recheck(&peer, request.deadline.end)?;',server.index('let facts = self.dispatch('))
assert '.request_cleanup(' not in client and 'before-resume-private-control-v2.json' in server
write('source-review.json',dict(method='supplemental exactsource order/authentication/version/targetscope review; not independentsecurityaudit or actualdesktopacceptance',checks=dict(endpointInstalledBeforeQualificationClientMutation=True,originalRuntimeArcBoundBeforeNativeOwnerStart=True,nativeOrdinaryNoJobSameImageTokenAndLocatorFences=True,oldV1NoFallbackOrForcedCleanup=True,oneActualOriginalWorkerWindowAndReturnValidation=True,diagnosticNotFullHistory=True,nativeHistoryDistinctOneStatus5=True,terminalAckDeliveryDoesNotRequireConnectedPipeAfterServerDisconnect=True,emptyFixtureScopeNeverBindsRuntimeOrCreatesOwner=True),ordinaryDesktopStillPending=True,actualFixedOfficialOwnerRecoveryStillPending=True))
write('first-error-and-fixes.json',dict(product01=dict(raw=101,elapsedMs=runs['product01']['elapsedMs'],error='outer bootstrap used private CommandV2 in function signature at3sites; no native execution',fix='keep enum private, parse string inside recovery_control parent wrappers, no new native permission'),product03=dict(raw=0,elapsedMs=runs['product03']['elapsedMs'],passed='8strict wire/reply/ack/rawrefusal DATA tests'),binary02=dict(raw=0,elapsedMs=runs['binary02']['elapsedMs']),binary04=dict(raw=0,elapsedMs=runs['binary04']['elapsedMs'],postUnitChange='native peer DATA logging and terminalACK expected disconnect handling'),launcherConstruction='two JS tool cells had syntax errors before any script executed; fresh unexecuted helper placeholder replaced before PS51 parse; no desktop/native process started',actualDesktopFirstError=None,currentR2Touched=False))
schema=ROOT/'generated-schema';schema.mkdir(exist_ok=False)
for name in ['desktop-schema.json','windows-schema.json']:shutil.copyfile(CODE/'apps/desktop/tauri/gen/schemas'/name,schema/name)
(schema/'desktop-schema-original.json').write_bytes(git('show','HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'))
print(json.dumps(dict(sources=len(sources),rust=8,scripts=2,ocxdSha=artifacts[0]['sha256'],testSha=artifacts[1]['sha256'],nativeDesktopPending=True,raws={k:r['raw'] for k,r in runs.items()})))
