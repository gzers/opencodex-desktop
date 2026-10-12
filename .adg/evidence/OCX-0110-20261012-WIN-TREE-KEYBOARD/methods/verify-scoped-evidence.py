import pathlib,json,hashlib
root=pathlib.Path.cwd() / '.adg/evidence'
source='afff490a430a81f8c233d44928aa3681a23e91e0'
tree=root/'OCX-0110-20261012-WIN-TREE-KEYBOARD'
def load(p): return json.loads(p.read_text(encoding='utf-8-sig'))
expected={'stable-after-month-click.json':'收起 10','stable-after-right.json':'展开 manual-preferences','stable-after-enter.json':'收起 manual-preferences','stable-after-child-right.json':'展开 bk_20261012044452_a921f503','stable-after-space.json':'收起 bk_20261012044452_a921f503','stable-after-leaf-right.json':'backup-manifest.json','stable-after-down.json':'preferences.json','stable-after-parent-left.json':'收起 bk_20261012044452_a921f503','stable-after-home.json':'收起 backups'}
for filename, label in expected.items():
    doc=load(tree/filename)
    assert doc['sourceCommit']==source and doc['applicationPid']==29936
    assert doc['result']['activeElement']['label']==label and doc['result']['activeElement']['inTree']
refresh=load(tree/'native-refresh-result.json')['result']
assert refresh['sawBusy'] and refresh['clicked']
assert [v['busy'] for v in refresh['transitions']]==[True,False]
assert refresh['priorExpanded']==refresh['afterExpanded'] and len(refresh['priorExpanded'])==5
assert refresh['focusAfter'].strip()=='刷新'
update=root/'OCX-0110-20261012-WIN-UPDATE-DETAILS'
manager=(update/'native-manager-details.ax.txt').read_text(encoding='utf-8')
panel=(update/'native-panel-details.ax.txt').read_text(encoding='utf-8')
assert '按钮 (disabled) 安装管理器更新' in manager
assert '按钮 (disabled) 备份并更新面板' in panel
assert '发行方尚未提供更新说明' in manager and '当前版本查询未提供发布说明' in panel
assert '当前 未知' in panel and '可用 2.83.0' in panel
status=load(update/'native-status-after-cache-details.json')
assert status['sourceCommit']==source and status['result']['runtimeSource']['kind']=='unresolved'
for directory,scope in [(tree,'actual native keyboard/focus/refresh subset'),(update,'actual native cached-details viewing and disabled install states')]:
    entries=[]
    for p in sorted(directory.rglob('*')):
        if p.is_file() and p.name not in ['manifest.json','archive-verification.json']:
            data=p.read_bytes();entries.append({'path':p.relative_to(directory).as_posix(),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
    (directory/'manifest.json').write_bytes((json.dumps({'schemaVersion':1,'hashMode':'raw','entries':entries},ensure_ascii=False,indent=2)+'\n').encode())
    for entry in entries:
        assert hashlib.sha256((directory/entry['path']).read_bytes()).hexdigest()==entry['sha256']
    (directory/'archive-verification.json').write_bytes((json.dumps({'result':'passed','scope':scope,'sourceCommit':source,'applicationPid':29936,'filesVerified':len(entries),'fullNativeGateAccepted':False,'updateInstallAccepted':False,'performanceAccepted':False,'realCyclesAccepted':False,'formalTaskReceipt':False,'releaseAccepted':False},indent=2)+'\n').encode())
    print(json.dumps({'directory':directory.name,'files':len(entries),'scope':scope}))