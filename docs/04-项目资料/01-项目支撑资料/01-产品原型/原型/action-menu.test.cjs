const {test}=require('node:test'),assert=require('node:assert/strict'),vm=require('node:vm'),fs=require('node:fs'),path=require('node:path');
function make(){
 const listeners={},nodes=new Map();let sequence=0;
 class Node{
  constructor(id){this.id=id;this.isConnected=true;this.style={};this.attrs={};this.children=[];this.dataset={};this.bounds={left:80,top:60,right:190,bottom:94,width:110,height:34};}
  setAttribute(k,v){this.attrs[k]=v;}
  removeAttribute(k){delete this.attrs[k];}
  set innerHTML(html){this.children=[...html.matchAll(/<button[^>]*data-action-menu-index="(\d+)"[^>]*>([^<]*)<\/button>/g)].map(m=>{const n=new Node(this.id+'-'+m[1]);n.dataset.actionMenuIndex=m[1];n.disabled=/disabled/.test(m[0]);n.text=m[2];return n;});this.html=html;}
  querySelectorAll(){return this.children.filter(n=>!n.disabled);}
  querySelector(){return this.querySelectorAll()[0];}
  appendChild(n){nodes.set(n.id,n);}
  contains(n){return n===this||this.children.includes(n);}
  closest(){return this.dataset.actionMenuIndex!==undefined?this:null;}
  remove(){nodes.delete(this.id);this.isConnected=false;}
  matches(){return this.opened===true;}
  showPopover(){this.opened=true;}
  hidePopover(){this.opened=false;}
  focus(){document.activeElement=this;}
  getBoundingClientRect(){return this.id.startsWith('prototype-action-menu')?{...this.bounds,height:160}:this.bounds;}
 }
 const body=new Node('body'),windowNode=new Node('window');windowNode.bounds={left:40,top:30,right:500,bottom:360,width:460,height:330};
 const document={body,activeElement:null,createElement:()=>new Node('new-'+(++sequence)),querySelector:()=>windowNode,addEventListener(type,fn){(listeners[type]??=[]).push(fn);}},window={addEventListener(type,fn){(listeners[type]??=[]).push(fn);}};
 vm.runInNewContext(fs.readFileSync(path.join(__dirname,'action-menu.js'),'utf8'),{window,document,innerWidth:700,innerHeight:600});
 const menu=()=>[...nodes.values()].find(n=>n.id.startsWith('prototype-action-menu'));
 const key=value=>{const e={key:value,preventDefault(){},stopImmediatePropagation(){}};(listeners.keydown||[]).forEach(fn=>fn(e));};
 const click=target=>(listeners.click||[]).forEach(fn=>fn({target}));
 return {api:window.prototypeActionMenu,anchor:new Node('anchor'),outside:new Node('outside'),menu,key,click,document};
}
test('按钮下方弹出无遮罩菜单，低处向上展开并约束软件窗口；菜单项转义',()=>{
 const h=make();h.anchor.bounds.top=330;h.anchor.bounds.bottom=354;h.api.open({anchor:h.anchor,items:[{label:'<复制>'}]});const m=h.menu();assert.equal(m.attrs.role,'menu');assert.equal(h.anchor.attrs['aria-expanded'],'true');assert.ok(m.html.includes('&lt;复制&gt;'));assert.ok(parseFloat(m.style.top)<h.anchor.bounds.top);assert.ok(parseFloat(m.style.left)>=40);
});
test('菜单跳过禁用项，方向键循环、首尾定位、Esc关闭并归还焦点',()=>{
 const h=make();h.api.open({anchor:h.anchor,items:[{label:'禁用',disabled:true},{label:'比较'},{label:'恢复'}]});assert.equal(h.document.activeElement.text,'比较');h.key('ArrowDown');assert.equal(h.document.activeElement.text,'恢复');h.key('ArrowDown');assert.equal(h.document.activeElement.text,'比较');h.key('End');assert.equal(h.document.activeElement.text,'恢复');h.key('Home');assert.equal(h.document.activeElement.text,'比较');h.key('Escape');assert.equal(h.api.state(),null);assert.equal(h.document.activeElement,h.anchor);
});
test('执行一项只调用一次，外部点击关闭不执行；同触发再次点击关闭',()=>{
 const h=make();let count=0;const options={anchor:h.anchor,items:[{label:'复制',onSelect:()=>count++}]};h.api.open(options);h.click(h.menu().children[0]);assert.equal(count,1);assert.equal(h.api.state(),null);h.api.open(options);h.click(h.outside);assert.equal(count,1);h.api.open(options);h.api.open(options);assert.equal(h.api.state(),null);
});
test('危险操作与常规动作分隔，禁用的危险项不可执行',()=>{
 const h=make();let count=0;h.api.open({anchor:h.anchor,items:[{label:'高级配置'},{label:'丢弃全部修改',danger:true,separatorBefore:true,disabled:true,onSelect:()=>count++}]});const menu=h.menu();assert.match(menu.html,/role="separator"/);assert.match(menu.html,/class="danger"/);h.click(menu.children[1]);assert.equal(count,0);assert.ok(h.api.state());
});
