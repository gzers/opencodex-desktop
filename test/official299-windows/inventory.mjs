// Read-only artifact byte inventory, independent of npm's lock-file claims.
import {readdirSync,readFileSync,lstatSync,writeFileSync} from 'node:fs';
import {resolve,join,relative} from 'node:path';
import {createHash} from 'node:crypto';
const [rootArg,out]=process.argv.slice(2);
if(!rootArg||!out)throw Error('explicit root and new output required');
const root=resolve(rootArg),files=[];
function scan(dir){for(const name of readdirSync(dir).sort()){const p=join(dir,name),s=lstatSync(p);if(s.isSymbolicLink())throw Error('linked inventory input refused: '+relative(root,p));if(s.isDirectory())scan(p);else if(s.isFile())files.push({path:relative(root,p).replaceAll('\\','/'),size:s.size,sha256:createHash('sha256').update(readFileSync(p)).digest('hex')});else throw Error('unsupported input object');}}
scan(root);files.sort((a,b)=>a.path.localeCompare(b.path,'en'));
const canonical=JSON.stringify(files);
writeFileSync(out,JSON.stringify({root,files:files.length,bytes:files.reduce((n,v)=>n+v.size,0),treeSha256:createHash('sha256').update(canonical).digest('hex'),entries:files},null,2),{flag:'wx'});
console.log(JSON.stringify({files:files.length,treeSha256:createHash('sha256').update(canonical).digest('hex')}));
