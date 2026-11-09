import {readFileSync} from 'node:fs';
import vm from 'node:vm';
const s=readFileSync(process.argv[2],'utf8');
const code=s.slice(s.indexOf('function matches('),s.indexOf('const owner='));
const f=vm.runInNewContext(new Bun.Transpiler({loader:'ts'}).transformSync(code)+'\nmatches');
const p="C:\\Users\\fixture\\name space's\\opencodex-service.cmd";
const cases=[['cmd /k rem "'+p+'"',true],['cmd /k rem '+p+'.backup',false],['cmd /k rem C:\\other\\opencodex-service.cmd',false],['cmd /k rem "'+p.toUpperCase()+'"',true],['cmd /k rem prefix'+p,false]];
for(const [c,v] of cases)if(f(c,p)!==v)throw Error('matcher counterexample');
console.log('Five literal boundary matching cases passed');
