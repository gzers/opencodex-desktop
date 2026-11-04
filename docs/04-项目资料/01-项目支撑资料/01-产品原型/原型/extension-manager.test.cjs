const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
test('业务扩展：映射前置、目录保护、共享状态及平台版本',async t=>{
 const h=await make('#extensions');t.after(()=>h.dispose());const d=h.document,w=h.w,s=()=>w.extensionPrototype.state();
 const body=()=>d.querySelector('#modalBody').textContent;
 const wait=()=>h.wait(520);
 await t.test('不再有复选框；未映射平台不能启停或切换但详情可达',()=>{
  assert.equal(d.querySelectorAll('#extensionManagerContent input[type=checkbox]').length,0);
  assert.ok(d.querySelector('#xp3RightRows [data-ic-action=map][data-id=claude]'));
  assert.equal(d.querySelector('#xp3RightRows [data-ic-action=toggle][data-id=claude]'),null);
  h.click('[data-xp3=mode][data-value=client]');const before=s().current.client;
  h.click('#xp3LeftRows [data-ic-row=claude]');assert.equal(s().current.client,before);
  h.click('#xp3LeftRows [data-ic-detail=claude]');assert.ok(d.querySelector('.xp3-platform-detail'));assert.match(body(),/Skills 目录映射/);
 });
 await t.test('已有文件与无权限阻止映射，关闭不会改变状态',async()=>{
  h.click('[data-platform-action=inspect]');await wait();assert.match(body(),/先备份文件并清空/);assert.equal(d.querySelector('[data-platform-action=map]'),null);assert.equal(s().mappings.mac.claude.mapped,false);
  h.change('#xp3DirectoryScene','denied');h.click('[data-platform-action=inspect]');await wait();assert.match(body(),/检查访问权限/);assert.equal(s().mappings.mac.claude.mapped,false);w.closeModal();
 });
 await t.test('目录检查后若又出现文件，提交时再次拦截',async()=>{
  h.change('#xp3DirectoryScene','empty');h.click('#xp3LeftRows [data-ic-detail=claude]');h.click('[data-platform-action=inspect]');await wait();assert.ok(d.querySelector('[data-platform-action=map]'));
  h.click('[data-platform-action=map]');h.change('#xp3DirectoryScene','occupied');await wait();assert.equal(s().mappings.mac.claude.mapped,false);assert.match(body(),/先备份文件并清空/);
 });
 await t.test('备份清空后映射成功，同一平台的 Skills 解锁',async()=>{
  h.change('#xp3DirectoryScene','empty');h.click('[data-platform-action=inspect]');await wait();h.click('[data-platform-action=map]');await wait();assert.equal(s().mappings.mac.claude.mapped,true);assert.equal(s().current.client,'claude');w.closeModal();
  assert.ok(d.querySelector('#xp3RightRows [data-ic-detail=design]'));assert.equal(d.querySelectorAll('#xp3LeftRows .ic-leading-icon').length,0);
 });
 await t.test('源目录启停同步到其他已映射平台，未映射平台仍不可管理',()=>{
  h.click('#xp3RightRows [data-ic-action=toggle][data-id=design]');assert.equal(s().resources.find(r=>r.id==='design').shared,false);
  h.click('#xp3LeftRows [data-ic-row=codex]');assert.equal(d.querySelector('#xp3RightRows [data-ic-action=toggle][data-id=design]').getAttribute('aria-pressed'),'false');
  h.click('[data-xp3=mode][data-value=resource]');assert.equal(d.querySelector('#xp3RightRows [data-ic-action=toggle][data-id=hermes]'),null);
  h.click('#xp3LeftRows [data-ic-detail=design]');assert.ok(d.querySelector('[data-extension-detail-target=hermes]').disabled);w.closeModal();
 });

 await t.test('其他链接不会被覆盖；缺少身份或未安装平台不可映射',async()=>{
  h.click('#xp3RightRows [data-ic-detail=qoder]');h.click('[data-platform-action=inspect]');await wait();assert.match(body(),/链接到其他位置/);assert.equal(s().mappings.mac.qoder.mapped,false);w.closeModal();
  assert.equal(d.querySelector('#xp3RightRows [data-ic-action=toggle][data-id=aside]'),null);
  assert.equal(d.querySelector('#xp3RightRows [data-ic-action=toggle][data-id=gajae]'),null);
 });
 await t.test('平台版本读取、检查、取消、失败重试和更新',async()=>{
  h.click('#xp3RightRows [data-ic-detail=claude]');h.change('#xp3VersionScene','read-error');h.click('[data-platform-action=read-version]');await wait();assert.match(body(),/操作失败/);assert.equal(s().versions.claude.current,'');
  h.change('#xp3VersionScene','normal');h.click('[data-platform-action=read-version]');await wait();assert.equal(s().versions.claude.current,'1.2.0');
  h.change('#xp3VersionScene','check-error');h.click('[data-platform-action=check-version]');await wait();assert.match(body(),/操作失败/);assert.equal(s().versions.claude.current,'1.2.0');
  h.change('#xp3VersionScene','normal');h.click('[data-platform-action=check-version]');await wait();assert.equal(s().versions.claude.latest,'1.3.0');
  h.click('[data-platform-action=update]');h.click('[data-platform-action=cancel-update]');assert.equal(s().versions.claude.current,'1.2.0');assert.equal(s().versions.claude.confirm,false);
  h.change('#xp3VersionScene','update-error');h.click('[data-platform-action=update]');h.click('[data-platform-action=confirm-update]');await wait();assert.equal(s().versions.claude.current,'1.2.0');assert.match(body(),/操作失败/);
  h.change('#xp3VersionScene','normal');h.click('[data-platform-action=update]');h.click('[data-platform-action=confirm-update]');await wait();assert.equal(s().versions.claude.current,'1.3.0');assert.match(body(),/平台更新完成/);
  h.click('[data-platform-action=check-version]');await wait();assert.match(body(),/已是最新版本/);w.closeModal();
 });
 await t.test('macOS 与 Windows 路径及映射分别隔离',async()=>{
  h.click('#xp3RightRows [data-ic-action=path][data-id=claude]');assert.equal(s().lastOpenedPath.manager,'访达');assert.equal(s().lastOpenedPath.path,'~/.claude/skills');
  h.change('#xp3DirectoryOS','windows');assert.equal(d.querySelector('#xp3RightRows [data-ic-row=claude] .ic-path-cell .ic-path').textContent,'%USERPROFILE%\\.claude\\skills');
  assert.equal(s().mappings.windows.claude.mapped,false);h.click('#xp3RightRows [data-ic-action=path][data-id=claude]');assert.equal(s().lastOpenedPath.manager,'文件资源管理器');
  h.click('#xp3RightRows [data-ic-detail=claude]');h.click('[data-platform-action=inspect]');await wait();h.click('[data-platform-action=map]');await wait();assert.equal(s().mappings.windows.claude.mapped,true);w.closeModal();
 });
 await t.test('MCP 保留独立启停，未映射 Skills 不阻止 MCP；公共样例不修改',()=>{
  w.extensionPrototype.setKind('mcp');h.click('#xp3LeftRows [data-ic-row=documentation]');assert.ok(d.querySelector('#xp3RightRows [data-ic-action=toggle][data-id=hermes]'));h.click('#xp3RightRows [data-ic-action=toggle][data-id=hermes]');assert.ok(s().resources.find(r=>r.id==='documentation').selected.includes('hermes'));
  assert.deepEqual(h.errors,[]);assert.equal(h.network.length,0);
 });
});

