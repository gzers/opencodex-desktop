"""Run source-extracted existing Rust literal matcher against the frozen replay.
Only builds under TemporaryDirectory; no code checkout edits or runtime access.
"""
from pathlib import Path
import importlib.util,json,tempfile,subprocess,hashlib
BASE=Path(__file__).resolve().parent
identity=json.loads((BASE/'source-identity.json').read_text())
# Locate current code checkout by walking to repository .adg root.
repo=next(p for p in BASE.parents if (p/'.adg/project.json').exists())
code=repo.parent/'ocx-0219-fixes'
rel='apps/desktop/tauri/src/modules/model_config/matching.rs'
src=(code/rel).read_text()
assert hashlib.sha256((code/rel).read_bytes()).hexdigest()==identity['files'][rel]
spec=importlib.util.spec_from_file_location('analysis_evaluate',BASE/'evaluate.py')
evalmod=importlib.util.module_from_spec(spec);spec.loader.exec_module(evalmod)
snapshot=json.loads((BASE/'model-id-snapshot.json').read_text());seed=json.loads((BASE/'release-seed-rules.json').read_text())
models=sorted({m for p in snapshot['providers'] for m in p['model_ids']})
rows=[]
for f in seed['families']:
 r=f['content']['matching'];assert r['channelMode']=='all'
 for p in r['include']+r.get('exclude',[]):
  for m in models:
   assert not any(c in p['value']+m for c in '\n\t\r')
   rows.append('\t'.join([p['kind'],p['value'],m,'1' if r.get('caseSensitive',False) else '0','1' if evalmod.literal(p,m,r.get('caseSensitive',False)) else '0']))
# Additional independent expected literal glob regressions.
for kind,p,m,expected in [('glob','gpt.[0-9]','gpt.5',False),('glob','gpt-5?5','gpt-5x5',True),('glob','gpt-*','gpt-5.5',True),('exact','gpt-5.5','gpt-5-5',False),('prefix','gpt-','notgpt-5.5',False)]:
 rows.append('\t'.join([kind,p,m,'0','1' if expected else '0']))
# Extract actual functions unchanged, supplying only the minimal Pattern schema.
funcs=src[src.index('fn folded('):src.index('#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]')]
header='''use std::io::{self, BufRead};
enum MatchKind { Exact, Prefix, Glob }
struct Pattern { kind: MatchKind, value: String }
'''
main='''fn main() {
 let mut checked=0usize;
 for line in io::stdin().lock().lines() {
  let line=line.unwrap(); let v:Vec<&str>=line.split('\\t').collect(); assert_eq!(v.len(),5);
  let kind=match v[0] {"exact"=>MatchKind::Exact,"prefix"=>MatchKind::Prefix,"glob"=>MatchKind::Glob,_=>panic!("kind")};
  let p=Pattern{kind,value:v[1].into()};
  let actual=matches(&p,v[2],v[3]=="1"); assert_eq!(actual,v[4]=="1","row {}",checked);checked+=1;
 }
 println!("checked={}",checked);
}
'''
with tempfile.TemporaryDirectory(prefix='ocxd-model-match-analysis-') as temp:
 tmp=Path(temp);(tmp/'main.rs').write_text(header+funcs+main)
 build=subprocess.run(['rustc','--edition=2021',str(tmp/'main.rs'),'-o',str(tmp/'matcher')],capture_output=True,text=True)
 assert build.returncode==0,build.stderr
 run=subprocess.run([str(tmp/'matcher')],input='\n'.join(rows)+'\n',capture_output=True,text=True)
 assert run.returncode==0,run.stderr
 result={'status':'passed','actual_source_sha256':identity['files'][rel],'method':'original folded/glob/matches source extracted unchanged into isolated Rust driver','checks':len(rows),'output':run.stdout.strip(),'temporary_workspace':temp,'temporary_workspace_removed':None,'not_checked':'Full production resolver/template adoption, capability semantics and new-rule production integration.'}
result['temporary_workspace_removed']=not Path(result['temporary_workspace']).exists()
assert result['temporary_workspace_removed']
(BASE/'existing-rust-crosscheck.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(result,ensure_ascii=False,indent=2))
