/* 两层固定栏的锚点避让与状态保持；最小DOM不替代浏览器sticky行为。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const value=(h,name)=>h.nodes.get('mpContent').style.getPropertyValue(name);
test('列表共用Tab固定栏，测量顶部内距并随高度变化，不改变草稿 / 保存状态',()=>{
 const h=make(),before=JSON.stringify(h.state().draft);h.context.window.getComputedStyle=()=>({paddingTop:'20px',paddingLeft:'22px',paddingRight:'22px'});h.resizeTabs(40);assert.match(h.page(),/id="mpTabsBar" class="mp-toolbar mp-tabs-bar"/);assert.equal(value(h,'--mp-tabs-top'),'-20px');assert.equal(value(h,'--mp-tabs-height'),'40px');h.scroll(600);assert.equal(value(h,'--mp-tabs-height'),'40px');assert.equal(h.state().channelDetail,false);h.click('tab',{tab:'templates'});h.resizeTabs(76);assert.equal(value(h,'--mp-tabs-height'),'76px');assert.equal(JSON.stringify(h.state().draft),before);assert.equal(JSON.stringify(h.state().saved),before);
});
test('模版锚点同时避让Tab和详情标题；窄窗换行 / 导航收起 / 重绘保留编辑',()=>{
 const h=make();h.open('gpt');h.meta('description','保留编辑');const before=JSON.stringify(h.state().buffers);h.context.window.getComputedStyle=()=>({paddingTop:'20px',paddingLeft:'22px',paddingRight:'22px'});
 for(const [tabs,header,viewport] of [[40,72,600],[76,116,500],[40,72,600]]){
  h.resizeTabs(tabs);h.resize(header,viewport,20,1);h.click('editor-section',{id:'common'});const main=h.nodes.get('mpMain'),section=h.nodes.get('mpTemplateSection-common');assert.equal(section.getBoundingClientRect().top-main.getBoundingClientRect().top,tabs+header+13);assert.equal(value(h,'--mp-editor-top'),tabs-20+'px');assert.equal(value(h,'--mp-editor-nav-top'),tabs+header+12-20+'px');assert.equal(value(h,'--mp-editor-nav-height'),viewport-tabs-header-24+'px');assert.equal(JSON.stringify(h.state().buffers),before);
 }
 const main=h.nodes.get('mpMain');h.click('toggle-editor-nav');const offset=h.nodes.get('mpTemplateSection-common').getBoundingClientRect().top-main.getBoundingClientRect().top;h.proto.render();assert.equal(h.nodes.get('mpTemplateSection-common').getBoundingClientRect().top-main.getBoundingClientRect().top,offset);assert.equal(JSON.stringify(h.state().buffers),before);
});
test('渠道详情使用同一两层偏移，侧栏可用高度随Tab换行更新且未确认输入保持',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});h.event('input','[data-channel-config]',{channelConfig:'name'},'未确认名字');const before=JSON.stringify(h.state().channelConnections);h.context.window.getComputedStyle=()=>({paddingTop:'20px',paddingLeft:'22px',paddingRight:'30px'});h.nodes.get('mpChannelHeader').getBoundingClientRect=()=>({top:100,bottom:172,height:72});h.resize(72,500,20,1);
 h.resizeTabs(76);assert.equal(value(h,'--mp-tabs-height'),'76px');assert.equal(value(h,'--mp-editor-top'),'56px');assert.equal(value(h,'--mp-editor-offset'),'161px');assert.equal(value(h,'--mp-editor-nav-height'),'328px');assert.equal(value(h,'--mp-header-inset-right'),'30px');assert.equal(JSON.stringify(h.state().channelConnections),before);
});
