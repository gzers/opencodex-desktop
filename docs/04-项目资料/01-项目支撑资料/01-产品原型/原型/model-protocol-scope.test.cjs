const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const channel=h=>h.state().draft.providers.find(p=>p.id==='newapi');
test('渠道和向导无默认协议；取消/重开无隐藏协议或清单适配',()=>{
 const h=make();assert(h.state().draft.providers.every(p=>!Object.hasOwn(p,'protocol')));const before=JSON.stringify(h.state().saved);
 h.click('new-channel');assert.doesNotMatch(h.dialog(),/mpChannelProtocol|默认接口协议/);h.context.closeModal();h.click('new-channel');assert.doesNotMatch(h.dialog(),/mpChannelProtocol/);assert.equal(JSON.stringify(h.state().saved),before);
 h.context.closeModal();h.click('edit-provider',{id:'newapi'});assert.doesNotMatch(h.page(),/mpChannelInline-protocol|data-channel-config="protocol"/);
 h.click('provider-config');assert.doesNotMatch(h.dialog(),/data-field-mode="protocol"/);
});
test('渠道旧协议与渠道层覆盖不能改变模型协议，单模型覆盖优先',()=>{
 const h=make(),policy=h.state().draft,p=policy.providers.find(p=>p.id==='newapi');
 p.protocol='Ollama';p.defaults.protocol='Ollama';p.familyOverrides.gpt={protocol:'Ollama'};
 assert.equal(h.proto.resolveRoute('newapi','newapi::0',policy).protocol,'Responses');
 assert.match(h.proto.resolveRoute('newapi','newapi::0',policy).source,/模版/);
 assert.equal(h.proto.commitModelPatches('newapi',[{key:'newapi::0',rules:{protocol:'Chat'}}]),'');
 assert.equal(h.proto.resolveRoute('newapi','newapi::0').protocol,'Chat');assert.equal(h.proto.resolveRoute('newapi','newapi::0').source,'单模型自定义');
});
test('清单获取方式显式选择但不持久化，未知模型保持待配置',()=>{
 const h=make();h.click('new-channel');for(const [id,v] of [['mpChannelName','无协议样例'],['mpChannelURL','https://list.example.test/v1'],['mpChannelKey','demo-only']])h.nodes.get(id).value=v;
 h.runConfirm();h.click('channel-fetch');assert.match(h.dialog(),/请选择本次清单获取方式/);
 h.nodes.get('mpChannelListMethod').value='openai-list';h.nodes.get('mpChannelListMethod').onchange();h.click('channel-fetch');h.drain();assert.match(h.dialog(),/gpt-5.4/);
 h.nodes.get('mpChannelManual').value='unknown-custom-model';h.click('channel-manual');h.event('change','[data-channel-model]',{channelModel:'unknown-custom-model'},'',true);h.runConfirm();
 const p=h.state().draft.providers.find(p=>p.name==='无协议样例');assert(p);assert(!Object.hasOwn(p,'protocol'));assert(!Object.hasOwn(p,'listMethod'));assert.equal(h.state().saved.providers.some(p=>p.name==='无协议样例'),false);
 const key=p.models.find(m=>m.id==='unknown-custom-model').key,r=h.proto.resolveRoute(p.id,key);assert.equal(r.protocol,null);assert.match(r.error,/协议待配置/);
 assert.equal(h.proto.commitModelPatches(p.id,[{key,rules:{protocol:'Chat'}}]),'');assert.equal(h.proto.resolveRoute(p.id,key).error,'');
});
test('复制保留模型层协议且不制造渠道协议',()=>{
 const h=make();h.click('copy-provider',{id:'newapi'});h.runConfirm();const copy=h.state().draft.providers.find(p=>p.id!=='newapi'&&p.name.includes('副本'));
 assert(copy);assert(!Object.hasOwn(copy,'protocol'));assert.deepEqual(copy.models.map(m=>m.overrides),channel(h).models.map(m=>m.overrides));
});
test('已有渠道获取清单也要选择临时适配，结果不改变模型协议',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});const baseline=JSON.stringify(channel(h).models);
 h.click('refresh');assert.equal(channel(h).refresh,'idle');h.nodes.get('mpDetailListMethod').value='google-list';h.click('refresh');h.drain();
 assert.equal(channel(h).refresh,'idle');assert.equal(JSON.stringify(channel(h).models),baseline);assert(!Object.hasOwn(channel(h),'listMethod'));assert.equal(h.proto.resolveRoute('newapi','newapi::0').protocol,'Responses');
});
