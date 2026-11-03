from pathlib import Path
import json,hashlib,datetime
R=Path('/tmp/ocx020-final-review-agent/update-scale-fix')
C=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes')
COMMIT='34a3553f9f014c0d553e16e8c509669a3e64fcd9'
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p,**kw):return {'path':str(p),'sha256':h(p),**kw}
def put(name,obj):(R/name).write_text(json.dumps(obj,ensure_ascii=False,indent=2)+chr(10))
identity=json.loads((R/'receipts/source-identity-after.json').read_text());assert identity['sameUpdateCenterBytesAsAllReviewRuns'] and not identity['otherProductionSourceChangesDuringReview']
reports=[]
for engine in ['chromium','webkit']:
 for suite,relative in [('original-scale','reports/update-scale-contract-'+engine+'-r1'),('hosts','reports/update-hosts-'+engine+'-r1'),('expanded','reports/update-scale-expanded-'+engine+'-r2'),('repo-regression','repo-regression-'+engine)]:
  p=R/relative/'report.json';x=json.loads(p.read_text());assert x['completed'] and not x.get('failure') and all(y['passed'] for y in x['checks'])
  expected={'original-scale':2,'hosts':156,'expanded':372,'repo-regression':7}[suite];assert len(x['checks'])==expected
  row=ref(p,suite=suite,engine=engine,assertions=len(x['checks']),result='pass')
  if suite!='repo-regression':
   boundary=p.parent/'browser-boundary-and-fonts.json';b=json.loads(boundary.read_text());assert not b['pageErrors'] and not b['resourceErrors'] and not b['blockedExternal'];row['boundary']=ref(boundary)
  else:
   gate=x['checks'][-1];assert not gate['actual']['errors'] and not gate['actual']['external']
  if suite=='expanded':
   rows=[s for s in x['samples'] if s['state']=='details-long-top'];assert len(rows)==12
   row['actualPaintedButtonHeights']=sorted(set((s['scale'],s['buttonHeight']) for s in rows));row['actualWidths']=[900,1180];row['actualHeights']=[600,760];row['scales']=[100,150,200]
  reports.append(row)
commands=[]
for engine in ['chromium','webkit']:
 commands.extend(json.loads((R/'receipts'/f'original-runs-{engine}.json').read_text()))
 for attempt,code in [(1,1),(2,0)]:
  n=f'update-scale-expanded-{engine}-r{attempt}';script='update-scale-expanded-r1.mjs' if attempt==1 else 'update-scale-expanded.mjs'
  commands.append({'name':n,'cwd':str(R),'commandAsExecuted':f'AUDIT_ENGINE={engine} AUDIT_OUT={R}/reports/{n} AUDIT_SOURCE=working-UpdateCenter-sha256:{identity["sourceFiles"][0]["workingSha256"]} TMPDIR={R}/browser-temp node {R}/harness/update-scale-expanded.mjs > {R}/reports/{n}.log 2>&1',
   'preservedHarness':ref(R/'harness'/script),'exitCode':code,'note':'r1 failed JS parsing before browser launch; preserved, never a product failure' if attempt==1 else 'Expanded real viewport/proportion/geometry/pointer/scroll checks'})
 commands.append({'name':'repo-regression-'+engine,'cwd':str(C),'command':f'AUDIT_ENGINE={engine} AUDIT_OUT={R}/repo-regression-{engine} AUDIT_SOURCE={COMMIT} TMPDIR={R}/browser-temp node apps/desktop/ui/qa/update-dialog-scale.mjs > {R}/reports/repo-regression-{engine}.log 2>&1','script':ref(C/'apps/desktop/ui/qa/update-dialog-scale.mjs'),'exitCode':0,'executed':True})
put('exact-commands.json',{'commands':commands,'noTestsOtherThanFocusedUpdateBrowserQa':True})
static={'script':ref(C/'apps/desktop/ui/qa/update-dialog-scale.mjs'),'actualRuns':'Both engines actually passed7/7; not static-only acceptance',
 'original200PercentThresholdPreserved':True,'additional50And150RatioAssertions':True,'explicitExternalBlockAndPageErrorGate':True,
 'nonBlockingScriptFinding':{'priority':'P2','path':'apps/desktop/ui/qa/update-dialog-scale.mjs','line':14,
 'problem':'finally unconditionally writes completed:true and result derives from checks.every. An early navigation/selector exception before checks can emit completed:true/result:pass with an empty list, despite process exit1.',
 'action':'Track completed/failure explicitly and require nonempty checks, while retaining the nonzero process exit. Current successful executions are unaffected; this is a harness metadata robustness issue, not an outstanding product-scale failure.'},
 'scopeNote':'Permanent regression checks1180x760 at50/100/150/200%. Its layout/scroll coverage is supplied separately by this review expanded900/1180 and100/150/200% matrix.'}
