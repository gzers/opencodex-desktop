/* 工具板版本选择与原有范围行为接线；不代表实屏像素对齐。 */
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {make}=require('./component-topic-harness.cjs');
test('原型版本使用版本+协作包名，导航排其后，切换范围保持原行为',async t=>{
 const h=await make('#models'),{w,document:d}=h,$=q=>d.querySelector(q);t.after(()=>h.dispose());w.scrollTo=()=>{}; // jsdom没有视口滚动实现；导航可达性以折叠组状态检查。
 const select=$('#prototypeDesignScope'),control=$('#prototypeVersionControl'),rail=$('.proto-rail');
 assert.equal(select.getAttribute('aria-label'),'原型版本');assert.equal(select.closest('label').firstChild.textContent,'原型版本');
 assert.deepEqual([...select.options].map(o=>[o.value,o.textContent]),[['models','0.2.0 · 模型配置管理与同步'],['conversations','待定版本 · Codex 对话配置管理'],['extensions','待定版本 · 多 Agent 平台扩展管理'],['capabilities','待定版本 · 渠道特殊能力与代理兼容']]);
 assert.equal(control.parentElement,rail);assert.equal(control.nextElementSibling,$('.pctl-bar'));assert.equal(rail.nextElementSibling.id,'protoCommon');
 const wrapper=select.closest('.select');assert.ok(wrapper);assert.match(wrapper.querySelector('.select-trigger').getAttribute('aria-label'),/^原型版本：/);
 for(const [scope,name] of [['conversations','Codex 对话配置管理'],['extensions','多 Agent 平台扩展管理'],['models','模型配置管理与同步'],['capabilities','渠道特殊能力与代理兼容']]){
  h.change('#prototypeDesignScope',scope);await h.settle();assert.equal(d.documentElement.dataset.designScope,scope);assert.equal(w.conversationPrototype.scope(),scope);assert.ok(wrapper.querySelector('.select-value').textContent.includes(name));assert.equal(new URL(w.location.href).searchParams.get('design'),null);
 }
 // 移位后仍由公共导航打开折叠组，避免跳到隐藏内容。
 $('#protoCommon').open=false;h.click('.pctl-bar a[href="#ctl-look"]');assert.equal($('#protoCommon').open,true);
 assert.equal(h.network.length,0);assert.deepEqual(h.errors,[]);
});
test('选项名称对应真实协作包标题，原生控件和增强外层共用满宽配方',()=>{
 const packages='../../../03-项目协作资料/01-协作包/';
 for(const [folder,name] of [['2026-10-04-模型配置管理与同步','模型配置管理与同步'],['2026-10-11-Codex对话配置管理','Codex 对话配置管理'],['2026-10-14-多Agent平台扩展管理','多 Agent 平台扩展管理'],['2026-10-16-渠道特殊能力与代理兼容','渠道特殊能力与代理兼容']]){
  const text=fs.readFileSync(path.resolve(__dirname,packages,folder,'README.md'),'utf8');assert.equal(text.split('\n')[0],'# '+name);
 }
 const css=fs.readFileSync(path.join(__dirname,'conversation-config.css'),'utf8');
 assert.match(css,/\.prototype-side \.cv-scope-control\{[^}]*width:100%;[^}]*padding:0;/);
 assert.match(css,/\.cv-scope-control :is\(select,\.select\)\{[^}]*box-sizing:border-box;width:100%;max-width:100%;min-width:0/);
});
