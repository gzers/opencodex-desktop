import { writeFileSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
const configPath=join(process.env.OPENCODEX_HOME!,'config.json');
const fixture={port:0,defaultProvider:'fixture',providers:{fixture:{adapter:'openai-responses',baseUrl:'https://fixture.invalid/v1',apiKey:'synthetic-fixture-not-real',models:['fixture-model']}}};
writeFileSync(configPath,JSON.stringify(fixture));
const {loadConfig}=await import('./official299/src/config');
const {readConfigAdmissionSnapshot}=await import('./official299/src/config/diagnostics');
const loaded=loadConfig(),observed=readConfigAdmissionSnapshot();
console.log(JSON.stringify({kind:observed.kind,source:observed.diagnostics.source,error:observed.diagnostics.error,loadedProviders:Object.keys(loaded.providers).length,observedProviders:Object.keys(observed.diagnostics.config.providers).length,fileUnchanged:readFileSync(configPath,'utf8')===JSON.stringify(fixture)}));
