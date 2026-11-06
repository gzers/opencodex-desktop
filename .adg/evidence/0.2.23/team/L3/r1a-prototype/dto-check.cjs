const fs=require('node:fs'),cp=require('node:child_process');const Ajv=require('/Users/ezio/.npm/_npx/56d755bf05e5ecee/node_modules/ajv/dist/2020');
const repo='/Users/ezio/.codex/worktrees/ocxd-023-prototype/governance-main',prefix='docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/';
const schema=JSON.parse(cp.execFileSync('git',['show','1d32a9cf4:docs/03-开发实施/0.2.23-contracts/L2-sync-v1/dto.schema.json'],{cwd:repo}));
const sync=require(repo+'/'+prefix+'sync-recovery-023.js');const ajv=new Ajv({strict:false,validateFormats:false,allErrors:true});ajv.addSchema(schema,'l2');let checks=0;const failures=[];
function check(kind,value,label){const valid=ajv.getSchema('l2#/$defs/'+kind)||ajv.compile({$ref:'l2#/$defs/'+kind});checks++;if(!valid(value))failures.push({kind,label,errors:valid.errors});}
for(const [scene]of sync.scenes){const c=sync.create();c.scene(scene);check('RemoteObservation',c.state().observation,scene);const p=c.prepare();if(p)check(c.state().direction==='upload'?'UploadPlan':'DownloadPlan',p,scene);if(c.state().fact)check('OperationFact',c.state().fact,scene);}
for(const scene of ['empty','generation_empty','upload']){const c=sync.create();c.scene(scene);c.prepare();c.confirm();for(let i=0;i<7;i++){check('OperationFact',c.state().fact,scene+'-'+i);c.advance();}}
for(const mode of ['download','export']){const c=sync.create();c.scene('initialization_ambiguous');check('RecoveryPlan',c.recover(0,mode),mode);c.applyRecovery();check('RecoveryReceipt',c.state().receipt,mode);}
const c=sync.create();c.scene('conflict');check('UploadPlan',c.resolve(1),'resolve');
const result={schemaCommit:'1d32a9cf4',checks,passed:checks-failures.length,failures};fs.writeFileSync(__dirname+'/dto-results.json',JSON.stringify(result,null,2));console.log(JSON.stringify(result,null,2));if(failures.length)process.exitCode=1;
