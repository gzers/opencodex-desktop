from pathlib import Path
import sys,json,yaml
sys.path.insert(0,'/Users/ezio/.codex/skills/project-governance/scripts/lib')
from runtime import kernel as k
root=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main')
bugs='docs/04-项目资料/01-项目支撑资料/06-问题记录/0.2.20-bugs.md'
dmd='docs/01-需求管理/需求/DMD-OPENCODEX-UI-020.md'
imp='docs/03-开发实施/IMP-OPENCODEX-DESKTOP-28.md'
rel='docs/03-开发实施/REL-OPENCODEX-DESKTOP-23.md'
task_id='TASK-OPENCODEX-UI020-CLOSEOUT-20261104'
store=k.RuntimeStore(root)
assert not store.object_path('TASK',task_id).exists()
criteria=[]
for line in (root/bugs).read_text().splitlines():
 if line.startswith('| S20-'):
  cells=line.split('|');idx=cells[1].strip().split(' ')[0].split('-')[1];text=cells[-2].strip()
  if idx=='19': text+=' 用户追加默认100%字体收紧：正文/导航13px、控件12px、副标题15px、弹窗标题18px、页标题22px，软件与原型同源，小字10/11/12px与缩放语义保持。'
  criteria.append({'id':'AC-020-'+idx,'text':text})
assert len(criteria)==20

def read(p):
 _,front,body=(root/p).read_text().split('---',2)
 return yaml.safe_load(front),body

def write(p,m,b):
 (root/p).write_text('---'+chr(10)+chr(10).join(key+': '+json.dumps(value,ensure_ascii=False) for key,value in m.items())+chr(10)+'---'+b)
md,b=read(dmd)
b=b.split('机读验收绑定补正（2026-11-04）')[0].rstrip()
md.update(state='in_progress',next_action='完成最终候选完整CI、原包签名/安装/双平台实际OTA及S20逐项回执后公开stable并关闭。',acceptance_criteria=criteria,resolution=None)
b=b.replace('范围与逐项验收唯一来源：','范围与逐项比较、偏差和证据来源：',1)
b+='\n\n机读验收绑定补正（2026-11-04）：frontmatter的AC-020-01～20按原S20验收列冻结，AC-020-19纳入用户已确认字体增量。具体通过/失败与原始证据仍归原问题记录；收口TASK冻结完整验收范围，不把此前进度摘要当正式关闭回执。\n'
write(dmd,md,b)
mi,b=read(imp)
mi.update(object_kind='implementation.change',state='in_progress',summary='实现与原型复核已有分层证据；最终包、双平台实际OTA与稳定发布收口仍进行中。',demand_ids=['DMD-OPENCODEX-UI-020'],task_ids=[task_id],completion_summary=None)
write(imp,mi,b)
# Reuse this script's owned planned release authority after a parser rejection.
(root/rel).write_text('''---
id: REL-OPENCODEX-DESKTOP-23
object_kind: release
state: planned
title: 0.2.20 原型一致性与默认字体修复发布
summary: bda7c92d为私有候选；完整CI、Mac包、独立四安装及双平台实际OTA仍待收口，未公开stable。
---

# 0.2.20 原型一致性与默认字体修复发布

用户已授权从治理到stable连续推进。本记录保留候选发布决定与实际结果。范围DMD-OPENCODEX-UI-020、IMP-OPENCODEX-DESKTOP-28及原0.2.20-bugs的S20-01～20；原型cfbd90c4b+1da4cabca叠加默认字体增量，排除并行未来模块。

产品源码bda7c92da4e1f2e02e6bc2c641e0747748b98dd5、私有草稿432929365、tag v0.2.20；公开stable仍0.2.19。Windows实际资产已按tag公钥验签，原始EXE SHA为f864de162ba28b4f2c386fce4bf06d3b959bfc4475c959b7c557a8c0d5b84158；独立四安装41675807159进行中。Mac首轮因GitHub ARM容量不足未分配runner，保留cancelled/failure；同Release41671938225只重跑Mac，attempt2已打包，Windows原包不重建。

完整CI41671607927待Windows后续原生矩阵。额外官方合同41672846917在test-only诊断源644da8f7完整2项通过，产品代码和断言与bda相同，仅增加失败回执日志；前次失败不删除。字体、Mac100/200%与全路由已有证据，最终原包/安装/OTA仍单独验收。

不改日常配置、凭据、服务或真实WebDAV。本机OTA基线为独立0.2.19副本，已有两次真实UI保存、合成Key与精确前像备份；实际升级未执行。先在既有beta通道验收同一签名候选，再晋升stable。原资产、匿名下载、实际OTA、兼容与回退结论取得回执后回填；当前为保持私有、继续验收，不是发布完成证明。

详细证据见.adg/evidence/0.2.20/font-final-candidate-bda7c92d、integration-20261104和原问题清单。保留0.2.19原包/签名作为回退来源；回退不移动公开tag，不把自动降级当作已有能力。
''')
def adopt_or_sync(value):
 if store.object_path(value['kind'],value['id']).exists():
  current=store.load(value['kind'],value['id'])
  if current['source_digest']==value['source_digest']:return current
  updated=k.sync_source_bound_object(current,root,expected_digest=current['content_digest'])
  store.put(updated,expected_digest=current['content_digest'])
  return updated
 store.adopt_source_projection(value)
 return value
