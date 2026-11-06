const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict'),crypto=require('node:crypto');
const root='/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes',runtime='/private/tmp/post022-channel-name-review';
const ts=require(root+'/apps/desktop/ui/node_modules/typescript');
const sha=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
function functions(path,names){const source=ts.createSourceFile(path,fs.readFileSync(path,'utf8'),ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);return source.statements.filter(n=>ts.isFunctionDeclaration(n)&&names.includes(n.name?.text)).map(n=>n.getText(source)).join(String.fromCharCode(10));}
const derive=runtime+'/src/codex/catalog/derive-entry.ts',codec=runtime+'/src/providers/slug-codec.ts';
const runtimeSource='const SLUG_ALIAS_SEPARATOR="-";'+functions(codec,['encodeRoutedModelId','routedSlug'])+functions(derive,['routedDisplayName'])+'\nmodule.exports={routedSlug,routedDisplayName};';
const c={module:{exports:{}}};c.exports=c.module.exports;vm.runInNewContext(ts.transpileModule(runtimeSource,{compilerOptions:{module:ts.ModuleKind.CommonJS}}).outputText,c);
const slug=(key,name)=>{const config={providers:{[key]:{name}}},s=c.module.exports.routedSlug(key,'gpt-6.1-sol');return c.module.exports.routedDisplayName(s,undefined,config)};
const displayNameOnly=slug('Krill-1','Krill'),actualKeyRename=slug('Krill','Krill');assert.equal(displayNameOnly,'Krill-1/gpt-6.1-sol');assert.equal(actualKeyRename,'Krill/gpt-6.1-sol');
const file=root+'/apps/desktop/ui/src/features/models/channelRename.ts';
const ui={module:{exports:{}},exports:null,require:n=>{assert.equal(n,'./channelActions');return {protectedChannel:()=>false}}};ui.exports=ui.module.exports;
vm.runInNewContext(ts.transpileModule(fs.readFileSync(file,'utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS}}).outputText,ui);
const ch=(label,key)=>({id:'00000000-0000-4000-8000-000000000001',label,connection:{providerId:key}});
const fresh=ch('Krill-1','Krill-1');ui.module.exports.renameChannelLabel(fresh,'Krill',[structuredClone(fresh)]);assert.equal(fresh.connection.providerId,'Krill');
const inherited=ch('Krill','Krill-1'),saved=structuredClone(inherited);ui.module.exports.renameChannelLabel(inherited,'Krill',[saved]);assert.equal(inherited.connection.providerId,'Krill-1');
const revert=structuredClone(saved);ui.module.exports.renameChannelLabel(revert,'Krill-tmp',[saved]);ui.module.exports.renameChannelLabel(revert,'Krill',[saved]);assert.equal(revert.connection.providerId,'Krill-1');
const result={runtimeArchive:'opencodex-2.97.0-post021.ownerquery.2.tgz',runtimeArchiveSha256:'bba03ba0cf02c7bcd1895f63c670cb4a2f4b5272657145e45701b20d5e0f1957',method:'Actual function declarations from hash-verified runtime archive; TypeScript transpilation in VM. Actual manager channelRename.ts module; only protectedChannel mocked false for custom fixture.',cases:{displayNameOnly,actualKeyRename,freshRename:fresh,inheritedSameLabel:inherited,revertToSavedLabel:revert},sourceHashes:{managerRename:sha(file),runtimeDeriveEntry:sha(derive),runtimeSlugCodec:sha(codec)},limits:['No actual daily configuration/catalog read or modified','No full runtime catalog pipeline, server reload, Codex restart or picker UI run','Establishes field semantics and reproducible conditional path; not user现场root cause']};
fs.writeFileSync(runtime+'/field-semantics-result.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result,null,2));
