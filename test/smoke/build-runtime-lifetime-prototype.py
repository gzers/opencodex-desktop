"""Canonical producer archive from the verified record prototype.
No release pin promotion; no trace hooks or dependency/vendor files in archive.
"""
from pathlib import Path
import argparse
import ast
import hashlib
import importlib.util
import json

ROOT=Path(__file__).resolve().parents[2]
BASE='33887e710fad9d5ae78eb4f85c6d02f7da0d413238437a65033b5a79fba168e4'
VERSION='2.97.0-desktop023.lifecycle.1-producer-prototype'
POSTIMAGES={
 'src/lib/windows-runtime-start-boot.ts':'ab65e461cbc3017ea13b2d3cc633928a5bfda1a94718472b7fc7b706a5b62973',
 'src/cli/index.ts':'21a6e7ab4ee7ec452b18bf57d1fdbf35542e2832256d1d58b117a134b114237b',
 'src/config/process-state.ts':'72dc4297a570b548b2700638b862c3f9782dc3831538d1fd604e75dd31c8e12f',
 'src/server/index/serve-options.ts':'e42646dfe5a025004d781f25a08ce0a28ce69cd199af0765f813461bf1052d8e',
 'src/lib/windows-runtime-lifetime.ts':'6276147ab1becc2368a060ed2478277c50c87ee4d500dfab5be59e61ab2f7837',
}
spec=importlib.util.spec_from_file_location('runtime_builder',ROOT/'.github/scripts/build_runtime_023.py')
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)


def build(archive,output):
 raw=Path(archive).read_bytes();assert b.sha(raw)==BASE
 assert not output.exists(),'Original evidence must not be overwritten'
 original,modes=b.read_archive(raw)
 assert json.loads(original['package.json'])['version']=='2.97.0-desktop023.lifecycle.1-record-prototype'
 # Use exactly the producer transforms already executed in actual Windows.
 tree=ast.parse((ROOT/'test/smoke/stage-runtime-lifetime-candidate.py').read_text())
 names={'replace_one','cli','state','health'}
 functions=ast.Module(body=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name in names],type_ignores=[])
 ns={};exec(compile(functions,'verified-producer-hooks','exec'),ns)
 results=[]
 preimages={}
 for _ in range(2):
  files=dict(original);new_modes=dict(modes)
  helper='src/lib/windows-runtime-lifetime.ts';assert helper not in files
  files[helper]=(ROOT/'test/smoke/runtime023-windows-runtime-lifetime.ts').read_bytes();new_modes[helper]=0o644
  boot='src/lib/windows-runtime-start-boot.ts';assert boot not in files;files[boot]=(ROOT/'test/smoke/runtime023-windows-start-boot.ts').read_bytes();new_modes[boot]=0o644
  for path,hook in [('src/cli/index.ts','cli'),('src/config/process-state.ts','state'),('src/server/index/serve-options.ts','health')]:
   preimages[path]=b.sha(files[path]);files[path]=ns[hook](files[path].decode()).encode()
  assert {p:b.sha(files[p]) for p in POSTIMAGES}==POSTIMAGES,'Reviewed producer postimages must remain exact'
  package=json.loads(files['package.json']);package['version']=VERSION
  package['desktop023LifetimeProducerPrototype']=dict(candidateOnly=True,baseArchiveSha256=BASE,
   originalRuntime='2.97.0-desktop023.names.2',sourceFiles=POSTIMAGES,
   priorProducerEvidenceBeforeEarlyBoot=dict(run=42586501118,source='4162f5234c911be28e371f9a8463609ae1fa5d31',artifactSha256='9d5f037d8ee25fd4d5e4f2e9b7010638219c13e736c63c888c6d7b3f59528f0b',accepted=False),
   limits='Producer/record only; production Desktop consumer, exact client file restoration, same-source dual-SID and final release remain unproved. Early bootstrap order changed after the prior diagnostic; Windows execution pending. Trace hooks excluded.')
  files['package.json']=b.json_bytes(package)
  compatibility=json.loads(files[b.COMPATIBILITY]);entries={r['path']:r for r in compatibility['files']}
  for p in ['package.json',*POSTIMAGES]:entries[p]={**entries.get(p,{'path':p}),'sha256':b.sha(files[p])}
  compatibility['files']=[entries[p] for p in sorted(entries)]
  files[b.COMPATIBILITY]=b.json_bytes(compatibility)
  assert {p for p in files if p not in original or files[p]!=original[p]}=={'package.json',b.COMPATIBILITY,*POSTIMAGES}
  results.append(b.canonical_archive(files,new_modes))
 assert results[0]==results[1]
 unpacked,_=b.read_archive(results[0]);assert unpacked==files
 assert 'src/lib/desktop-lifecycle-trace.ts' not in unpacked
 assert not any(p.startswith('node_modules/') for p in unpacked)
 absent=[]
 for entry in json.loads(files[b.COMPATIBILITY])['files']:
  if entry['path'] in files:assert b.sha(files[entry['path']])==entry['sha256']
  else:absent.append(entry['path'])
 assert sorted(absent)==['bun.lock','scripts/model-metadata.source.json']
 output.mkdir();archive=output/('opencodex-'+VERSION+'.tgz');archive.write_bytes(results[0])
 report=dict(accepted=False,diagnosticOnly=True,candidateOnly=True,productionPinsChanged=False,version=VERSION,
  baseArchiveSha256=BASE,archive=archive.name,bytes=len(results[0]),sha256=b.sha(results[0]),
  twoIndependentBuildsIdentical=True,memberCount=len(files),originalSourcePreimages=preimages,
  reviewedProducerPostimages=POSTIMAGES,traceHooksIncluded=False,dependencyTreesIncluded=False,
  compatibilityInheritedAbsentMembersUnchanged=absent,lifetimeConsumerIntegrated=False)
 (output/'verification.json').write_bytes(b.json_bytes(report));print(json.dumps(report))


if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--archive',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
 a=p.parse_args();build(a.archive,a.output)
