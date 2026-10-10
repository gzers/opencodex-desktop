const {test}=require('node:test');
const assert=require('node:assert/strict');
const {paths}=require('./data-paths.js');
test('Windows 按实际安装位置派生，支持不同盘符、中文和空格',()=>{
  for(const installation of ['C:\\Tools\\OpenCodex Desktop','D:\\工具\\桌面管理器','G:\\Apps\\OpenCodex']){
    const p=paths('windows',{installation:installation+'\\'});
    assert.equal(p.root,installation+'\\data');
    assert.equal(p.backups,installation+'\\data\\backups');
    assert.equal(p.entry,installation+'\\data\\runtime\\bin\\ocx.cmd');
    assert.equal(p.prefix,installation+'\\data\\runtime\\opencodex');
  }
});
test('macOS 数据位置不随应用移动，默认 HOME 与入口同根',()=>{
  const p=paths('macos',{installation:'~/Applications/OpenCodex Desktop.app'});
  assert.equal(p.root,'~/Library/Application Support/com.gzers.opencodex.desktop');
  assert.equal(p.home,p.root+'/opencodex-home');
  assert.equal(p.entry,p.root+'/runtime/bin/ocx');
});
test('两平台保留自定义根、外置 HOME 与自定义托管前缀',()=>{
  for(const platform of ['macos','windows']){
    const root=platform==='windows'?'F:\\MyData':'~/MyData';
    const p=paths(platform,{root,home:'external-home',prefix:'custom-prefix'});
    assert.equal(p.root,root);assert.equal(p.home,'external-home');assert.equal(p.prefix,'custom-prefix');
    assert.equal(p.custom,true);assert.notEqual(p.defaultRoot,p.root);
  }
});
test('受保护安装目录不静默改回 AppData，系统凭据不落在数据分区',()=>{
  const p=paths('windows',{installation:'C:\\Program Files\\OpenCodex Desktop'});
  assert.equal(p.root,'C:\\Program Files\\OpenCodex Desktop\\data');
  assert.equal(p.credentials,'Windows 凭据管理器');
  assert.equal(paths('macos').credentials,'macOS 钥匙串');
  assert.throws(()=>paths('linux'));
});
