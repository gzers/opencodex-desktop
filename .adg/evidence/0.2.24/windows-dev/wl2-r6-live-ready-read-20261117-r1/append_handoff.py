from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent;p=ROOT.parents[4]/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
s=json.loads((ROOT/'final-status.json').read_text());source=json.loads((ROOT/'final-source.json').read_text())
marker='## 2026-11-17 r6 原Ready完整双树只读校验';assert marker not in p.read_text(encoding='utf-8-sig')
text=f'''

## 2026-11-17 r6 原Ready完整双树只读校验

源码{s['sourceHead']}/tree{s['sourceTree']}已推送，承接7c0bc016b/d75d9a8f7；TEAM16/IMP142/r6及新fetch mainfac8573e5/治理578230717不变。9允许Windows文件，证据 .adg/evidence/0.2.24/windows-dev/wl2-r6-live-ready-read-20261117-r1/。actual内部gate需原Source/保留Origin元数据/typed ServiceObservation＋canonicalReady/Released/Open且无Stop/pending/content资格，完整BOTH-root读视图同时保留，原件custody/anchors/current非产物workID/bytes/8/children/Grok全部核完、source/Ready前后核再释放临时view；不新claim/lease/journal/snapshot，不DELETE/写许可，失败部分view保留poison。三已声明catalog叶只允许文件形状DATA差异，不授内容policy/restore许可。

首library01 MSVC E0616/raw101无测试/新EXE，测试私有archive字段改为实际父目录membership“不增快照”断言，生产privacy未变。library02 raw101/53过1败，新artifact普通write继承未protectedDACL被原检查拒绝，仅修合成创建；同字节替换反例保留旧work到根外、PrivateRecord新路径，membership/bytes相同仅ID不同，要求InvalidData不以上游权限/超时充数。library03 raw101/54过1败，临时view复用allowwrite observer导致拒写probe遇Ok；保原raw32断言，新增专用无DELETE/readData-control/shareRead普通文件view，目录/协调sharing不变，不改old open_node/claim/恢复函数。两个失败EXE和准确scene/source/raw保留。

末library04 raw0/{s['runs']['library04']['passed']}过0败，完整两树持有时普通config写raw32、原claim仍可取、journal序列不变；config编辑/未知目录/artifact目录/Grok编辑/samebytes不同ID均InvalidData且原custody字节保持；实际旧fixture10秒截止后read<=5秒仍可检查，原journal变更检查Tim edOut，准确序列未续。该正例是native storage read，不是actual ordinaryOwner/private typed gate；公共Status/typed adapter/后继与实际原包仍未接。14处原变更/完整Stop/typed传输/内容policy保持原样。

产品custom-protocol MSVCdebug bins raw0/{s['runs']['product01']['elapsedMs']}ms，{len(source['productInputs'])}产品输入/{len(source['testRust'])}最终Rust核工作/Git（CRLF明确等价），预存schema未手改/暂存，三制品SHA归product01-artifacts。{s['cleanup']['scenes']}scene私有ZIP/hash/membership核验后全留（含失败部分readview/custody/canonical），原canonical/快照/history另归档；ZIP非NTFS克隆，无删除/历史拒绝重试。本轮不改官方/日常配置/PATH/账号/策略，不业务/模型请求；原生产副作用/初始化/ownerreceipt/原缺席根/最大容量/普通MSI/CAP-trust/历史清理未完，Start/allAccepted/typedClosed/productionExecuted=false，不旧raw5/空GUI/W2/双SID/发布。
'''.replace('Tim edOut','TimedOut')
with p.open('a',encoding='utf-8',newline='') as f:f.write(text)
print(json.dumps(dict(appended=True,source=s['sourceHead'])))
