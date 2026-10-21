// Source/consumer evidence, not rendered acceptance. No app is launched and no
// business data or personal files are read. Output belongs in an evidence root.
import fs from 'node:fs'
import path from 'node:path'
import {fileURLToPath} from 'node:url'
import {createHash} from 'node:crypto'
export function componentInventory(src,registry){
 const walk=dir=>fs.readdirSync(dir,{withFileTypes:true}).flatMap(entry=>entry.isDirectory()?walk(path.join(dir,entry.name)):/\.(vue|ts|mjs|js)$/.test(entry.name)?[path.join(dir,entry.name)]:[])
 const files=walk(src),relative=file=>path.relative(src,file).split(path.sep).join('/'),known=new Set(files),reverse=new Map(),contents=new Map(files.map(file=>[file,fs.readFileSync(file,'utf8')]))
 function resolve(from,spec){
  const base=spec.startsWith('@/')?path.join(src,spec.slice(2)):spec.startsWith('.')?path.resolve(path.dirname(from),spec):null
  if(!base)return null
  return [base,...['.vue','.ts','.mjs','.js','/index.ts','/index.js'].map(ext=>base+ext)].find(p=>known.has(p))??null
 }
 for(const [file,source] of contents){
  for(const match of source.matchAll(/(?:\bfrom\s*|\bimport\s*\(\s*|\bimport\s*)['"]([^'"]+)['"]/g)){
   const target=resolve(file,match[1]);if(!target)continue
   if(!reverse.has(target))reverse.set(target,new Set());reverse.get(target).add(file)
  }
 }
 const rows=registry.map(entry=>{
  const implementations=entry.implementations.filter(p=>!p.startsWith('../')).map(p=>path.resolve(src,p)).filter(p=>known.has(p))
  const direct=new Set(implementations.flatMap(p=>[...reverse.get(p)??[]])),transitive=new Set(),pending=[...direct]
  while(pending.length){const next=pending.shift();if(transitive.has(next))continue;transitive.add(next);pending.push(...reverse.get(next)??[])}
  return{id:entry.id,kind:entry.kind,gallery:entry.gallery,declaredConsumers:entry.consumers,directConsumers:[...direct].map(relative).sort(),applicationReach:[...transitive].filter(p=>!relative(p).startsWith('dev/')).map(relative).sort(),developmentReach:[...transitive].filter(p=>relative(p).startsWith('dev/')).map(relative).sort(),implementations:entry.implementations.map(p=>{const absolute=p.startsWith('../')?path.resolve(src,'..',p):path.resolve(src,p);return{path:p,exists:fs.existsSync(absolute),sha256:fs.existsSync(absolute)&&fs.statSync(absolute).isFile()?createHash('sha256').update(fs.readFileSync(absolute)).digest('hex'):null}})}
 })
 const registered=new Set(registry.flatMap(entry=>entry.implementations).map(p=>path.resolve(src,p)))
 const publicFiles=files.filter(p=>/^components\/(ui|layout|patterns)\//.test(relative(p)))
 const unregistered=publicFiles.filter(p=>!registered.has(p)).map(relative)
 return{scope:'static import graph includes types/dynamic dependencies; reach is not runtime execution; visual/functional/platform acceptance separate',registryCount:rows.length,publicSourceCount:publicFiles.length,unregistered,missingImplementations:rows.flatMap(row=>row.implementations.filter(p=>!p.exists).map(p=>({id:row.id,path:p.path}))),components:rows}
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const {componentRegistry}=await import('../src/components/registry.ts'),src=fileURLToPath(new URL('../src/',import.meta.url)),result=componentInventory(src,componentRegistry)
 const output=process.argv[2];if(output)fs.writeFileSync(output,JSON.stringify(result,null,2)+'\n');else process.stdout.write(JSON.stringify(result,null,2)+'\n')
 if(result.unregistered.length||result.missingImplementations.length)process.exitCode=1
}
