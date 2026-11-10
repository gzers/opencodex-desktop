const {test}=require('node:test');
const assert=require('node:assert/strict');
const {createStore,fingerprint,routes,endpoint,validatePaths,capabilityRequests,servicePaths}=require('./channel-capabilities.js');
const fixture=()=>({id:'krill',name:'Krill',activeNodeId:'primary',connection:{auth:'apikey',keyConfigured:true,credentialRef:'opaque-1',specialAccess:{site:'krill',mode:'pat'}},nodes:[{id:'primary',url:'https://api.kbsfdxc.net/codex/v1'},{id:'backup',url:'https://backup.example/v1'}],models:[{id:'gpt-6.1-sol'}]});
test('认证与能力分开：认证成功不能让所有功能变为可用，Ultrafast支持按型号',()=>{
 const p=fixture(),s=createStore();const a=s.begin(p,'primary','connection','gpt-6.1-sol');s.finish(p,a);
 assert.equal(s.auth(p).id,'passed');assert.equal(s.row(p,'gpt-6.1-sol','notes').condition,'待验证');
 assert.equal(s.row(p,'gpt-6.1-sol','ultra').support,'supported');assert.equal(s.row(p,'gpt-6-astra','ultra').support,'supported');assert.equal(s.row(p,'gpt-6.1-terra','ultra').support,'unsupported');
 const f=s.begin(p,'primary','features','gpt-6.1-sol',['ws','notes']);s.finish(p,f);
 assert.equal(s.row(p,'gpt-6.1-sol','ws').condition,'本机入口待开启');assert.equal(s.row(p,'gpt-6.1-sol','notes').condition,'当前服务不可用');
});
test('同节点配置变动使结果失效，名称变动不失效，停止或迟到不能污染新版本',()=>{
 const p=fixture(),s=createStore();s.seed(p);const fp=fingerprint(p);p.name='新名称';assert.equal(fingerprint(p),fp);assert.equal(s.auth(p).id,'passed');
 const r=s.begin(p,'primary','features','gpt-6.1-sol',['notes']);p.connection.credentialRef='opaque-2';assert.equal(s.finish(p,r),false);assert.equal(s.auth(p).id,'stale');
 const r2=s.begin(p,'primary','connection','gpt-6.1-sol');s.stop(p,'primary');assert.equal(s.finish(p,r2),false);assert.notEqual(s.auth(p).id,'passed');
});
test('备用节点不会借用主节点认证；换型号不会沿用另一型号的功能实测',()=>{
 const p=fixture(),s=createStore();s.seed(p);assert.equal(s.auth(p,'backup').id,'unknown');
 assert.equal(s.row(p,'gpt-6-astra','compact').condition,'待验证');
 assert.equal(s.row(p,'gpt-6-astra','notes').condition,'当前服务不可用'); // 笔记属于节点与身份，不随型号重复认证。
 const r=s.begin(p,'backup','connection','gpt-6.1-sol');s.finish(p,r);assert.equal(s.auth(p,'backup').id,'passed');assert.equal(p.activeNodeId,'primary');
});
test('网络失败保留支持依据；当前未认证、原生身份与模拟服务恢复分别表达',()=>{
 const p=fixture(),s=createStore();s.seed(p);s.setScene('network');const r=s.begin(p,'primary','features','gpt-6.1-sol',['notes']);s.finish(p,r);
 assert.equal(s.row(p,'gpt-6.1-sol','ultra').support,'supported');assert.match(s.status(p).record.lastError,/网络超时/);
 s.setScene('ready');p.connection.specialAccess.mode='key';let a=s.begin(p,'primary','connection','gpt-6.1-sol');s.finish(p,a);let f=s.begin(p,'primary','features','gpt-6.1-sol',['notes']);s.finish(p,f);assert.equal(s.row(p,'gpt-6.1-sol','notes').condition,'需原生身份接入');
 p.connection.specialAccess.mode='pat';a=s.begin(p,'primary','connection','gpt-6.1-sol');s.finish(p,a);f=s.begin(p,'primary','features','gpt-6.1-sol',['notes']);s.finish(p,f);assert.equal(s.row(p,'gpt-6.1-sol','notes').condition,'可使用');
});
test('绑定冻结节点、认证和配置：未保存不能模拟接通，保存也不产生实际采用',()=>{
 const p=fixture(),s=createStore();s.seed(p);assert.equal(s.bind(p),true);assert.equal(s.binding([p],[p]).active,false);
 assert.equal(s.reconnect([p],[p]),true);assert.match(s.binding([p],[p]).label,/采用待观察/);
 const saved=structuredClone(p);p.connection.specialAccess.mode='key';const a=s.begin(p,'primary','connection','gpt-6.1-sol');s.finish(p,a);s.bind(p);assert.match(s.binding([p],[saved]).label,/尚未保存/);assert.equal(s.reconnect([p],[saved]),false);
 p.activeNodeId='backup';assert.notEqual(s.auth(p).id,'passed');
});
test('复制配置不复制认证结果，停用渠道不能作为实例接入',()=>{
 const p=fixture(),s=createStore();s.seed(p);const copy=structuredClone(p);copy.id='krill-copy';assert.equal(s.auth(copy).id,'unknown');
 p.enabled=false;assert.equal(s.bind(p),false);p.enabled=true;assert.equal(s.bind(p),true);p.enabled=false;assert.match(s.binding([p],[p]).label,/已停用/);assert.equal(s.reconnect([p],[p]),false);
});
test('停止复验保留同一配置的旧通过依据，并且旧代际不能完成新一轮',()=>{
 const p=fixture(),s=createStore();s.seed(p);const first=s.begin(p,'primary','connection','gpt-6.1-sol');s.stop(p,'primary');assert.equal(s.auth(p).id,'passed');assert.equal(s.finish(p,first),false);
 const second=s.begin(p,'primary','connection','gpt-6.1-sol');assert.equal(s.finish(p,first),false);assert.equal(s.finish(p,second),true);
});
test('节点路径以同域地址预览，路径修改使旧认证失效，节点名称不失效',()=>{
 const p=fixture(),s=createStore(),n=p.nodes[0];s.seed(p);
 assert.equal(endpoint(p,n,'models'),'https://api.kbsfdxc.net/codex/v1/models');
 assert.equal(endpoint(p,n,'native'),'https://api.kbsfdxc.net/backend-api/codex');
 n.name='更名线路';assert.equal(s.auth(p).id,'passed');
 n.paths={modelBase:'/proxy/v1',models:'/directory/models',native:'/native/codex'};
 assert.equal(s.auth(p).id,'stale');assert.equal(endpoint(p,n,'models'),'https://api.kbsfdxc.net/directory/models');
 assert.equal(routes(p,n).modelBase,'/proxy/v1');
 for(const invalid of ['https://else.example/v1','//else.example/v1','/v1?key=x','/v1/../hidden','/v 1'])assert.notEqual(validatePaths({models:invalid}),'');
 assert.equal(validatePaths(n.paths),'');
});
test('各型号的实测独立保留；无模型时仍可认证并检查节点能力',()=>{
 const p=fixture(),s=createStore();s.seed(p);
 let r=s.begin(p,'primary','features','gpt-6-astra',['call','compact']);s.finish(p,r);
 assert.equal(s.row(p,'gpt-6.1-sol','compact').condition,'当前为代理摘要');
 assert.equal(s.row(p,'gpt-6-astra','compact').condition,'当前为代理摘要');
 assert.equal(s.row(p,'gpt-6.1-terra','compact').condition,'待验证');
 p.models=[];r=s.begin(p,'primary','connection','');s.finish(p,r);assert.equal(s.auth(p).id,'passed');
 r=s.begin(p,'primary','features','',['ws','notes','call']);s.finish(p,r);
 assert.equal(s.row(p,'','notes').condition,'当前服务不可用');assert.equal(s.row(p,'','call').condition,'先获取模型列表');
});
test('能力地址按模型协议派生：快速档共用模型接口，WS复用响应接口，后台无虚构接口',()=>{
 const p=fixture(),n=p.nodes[0],call=capabilityRequests(p,n,'call','Responses');
 assert.deepEqual(call.urls,['https://api.kbsfdxc.net/codex/v1/responses']);
 for(const id of ['fast','ultra'])assert.deepEqual(capabilityRequests(p,n,id,'Responses').urls,call.urls);
 assert.deepEqual(capabilityRequests(p,n,'ws','Responses').urls,['wss://api.kbsfdxc.net/codex/v1/responses']);
 assert.deepEqual(capabilityRequests(p,n,'background','Responses').urls,[]);
 assert.deepEqual(capabilityRequests(p,n,'compact','Responses').urls,['https://api.kbsfdxc.net/codex/v1/responses/compact']);
 assert.deepEqual(capabilityRequests(p,n,'call',null).urls,[]);
 assert.deepEqual(capabilityRequests(p,{url:'https://local.example/api'},'call','Ollama').urls,['https://local.example/api/chat']);
});
test('原生服务前缀统一派生历史与笔记，压缩路径可覆盖但不改变支持判定',()=>{
 const p=fixture(),n=p.nodes[0],s=createStore();s.seed(p);
 n.paths={native:'/relay/codex',compact:'/custom/compact'};
 const urls=capabilityRequests(p,n,'notes','Responses').urls;
 assert.equal(urls.length,3);assert.ok(urls.every(u=>u.startsWith('https://api.kbsfdxc.net/relay/codex/alpha/')));
 assert.deepEqual(capabilityRequests(p,n,'compact','Responses').urls,['https://api.kbsfdxc.net/custom/compact']);
 assert.equal(s.row(p,'gpt-6.1-terra','ultra').support,'unsupported');assert.equal(s.auth(p).id,'stale');
 assert.equal(s.auth(p,'backup').id,'unknown');
});
test('特殊服务覆盖和恢复只修改同一节点路径，保留模型基址及目录配置',()=>{
 const p=fixture(),n=p.nodes[0];n.paths={modelBase:'/proxy/v1',models:'/directory/models',native:'/old/codex'};
 const next=servicePaths(n,'custom',{native:'/new/codex/',compact:'/new/compact/'});
 assert.deepEqual(next,{modelBase:'/proxy/v1',models:'/directory/models',native:'/new/codex',compact:'/new/compact'});
 assert.equal(n.paths.native,'/old/codex');
 assert.deepEqual(servicePaths({paths:next},'default',{}),{modelBase:'/proxy/v1',models:'/directory/models'});
 assert.equal(servicePaths({paths:{native:'/old/codex'}},'default',{}),null);
 assert.equal(validatePaths({native:'',compact:''}),'');
 assert.notEqual(validatePaths({compact:'https://other.example/compact'}),'');
});
