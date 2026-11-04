const test=require('node:test'),assert=require('node:assert/strict');
const fs=require('node:fs'),vm=require('node:vm'),path=require('node:path');
const code=fs.readFileSync(path.join(__dirname,'overview-hero.js'),'utf8');
const start=code.indexOf('function nearbyPosition('),end=code.indexOf('function place()',start);
const context={};vm.runInNewContext(code.slice(start,end),context);
const position=context.nearbyPosition;
const iconStart=code.indexOf('function inIconBounds('),iconEnd=code.indexOf('function inHoverBridge(',iconStart);
vm.runInNewContext(code.slice(iconStart,iconEnd),context);
const node={l:100,t:150,r:140,b:190,x:120,y:170};
const overlaps=(p,w,h,b)=>p.x+w>b.l&&p.x<b.r&&p.y+h>b.t&&p.y<b.b;

test('空旷时紧贴锚点优先方向，纵向跟随不同节点而非固定顶端',()=>{
 const first=position(node,180,100,800,500,[node],'right');
 assert.equal(first.x,150);assert.equal(first.y,120);
 const lower={...node,t:270,b:310,y:290};
 const next=position(lower,180,100,800,500,[lower],'right');
 assert.equal(next.x,150);assert.equal(next.y-first.y,120);
});
test('显式保护区域与窗口边界限制下换方向，仍保持完整可见',()=>{
 const target={l:730,t:150,r:770,b:190,x:750,y:170};
 const logo={l:470,t:110,r:610,b:280};
 const other={l:730,t:210,r:770,b:250};
 const p=position(target,220,130,800,500,[target,logo,other],'right');
 assert.ok(p);assert.ok(p.x>=10&&p.x+220<=790&&p.y>=10&&p.y+130<=490);
 for(const obstacle of [target,logo,other])assert.equal(overlaps(p,220,130,obstacle),false);
});
test('其他节点不参与避让，允许覆盖它们并保持固定尺寸紧邻当前节点',()=>{
 const other={l:155,t:100,r:350,b:260};
 const p=position(node,360,200,800,500,[node],'right');
 assert.equal(p.x,150);assert.equal(p.y,70);
 assert.equal(overlaps(p,360,200,node),false);
 assert.equal(overlaps(p,360,200,other),true);
});
test('小数坐标向外取整，不把紧邻的有效位置误判成碰撞',()=>{
 const target={l:100.2,t:150.2,r:140.2,b:190.2,x:120.2,y:170.2};
 const p=position(target,180,100,800,500,[target],'right');
 assert.equal(p.x,151);assert.ok(Math.abs(p.y-120.2)<1);
 assert.equal(position(target,900,600,800,500,[target],'right'),null);
});

test('圆形图标按实际轮廓命中，图标矩形角落和外侧文字区域不命中',()=>{
 const icon={left:100,top:150,width:32,height:32};
 assert.equal(context.inIconBounds(icon,116,166),true);
 assert.equal(context.inIconBounds(icon,132,166),true);
 assert.equal(context.inIconBounds(icon,100,150),false);
 assert.equal(context.inIconBounds(icon,99,166),false);
 assert.equal(context.inIconBounds(icon,160,166),false);
 const scaled={left:100,top:150,width:25.6,height:25.6};
 assert.equal(context.inIconBounds(scaled,112.8,162.8),true);
 assert.equal(context.inIconBounds(scaled,126,162.8),false);
});

test('移动通道只连接图标与弹窗相对边，不包含卡片侧面的大块空白',()=>{
 const icon={left:100,right:132,top:150,bottom:182};
 const right={left:142,right:502,top:80,bottom:360};
 assert.equal(context.inIconBridge(icon,right,137,166),true);
 assert.equal(context.inIconBridge(icon,right,137,120),false);
 assert.equal(context.inIconBridge(icon,right,137,210),false);
 assert.equal(context.inIconBridge(icon,right,135,184),false);
 const below={left:50,right:410,top:192,bottom:450};
 assert.equal(context.inIconBridge(icon,below,116,187),true);
 assert.equal(context.inIconBridge(icon,below,70,187),false);
 assert.equal(context.inIconBridge(icon,below,200,187),false);
});
