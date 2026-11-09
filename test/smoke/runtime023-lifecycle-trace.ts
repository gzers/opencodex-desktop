/** Candidate-only observation. No config, credential, response body or raw error. */
import {appendFileSync,existsSync,lstatSync,realpathSync} from 'node:fs';
import {dirname,isAbsolute,relative,resolve} from 'node:path';
const labels=new Set(['manager-risk','manager-stop','ownership-preflight','official-teardown','native-client-restore','grok-restore']);
let count=0;
function note(label:string,state:'begin'|'end'|'throw',elapsedMilliseconds:number){
  const output=process.env.OCXD_CANDIDATE_TRACE_FILE;
  if(!output)return;
  try{
    if(process.env.GITHUB_ACTIONS!=='true'||!labels.has(label)||count>=64||!isAbsolute(output))return;
    const root=process.env.RUNNER_TEMP;if(!root||!isAbsolute(root))return;
    const canonical=realpathSync(root),parent=realpathSync(dirname(output));
    const rel=relative(canonical,parent);if(rel.startsWith('..')||isAbsolute(rel))return;
    if(existsSync(output)){const stat=lstatSync(output);if(!stat.isFile()||stat.isSymbolicLink()||stat.nlink!==1||stat.size>16384)return;}
    // Test-owned file, fixed labels and durations only; no response/error/argv.
    appendFileSync(resolve(output),JSON.stringify({label,state,elapsedMilliseconds})+'\n',{mode:0o600});count++;
  }catch{/* Observation failure cannot alter the original official operation. */}
}
export function traceSync<T>(label:string,work:()=>T):T{
  const started=performance.now();note(label,'begin',0);
  try{const value=work();note(label,'end',performance.now()-started);return value;}
  catch(error){note(label,'throw',performance.now()-started);throw error;}
}
export async function traceAsync<T>(label:string,work:()=>Promise<T>):Promise<T>{
  const started=performance.now();note(label,'begin',0);
  try{const value=await work();note(label,'end',performance.now()-started);return value;}
  catch(error){note(label,'throw',performance.now()-started);throw error;}
}
