"""Prepare a read-only driver from an explicitly frozen production commit.

No renderer matcher and no handwritten preview implementation: the production
ConditionPreviewRequest/response declarations and preview_conditions function
are copied byte-for-byte; shared Rust modules are included by absolute path.
The desktop IPC transport is replaced, not its preview semantics.
"""
from pathlib import Path
import hashlib,json,re,subprocess,sys

root=Path(sys.argv[1]).resolve()
commit=sys.argv[2]
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()==commit
out=Path(__file__).parent.resolve()
driver=out/'driver';driver.mkdir(exist_ok=True)
command=root/'apps/desktop/tauri/src/commands/model_config_v2.rs'
source=command.read_text()
start=source.rfind('#[derive(',0,source.index('pub struct ConditionPreviewRequest'))
end=source.index('#[tauri::command]',source.index('pub fn preview_conditions('))
block=source[start:end]
(driver/'production-preview.rs').write_text(block)
modules=['schema','schema_v2','matching_v2','source_v2']
paths={name:root/f'apps/desktop/tauri/src/modules/model_config/{name}.rs' for name in modules}
for p in [command,*paths.values()]:
    rel=str(p.relative_to(root))
    committed=subprocess.check_output(['git','show',f'{commit}:{rel}'],cwd=root)
    assert committed==p.read_bytes(),f'Dirty production input: {rel}'
rust='''#![allow(dead_code)]
mod errors { #[derive(Debug)] pub struct AppError; impl std::fmt::Display for AppError { fn fmt(&self,f:&mut std::fmt::Formatter)->std::fmt::Result {write!(f,"isolated-driver unrelated storage error")} } }
pub mod modules { pub mod model_config {
'''
rust+=''.join(f'#[path = {json.dumps(str(p))}] pub mod {name};'+chr(10) for name,p in paths.items())
rust+='''}}
pub mod commands {pub mod model_config_v2 {
use crate::modules::model_config::{matching_v2,schema::{ModelRules,ModelSelection},schema_v2::*};
use serde::{Deserialize,Serialize};use std::collections::BTreeMap;
include!("production-preview.rs");
}}
fn main(){
 use std::io::Read;
 let mut input=String::new();std::io::stdin().read_to_string(&mut input).unwrap();
 let input:serde_json::Value=serde_json::from_str(&input).unwrap();
 let result=match input["command"].as_str().unwrap(){
  "model_v2_validate_content"=>{
   let content=serde_json::from_value(input["args"]["content"].clone()).unwrap();
   modules::model_config::matching_v2::validate_content_v2(&content).map(|_|serde_json::Value::Null)
  },
  "model_v2_preview_conditions"=>{
   let request=serde_json::from_value(input["args"]["request"].clone()).unwrap();
   commands::model_config_v2::preview_conditions(request).map(|v|serde_json::to_value(v).unwrap())
  },
  _=>panic!("isolated driver forbids this command")
 };
 let output=match result {Ok(value)=>serde_json::json!({"ok":true,"value":value}),Err(error)=>serde_json::json!({"ok":false,"error":error})};
 println!("{}",serde_json::to_string(&output).unwrap());
}
'''
(driver/'main.rs').write_text(rust)
(driver/'Cargo.toml').write_text('''[package]
name="l3-condition-trace-driver"
version="0.1.0"
edition="2021"
[[bin]]
name="l3-condition-trace-driver"
path="main.rs"
[dependencies]
serde={version="1",features=["derive"]}
serde_json="1"
schemars="0.8.22"
sha2="0.10"
uuid="1.26.1"
chrono={version="0.4",features=["serde"]}
thiserror="2"
reqwest={version="0.12",default-features=false}
''')
manifest={'commit':commit,'root':str(root),'productionSha256':{str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [command,*paths.values()]},'extractedPreviewSha256':hashlib.sha256(block.encode()).hexdigest(),'method':'Exact production pure preview function and DTO block; included unchanged production modules. Only desktop IPC transport replaced. No configuration/filesystem/network business operation.'}
(out/'driver-source-manifest.json').write_text(json.dumps(manifest,indent=2)+chr(10))
print(json.dumps(manifest,indent=2))
