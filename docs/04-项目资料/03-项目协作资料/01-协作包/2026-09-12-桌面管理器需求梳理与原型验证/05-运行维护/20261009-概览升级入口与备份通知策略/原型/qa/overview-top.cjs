const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const base='docs/04-项目资料/01-项目支撑资料/01-产品原型/';
const source=fs.readFileSync(base+'overview-dual.js','utf8');
new vm.Script(source);
const body=source.slice(source.indexOf('  function updateBackdrop()'),source.indexOf('  details.hidden = true;'));
const morph=fs.readFileSync('docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/候选/2026-09-28-Logo本体形变/morph.js','utf8');
new vm.Script(morph);
const receiver=morph.split('\n').find(line=>line.includes("setProperty('--embedded-padding'"));
const cases=[[75,660,103],[35,520,52],[66,640,85],[27,480,40]];
let count=0;
for(const [overhang,height,padding] of cases)for(const distance of [75,110,240])for(const scale of [1,.68])for(const scroll of [0,180]){
 const properties={};
 const main={offsetHeight:700,clientTop:1,scrollTop:scroll,getBoundingClientRect:()=>({top:40,height:700*scale})};
 const card={getBoundingClientRect:()=>({top:40+(distance-scroll+1)*scale})};
 const frame={style:{setProperty:(key,value)=>properties[key]=value}};
 const ctx={currentRoute:'overview',main,card,frame,getComputedStyle:()=>({getPropertyValue:key=>key==='--motion-overhang'?String(overhang):String(height)})};
 vm.createContext(ctx);vm.runInContext(body+';extra=updateBackdrop();',ctx);
 const extra=ctx.extra,top=distance-overhang-extra;
 assert.ok(top<=-64, 'frame starts above panel');
 assert.ok(Math.abs((top+height+extra)-(distance-overhang+height))<1e-8,'bottom unchanged');
 const delivered={padding:padding+extra,innerHeight:height+extra,document:{documentElement:{style:{setProperty:(_,value)=>delivered.result=parseFloat(value)}}}};
 vm.runInNewContext(receiver,delivered);
 assert.ok(Math.abs((top+delivered.result)-(distance-overhang+padding))<1e-8,'Logo unchanged after receiver clamp');
 assert.ok(Math.abs(parseFloat(properties['--motion-top-extra'])-extra)<1e-8);
 ctx.currentRoute='sync';assert.equal(vm.runInContext('updateBackdrop()',ctx),0,'inactive route does not measure');
 count++;
}
const css=fs.readFileSync(base+'overview-dual.css','utf8');assert.equal(css.split('{').length,css.split('}').length);
console.log(JSON.stringify({geometryCases:count,syntaxModules:2,cssBalanced:true,verified:'top coverage, scroll and scale compensation, bottom and Logo invariance, receiver padding, inactive route'}));
