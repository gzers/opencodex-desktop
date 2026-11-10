// Exercise the real unchanged official producer on a synthetic directory only.
import {join} from 'node:path';
import {writeFileSync} from 'node:fs';
const [source,root,out]=process.argv.slice(2);
const {syncClaudeAgentDefs}=await import(join(source,'src/claude/agents-inject.ts'));
const defs=['gpt-6-1-sol','new'].map(n=>({file:'ocx-'+n+'.md',name:'ocx-'+n,model:'fixture/'+n,description:'synthetic targeted recovery test',blockedSkills:[]}));
const result=syncClaudeAgentDefs(defs,root);
if(!result||result.length!==2)throw Error('official producer did not commit fixture write-set');
writeFileSync(out,JSON.stringify({result,officialSource:join(source,'src/claude/agents-inject.ts'),utc:new Date().toISOString()}));