put('repo-regression-review.json',static)
correction={'findingId':'FR-020-UPDATE-SCALE','previousProduct':'bda7c92da4e1f2e02e6bc2c641e0747748b98dd5','status':'resolved-for-reviewed-source','releaseBlockingForThisFinding':False,
 'fixedCommit':COMMIT,'actualUpdateCenterSha256':identity['sourceFiles'][0]['workingSha256'],'sourceIdentityBefore':ref(R/'receipts/source-identity-before.json'),'sourceIdentityAfter':ref(R/'receipts/source-identity-after.json'),
 'evidence':reports,'actual200Percent':'Both hosts and bothengines:34px at100%,51px at150%,68px at200%; originalratio>=1.8 kept unchanged.',
 'geometry':'900x600/1180x760 actualviewports, allfour edges, header/footer, close/confirm hit-tested and clicked with realbrowsermouse; longnotes independentinternal scroll and outerbody scroll preserveactions.',
 'scope':'Browser-only production consumers with explicitmemoryIPC. No OSinstall/restart/OTA/native acceptance. Does not replace main-thread newcandidateCI/package/releasegates.',
 'mappingUpdates':[{'match':{'file':'apps/desktop/ui/src/routes/OverviewRoute.vue','line':337,'component':'UpdateCenter'},'status':'recordedbrowserstate/geometry/shared-scale pass for34a3553f'},
 {'match':{'file':'apps/desktop/ui/src/routes/SettingsRoute.vue','line':851,'component':'UpdateCenter'},'status':'recordedbrowserstate/geometry/shared-scale pass for34a3553f'}],
 'doNotRewriteOldEvidence':'Originalbda scale failures and all previousreports remain unchanged; append thissource-bound resolution.',
 'scriptFinding':static['nonBlockingScriptFinding']}
put('mapping-correction.json',correction)
put('handoff.json',{'fixedCommit':COMMIT,'sourceSha256':identity['sourceFiles'][0]['workingSha256'],'result':'focused-update-scale-review-pass','blockerResolved':True,'reports':reports,'mappingCorrection':ref(R/'mapping-correction.json'),'exactCommands':ref(R/'exact-commands.json'),'staticAndActualRepoRegressionReview':ref(R/'repo-regression-review.json'),
 'totals':{'originalScaleAssertions':4,'hostAssertions':312,'expandedAssertions':744,'repoScriptAssertions':14,'passedAssertions':1074},
 'limits':'NewcandidatefullCI/packaging/publishing owned bymainthread; no claims aboutthem. Reviewagent only wrote thissubdirectory.',
 'visualReview':['reports/update-scale-expanded-webkit-r2/overview-900-600-200-details-long-bottom.png','reports/update-scale-expanded-chromium-r2/settings-900-600-200-details-long-top.png'],
 'failedAttemptsRetained':['reports/update-scale-expanded-chromium-r1.log','reports/update-scale-expanded-webkit-r1.log'],'createdAt':datetime.datetime.now(datetime.timezone.utc).isoformat()})
(R/'FINAL-REVIEW.md').write_text('''UPDATE-SCALE独立复核通过，原阻塞在提交34a3553f9f014c0d553e16e8c509669a3e64fcd9上已解除。

实际UpdateCenter SHA-256：1b3bb05e8e63a9251baeb3f93fe4aaf19479e865847c683df95d8cfd8d398a7f。开始时工作源码、结束时提交源码、Vite实际返回逐字节一致；其他生产源码本轮无变化。

Chromium/WebKit各通过：原始缩放合同2项，原update-hosts156项，扩展几何/比例/操作/滚动372项，仓库永久qa/update-dialog-scale.mjs实际7项。合计1074个断言。

扩展矩阵覆盖Overview/Settings、900×600/1180×760、100/150/200%。实际按钮高度34/51/68px，原200%>=1.8阈值未改；新增精确比例误差<0.03。四边、正文无横向裁切、头尾不收缩、关闭和确认真实鼠标命中、长说明与受限正文内部滚动、Escape/原页返回均通过。两引擎页面/资源/外网错误为0。900×600/200%长说明截图已目检，按钮持续可达。

新增驱动首轮因解析raw响应的JS正则语法错误，浏览器启动前退出1；日志/原harness保留，第二轮修正解析后通过，产品断言未放宽。旧bda真实失败不覆盖，修复结果以新SHA追加。

永久脚本本身已分别实际执行7/7；静态另有非产品阻塞P2：finally无条件completed:true，早期异常时空checks.every可使JSON误写pass，虽然进程仍exit1。建议后续显式记录failure/completed并要求非空checks；本轮实际通过结果不受影响。精确位置、命令、全部报告哈希见repo-regression-review.json、exact-commands.json及handoff.json。

仅本子目录有新增文件，未改产品/共享docs/原型。只做隔离浏览器内存IPC，不代表新正式包/native/OTA验收；主线程继续负责新候选完整CI、包与发布门。
''')
files=[]
for p in sorted(R.rglob('*')):
 if p.is_file() and 'browser-temp' not in p.relative_to(R).parts and p.name not in ['SHA256SUMS','artifact-manifest.json']:
  files.append({'path':str(p.relative_to(R)),'bytes':p.stat().st_size,'sha256':h(p)})
put('artifact-manifest.json',{'root':str(R),'files':files})
(R/'SHA256SUMS').write_text(''.join(x['sha256']+'  '+x['path']+chr(10) for x in files))
print(json.dumps({'result':'pass','commit':COMMIT,'UpdateCenterSha256':identity['sourceFiles'][0]['workingSha256'],'assertions':1074,'mapping':str(R/'mapping-correction.json'),'files':len(files)}))
