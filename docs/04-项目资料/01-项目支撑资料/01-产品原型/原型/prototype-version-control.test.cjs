/* 工具板版本选择与原有范围行为接线；不代表实屏像素对齐。 */
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {make}=require('./component-topic-harness.cjs');
test('原型版本使用版本+协作包名，导航排其后，切换范围保持原行为',async t=>{
 const h=await make('#models'),{w,document:d}=h,$=q=>d.querySelector(q);t.after(()=>h.dispose());w.scrollTo=()=>{}; // jsdom没有视口滚动实现；导航可达性以折叠组状态检查。
 const select=$('#prototypeDesignScope'),control=$('#prototypeVersionControl'),rail=$('.proto-rail');
 assert.equal(select.getAttribute('aria-label'),'原型版本');assert.equal(select.closest('label').firstChild.textContent,'原型版本');
 assert.deepEqual([...select.options].map(o=>[o.value,o.textContent]),[['models','0.2.0 · 模型配置管理与同步'],['conversations','待定版本 · Codex 对话策略与渠道能力'],['extensions','待定版本 · 多 Agent 平台扩展管理']]);
 assert.equal(control.parentElement,rail);assert.equal(control.nextElementSibling,$('.pctl-bar'));assert.equal(rail.nextElementSibling.id,'protoCommon');
 const wrapper=select.closest('.select');assert.ok(wrapper);assert.match(wrapper.querySelector('.select-trigger').getAttribute('aria-label'),/^原型版本：/);
 for(const [scope,name] of [['conversations','Codex 对话策略与渠道能力'],['extensions','多 Agent 平台扩展管理'],['models','模型配置管理与同步']]){
  h.change('#prototypeDesignScope',scope);await h.settle();assert.equal(d.documentElement.dataset.designScope,scope);assert.equal(w.conversationPrototype.scope(),scope);assert.ok(wrapper.querySelector('.select-value').textContent.includes(name));assert.equal(new URL(w.location.href).searchParams.get('design'),null);
 }
 // 移位后仍由公共导航打开折叠组，避免跳到隐藏内容。
 $('#protoCommon').open=false;h.click('.pctl-bar a[href="#ctl-look"]');assert.equal($('#protoCommon').open,true);
 assert.equal(h.network.length,0);assert.deepEqual(h.errors,[]);
});
test('旧渠道能力链接进入合并选项；对话策略只在该选项出现，直接打开会切回该选项',async t=>{
 const h=await make('?design=capabilities#conversations'),{w,document:d}=h,$=q=>d.querySelector(q);t.after(()=>h.dispose());w.scrollTo=()=>{};
 assert.equal(w.conversationPrototype.scope(),'conversations');assert.equal($('#prototypeDesignScope').value,'conversations');assert.equal(d.documentElement.dataset.route,'conversations');
 h.change('#prototypeDesignScope','models');await h.settle();assert.equal(w.conversationPrototype.scope(),'models');assert.equal($('[data-settings-section=dialog-policy]').hidden,true);
 w.setRoute('models');await h.settle();h.click('[data-mp=edit-provider][data-id=newapi]');const entry=$('#mpChannelSection-nodes [data-tp=open-settings]');assert.ok(entry);assert.equal(entry.hidden,true);
 w.openSettingsSection('dialog-policy');await h.settle();assert.equal(w.conversationPrototype.scope(),'conversations');assert.equal($('.settings-panel.active').id,'settings-dialog-policy');assert.equal($('[data-settings-section=dialog-policy]').hidden,false);
 h.change('#prototypeDesignScope','extensions');await h.settle();assert.equal(w.conversationPrototype.scope(),'extensions');assert.equal($('.settings-panel.active').id,'settings-general');assert.equal($('[data-settings-section=dialog-policy]').hidden,true);
 assert.equal(h.network.length,0);assert.deepEqual(h.errors,[]);
});
test('选项名称对应真实协作包标题，原生控件和增强外层共用满宽配方',()=>{
 const packages='../../../03-项目协作资料/01-协作包/';
 for(const [folder,name] of [['2026-10-04-模型配置管理与同步','模型配置管理与同步'],['2026-10-11-Codex对话策略与渠道能力','Codex 对话策略与渠道能力'],['2026-10-14-多Agent平台扩展管理','多 Agent 平台扩展管理']]){
  const text=fs.readFileSync(path.resolve(__dirname,packages,folder,'README.md'),'utf8');assert.equal(text.split('\n')[0],'# '+name);
 }
 const css=fs.readFileSync(path.join(__dirname,'conversation-config.css'),'utf8');
 assert.match(css,/\.prototype-side \.cv-scope-control\{[^}]*width:100%;[^}]*padding:0;/);
 assert.match(css,/\.cv-scope-control :is\(select,\.select\)\{[^}]*box-sizing:border-box;width:100%;max-width:100%;min-width:0/);
});
