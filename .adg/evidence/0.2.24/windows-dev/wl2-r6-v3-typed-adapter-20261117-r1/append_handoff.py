from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent;p=ROOT.parents[4]/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
s=json.loads((ROOT/'final-status.json').read_text());source=json.loads((ROOT/'final-source.json').read_text())
marker='## 2026-11-17 r6 首代公共typed生命周期分派接入v3';assert marker not in p.read_text(encoding='utf-8-sig')
text=f'''

## 2026-11-17 r6 首代公共typed生命周期分派接入v3

源码{s['sourceHead']}/tree{s['sourceTree']}已推送，承接14b3198a8/4aea903c3；TEAM16/IMP142/r6和新fetch共同源不变，7允许Windows文件。证据 .adg/evidence/0.2.24/windows-dev/wl2-r6-v3-typed-adapter-20261117-r1/。根WindowsOriginals导出切唯一新v3 adapter，私有nonClone/nonserde阶段tag只定位当前retained slot/UUID/actual Owner.scope/准确阶段，失败poison不fallback/migrate/adopt文本。capture/Ready/重复Ready完整readonly/preStop/完整Stop-route-finalClosed接原actual工厂，preStop保存Owner同一个真实ServiceObservation后才fallible prepare，complete接受tuple必须同原typed对象代次，Closed tag只在actual内部完整finalproof及原截止重读后发出。

Status end-only frozen入口按相同end-5s定位确定最早样本起点，不生成now+5；新动作借原ActionWindow，Owner.stop携原window；原Stop内read须精确等于Store.stop_deadline及Journal.original end，不能缩/延/晚补。共享trait/HMAC和原完整recovery_state_at/StatusReceipt/同次receiver尾段、原来源/claim/normal/supervisor/完整恢复关闭等14源片段核对保持。MSVC库raw0/{s['runs']['library01']['passed']}过0败，DATA locator stage/ID/scope/failed反例只比较DATA不构造fakeOwner/ServiceObservation/Closedtag/nativepermit；原capture/v3/native原语/Restart/Windows同次无Owner拒绝回归通过。初prepare patch格式marker不匹配在写前拒绝原记录留；无本轮非预期产品/测试失败，继承首错保持。

firstgeneration公开typed链已注册但actual ordinary fixedofficial capture/Ready/Closed/Status正例未执行，不能拿编译或purelocator/native组件当生产验收。后继目前明确拒绝并保留slot/失败，必须先接准确v3 predecessor有序集合、native namespace/原Owner.previous与newcapture原前驱引用，不放宽empty namespace断言。公开Start仍拒绝；原包config/auth/Grok/动态目录副作用、初始化/ownerreceipt/原缺席Codex安全退根、最大完整容量、普通MSI/CAP-trust/全部历史清理未完。

产品MSVCdebug custom-protocol bins raw0/{s['runs']['product01']['elapsedMs']}ms，{len(source['productInputs'])}产品输入/{len(source['testRust'])}最终Rust核工作/Git（明确CRLF等价），预存schema未手改/暂存，制品SHA归product01-artifacts。{s['cleanup']['scenes']}scene私有ZIP/hash/membership核验后全留，原canonical/快照/history另收存，ZIP非NTFS克隆，无删除/历史拒绝重试。marenben普通桌面登录仍待，控制器/Administrator不代验收。未改官方/日常配置/PATH/账户/策略，不业务/模型请求，不旧raw5/空GUI/W2/双SID/发布；Start/allAccepted/actualtypedClosed/productionExecuted=false。
'''
with p.open('a',encoding='utf-8',newline='') as f:f.write(text)
print(json.dumps(dict(appended=True,source=s['sourceHead'])))
