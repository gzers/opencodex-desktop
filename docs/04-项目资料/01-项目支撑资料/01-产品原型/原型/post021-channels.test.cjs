const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
async function setup(t){const h=await make('#models');h.w.TextEncoder=TextEncoder;t.after(()=>h.dispose());return h;}
test('new channel needs only a unique display name; local address discovery requires no private-network control',async t=>{
 const h=await setup(t);h.click('[data-mp=new-channel]');
 assert.equal(h.document.querySelector('#mpChannelId'),null);
 h.input('#mpChannelName','中文 渠道 / openai');h.input('#mpChannelURL','http://127.0.0.1:11434/v1');h.input('#mpChannelKey','mock-key');h.click('#modalConfirm');
 assert.match(h.document.querySelector('#modalBody').textContent,/获取模型列表/);
 assert.doesNotMatch(h.document.querySelector('#modalBody').textContent,/允许.*私有网络/);
 const method=h.document.querySelector('#mpChannelListMethod');method.value='openai-list';method.dispatchEvent(new h.w.Event('change',{bubbles:true}));h.click('[data-mp=channel-fetch]');await h.wait(750);
 assert.match(h.document.querySelector('#modalBody').textContent,/已获取/);
 assert.deepEqual(h.network,[]);
});
test('rename accepts non-ASCII and reserved words without a second identifier, while duplicate names stay invalid',async t=>{
 const h=await setup(t);h.click('[data-mp=edit-provider][data-id=newapi]');
 assert.equal(h.document.querySelector('#mpChannelInline-providerId'),null);
 for(const name of ['中文 名称 /','openai']){h.input('#mpChannelInline-name',name);assert.equal(h.document.querySelector('#mpChannelRenameError').textContent,'');assert.ok(h.document.querySelector('#mpChannelRenamePreview').textContent.includes(name));}
 const other=h.w.modelPrototype.state().saved.providers.find(p=>p.id!=='newapi').name;
 h.input('#mpChannelInline-name',other);assert.match(h.document.querySelector('#mpChannelRenameError').textContent,/已存在/);
});