d=k.new_demand(demand_id=md['id'],title=md['title'],summary=md['summary'],acceptance_criteria=criteria,next_action=md['next_action'],implementation_ids=md['implementation_ids'],source_path=dmd,source_digest=k.digest_bytes((root/dmd).read_bytes()),source_revision=1)
d['state']=md['state'];d=k.seal_object(d);d=adopt_or_sync(d)
i=k.new_implementation(implementation_id=mi['id'],title=mi['title'],summary=mi['summary'],demand_ids=mi['demand_ids'],task_ids=mi['task_ids'],source_path=imp,source_digest=k.digest_bytes((root/imp).read_bytes()),source_revision=1)
i['state']=mi['state'];i=k.seal_object(i);i=adopt_or_sync(i)
checks=['review:prototype-parity','review:saved-file-and-backup','review:credential-and-scope-boundaries','review:all-entry-coverage','review:macos-windows-native','review:release-and-ota']
targets=[{'id':label,'path':p,'before_digest':k.digest_bytes((root/p).read_bytes())} for label,p in [('demand',dmd),('implementation',imp),('issues',bugs),('release',rel)]]
t=k.new_task(task_id=task_id,title='0.2.20全范围验收与发布收口',summary='承接全部S20及字体规则；冻结六项审阅门与20项验收，真实原包/OTA/stable未完成前保持开放。',implementation_id=mi['id'],demand_ids=[md['id']],owner='current-chat-integration',baseline_digest=k.digest_bytes((root/'.adg/evidence/0.2.20/font-final-candidate-bda7c92d/source-freeze.json').read_bytes()),write_scope=['.adg/evidence/0.2.20/**',dmd,imp,bugs,rel],required_checks=checks,acceptance_bindings=k.build_acceptance_bindings([d],[x['id'] for x in criteria]),fact_writeback={'mode':'required','targets':targets})
store.put(t)
started=k.transition_object(t,'in_progress',expected_digest=t['content_digest']);store.put(started,expected_digest=t['content_digest'])
manifest={'task_id':task_id,'task_digest':started['content_digest'],'source':'RuntimeStore verified Markdown and CAS; no completion certificate created','required_checks':checks,'acceptance_count':len(criteria),'fact_writeback':targets}
(root/'.adg/evidence/0.2.20/font-final-candidate-bda7c92d/formal-closeout-binding.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'task':task_id,'state':started['state'],'digest':started['content_digest'],'acceptance':len(criteria),'closed':False}))