test('平台检测与手动安装路径加入管理',async t=>{
 const h=await make('#extensions');t.after(()=>h.dispose());const d=h.document,w=h.w,s=()=>w.extensionPrototype.state(),wait=()=>h.wait(750),body=()=>d.querySelector('#modalBody').textContent;
 await t.test('左侧只留标题入口；路径单列；未安装平台完整阻止管理',()=>{
  assert.equal(d.querySelectorAll('#xp3LeftRows .ic-action').length,0);assert.equal(d.querySelectorAll('#xp3LeftRows input[type=checkbox]').length,0);
  assert.ok(d.querySelector('#xp3RightRows .ic-path-col'));assert.equal(d.querySelector('#xp3RightRows [data-ic-row=gajae]').dataset.unavailable,'true');assert.equal(d.querySelector('#xp3RightRows [data-ic-action=toggle][data-id=gajae]'),null);
  h.click('#xp3LeftRows [data-ic-detail=design]');assert.ok(d.querySelector('.ext-detail-body'));w.closeModal();
 });
 await t.test('检测完成、失败保留、取消不落地',async()=>{
  h.click('[data-xp3=detect]');assert.match(body(),/正在检测/);await wait();assert.equal(s().detection.state,'ready');assert.equal(s().installations.mac.gajae.installed,false);assert.equal(d.querySelectorAll('.xp3-detection-list li').length,30);
  const previous=JSON.stringify(s().installations.mac);h.change('#xp3DetectionScene','failed');h.click('[data-detect-action=scan]');await wait();assert.equal(s().detection.state,'error');assert.equal(JSON.stringify(s().installations.mac),previous);
  h.change('#xp3DetectionScene','codex-missing');h.click('[data-detect-action=scan]');w.closeModal();await wait();assert.equal(s().installations.mac.codex.installed,true);
 });
 await t.test('未发现平台可进入详情，错误路径、身份或权限不加入',async()=>{
  h.click('#xp3RightRows [data-ic-action=install][data-id=gajae]');assert.ok(d.querySelector('#xp3InstallPath'));h.input('#xp3InstallPath','relative/path');h.click('[data-platform-action=validate-install]');assert.match(body(),/macOS 绝对路径/);assert.equal(s().installations.mac.gajae.installed,false);
  h.input('#xp3InstallPath','/opt/gjc/bin/gjc');
  for(const fixture of ['missing','denied','mismatch']){h.change('#xp3ManualScene',fixture);h.click('[data-platform-action=validate-install]');await wait();assert.equal(s().installations.mac.gajae.installed,false);assert.equal(d.querySelector('[data-platform-action=save-install]'),null);}
 });
 await t.test('验证本身不加入；编辑路径撤销验证；确认后才加入',async()=>{
  h.change('#xp3ManualScene','normal');h.click('[data-platform-action=validate-install]');await wait();assert.equal(s().installations.mac.gajae.installed,false);assert.ok(d.querySelector('[data-platform-action=save-install]'));
  h.input('#xp3InstallPath','/opt/gjc/current/gjc');assert.equal(d.querySelector('[data-platform-action=save-install]'),null);
  h.click('[data-platform-action=validate-install]');await wait();h.click('[data-platform-action=save-install]');assert.equal(s().installations.mac.gajae.installed,true);assert.equal(s().installations.mac.gajae.source,'manual');assert.equal(s().mappings.mac.gajae.mapped,false);
 });
 await t.test('手动加入后仍需映射，映射后解锁且重扫保留手动路径',async()=>{
  h.click('[data-platform-action=inspect]');await wait();h.click('[data-platform-action=map]');await wait();assert.equal(s().mappings.mac.gajae.mapped,true);w.closeModal();
  w.extensionPrototype.setMode('client');h.click('#xp3LeftRows [data-ic-row=gajae]');assert.equal(s().current.client,'gajae');assert.ok(d.querySelector('#xp3RightRows [data-ic-detail=design]'));
  h.change('#xp3DetectionScene','normal');h.click('[data-xp3=detect]');await wait();assert.equal(s().installations.mac.gajae.source,'manual');assert.equal(s().installations.mac.gajae.path,'/opt/gjc/current/gjc');w.closeModal();
 });
 await t.test('当前平台卸载后右侧无资源可操作，详情仍可重新配置',async()=>{
  h.click('#xp3LeftRows [data-ic-row=codex]');h.change('#xp3DetectionScene','codex-missing');h.click('[data-xp3=detect]');await wait();w.closeModal();assert.equal(s().installations.mac.codex.installed,false);assert.equal(d.querySelectorAll('#xp3RightRows [data-ic-row]').length,0);
  h.click('#xp3LeftRows [data-ic-detail=codex]');assert.ok(d.querySelector('#xp3InstallPath'));w.closeModal();
 });
 await t.test('Windows 校验反斜线路径，手动保存不影响 macOS 状态',async()=>{
  h.change('#xp3DirectoryOS','windows');w.extensionPrototype.setMode('resource');h.click('#xp3RightRows [data-ic-action=install][data-id=zcode]');h.input('#xp3InstallPath','/Applications/ZCode.app');h.click('[data-platform-action=validate-install]');assert.match(body(),/Windows 绝对路径/);
  h.click('[data-platform-action=browse-install]');assert.match(d.querySelector('#xp3InstallPath').value,/^[A-Z]:\\/);h.click('[data-platform-action=validate-install]');await wait();h.click('[data-platform-action=save-install]');assert.equal(s().installations.windows.zcode.installed,true);assert.equal(s().installations.mac.zcode.installed,false);w.closeModal();
  assert.deepEqual(h.errors,[]);assert.equal(h.network.length,0);
 });
});
