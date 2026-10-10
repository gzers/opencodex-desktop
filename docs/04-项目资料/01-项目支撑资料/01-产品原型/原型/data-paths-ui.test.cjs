/* Node VM 接线检查，不代表浏览器布局或真实目录迁移。 */
const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
const DataPaths=require('./data-paths.js');
function harness(){
  const nodes=new Map(), listeners={}, observers=[];
  function node(id){
    if(!nodes.has(id)){
      const classes=new Set();
      nodes.set(id,{id,innerHTML:'',dataset:{},classList:{contains:x=>classes.has(x),toggle(x,on){if(on)classes.add(x);else classes.delete(x);}},
        setAttribute(){},appendChild(child){nodes.set('#'+child.id,child);},querySelectorAll(){return [];},
        addEventListener(type,fn){this[type]=fn;}});
    }
    return nodes.get(id);
  }
  const win={DataPaths,prototypePanel:{refresh(){}},addEventListener(type,fn){(listeners[type]??=[]).push(fn);},
    dispatchEvent(event){(listeners[event.type]||[]).forEach(fn=>fn(event));}};
  let modal;
  const context={window:win,document:{body:node('body'),querySelector:node,createElement:()=>node('created')},
    MutationObserver:class{constructor(fn){observers.push(fn);}observe(){}},
    CustomEvent:class{constructor(type,options){this.type=type;this.detail=options.detail;}},
    openModal(...args){modal=args;},MANAGED_PREFIX:DataPaths.paths('macos').prefix,
    DEFAULT_MANAGED_PREFIX:DataPaths.paths('macos').prefix,MANAGED_ENTRY:DataPaths.paths('macos').entry,
    installPath:DataPaths.paths('macos').prefix,runtimeSources:{managed:{}},UNINSTALL_BODY_PLANS:{managed:{}},
    renderRuntimeSource(){},render(){}};
  const html=fs.readFileSync(path.join(__dirname,'index.html'),'utf8');
  const start=html.indexOf("window.addEventListener('prototype:data-paths'");
  assert.ok(start>=0);
  const bridge=html.slice(start,html.indexOf('</script>',start));
  vm.runInNewContext(bridge,context);
  vm.runInNewContext(fs.readFileSync(path.join(__dirname,'data-paths-ui.js'),'utf8'),context);
  function click(key,value){node('created').click({target:{closest:()=>({dataset:{[key]:value},disabled:false})}});}
  return {win,context,node,click,modal:()=>modal,bodyChange(on){node('body').classList.toggle('windows-mode',on);observers.forEach(fn=>fn());}};
}
test('切换平台与情景后，目录展示、托管入口和卸载记录使用同一模型',()=>{
  const h=harness();
  assert.equal(h.win.dataPathsPrototype.current().platform,'macos');
  h.click('pathPlatform','windows');h.click('pathScene','custom');
  const p=h.win.dataPathsPrototype.current();
  assert.ok(h.node('#card-install-paths tbody').innerHTML.includes(p.home));
  assert.ok(h.node('#card-install-partitions tbody').innerHTML.includes(p.backups));
  assert.equal(h.context.installPath,p.prefix);
  assert.equal(h.context.runtimeSources.managed.path,p.entry);
  assert.equal(h.context.UNINSTALL_BODY_PLANS.managed.pkg,p.prefix);
  assert.equal(h.context.UNINSTALL_BODY_PLANS.managed.entry,p.entry);
  h.click('pathScene','protected');
  assert.ok(h.node('#directoryRuleNote').innerHTML.includes('role="alert"'));
  assert.ok(h.win.dataPathsPrototype.current().root.includes('Program Files'));
  h.click('pathPlatform','macos');
  assert.ok(!h.node('#directoryRuleNote').innerHTML.includes('role="alert"'));
});
test('外观平台变化可同步路径，其他 body 样式变化不重置手动评审平台和独立安装草稿',()=>{
  const h=harness();h.context.installPath='explicit-existing-prefix';
  h.click('pathPlatform','windows');h.bodyChange(false);
  assert.equal(h.win.dataPathsPrototype.current().platform,'windows');
  assert.equal(h.context.installPath,'explicit-existing-prefix');
  h.bodyChange(true);h.bodyChange(false);
  assert.equal(h.win.dataPathsPrototype.current().platform,'macos');
  assert.equal(h.context.installPath,'explicit-existing-prefix');
});
test('修改预览展示保护范围，确认和取消不改变目录',()=>{
  const h=harness(), before=h.win.dataPathsPrototype.current().root;
  h.win.dataPathsPrototype.preview('root');
  assert.ok(h.modal()[1].includes('外置 HOME 与自定义托管前缀保留'));
  h.modal()[3]();assert.equal(h.win.dataPathsPrototype.current().root,before);
  h.win.dataPathsPrototype.preview('home');
  assert.ok(h.modal()[1].includes('外置目录的已有配置先展示差异'));
  assert.equal(h.modal()[4].cancelLabel,'取消');
});
