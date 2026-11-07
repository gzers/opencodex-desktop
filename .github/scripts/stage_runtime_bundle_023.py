"""Rebuild pinned names runtime and stage the complete bundle into Tauri resources.

This script prepares artifacts only. It neither installs programs nor starts a
proxy or touches a user HOME. Each download is checksum-pinned before use.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import urllib.request

ROOT=Path(__file__).resolve().parents[2]
BASE_URL='https://github.com/gzers/opencodex-desktop/releases/download/v0.2.22/opencodex-2.97.0-post021.ownerquery.2.tgz'
BASE_SHA='bba03ba0cf02c7bcd1895f63c670cb4a2f4b5272657145e45701b20d5e0f1957'
def digest(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
    return h.hexdigest()
def stage(platform,node,npm,output,base=None):
    output.mkdir(parents=True,exist_ok=True)
    original=Path(base) if base else output/'base-runtime.tgz'
    if not original.exists():
        with urllib.request.urlopen(BASE_URL,timeout=60) as response,original.open('wb') as f:
            size=0
            while block:=response.read(1024*1024):
                size+=len(block)
                if size>32*1024*1024:raise ValueError('base runtime exceeds budget')
                f.write(block)
    if digest(original)!=BASE_SHA:raise ValueError('base runtime digest differs')
    names=output/'maintained-runtime'
    subprocess.run(['python3',str(ROOT/'.github/scripts/build_runtime_023.py'),str(names),'--archive',str(original)],check=True)
    runtime_policy=json.loads((ROOT/'.github/runtime-platforms-023.json').read_bytes())
    archive=names/runtime_policy['runtime']['archive']
    bundle=output/'platform-bundle'
    subprocess.run(['python3',str(ROOT/'.github/scripts/build_desktop_runtime_bundle.py'),'--platform',platform,'--runtime-archive',str(archive),'--build-node',str(node),'--npm-cli',str(npm),'--cache',str(output/'npm-cache'),'--output',str(bundle)],check=True)
    result=json.loads((bundle/'result.json').read_bytes())
    destination=ROOT/'apps/desktop/tauri/resources/runtime-bundle'
    if destination.exists():raise ValueError('runtime resources already exist; use a clean build checkout')
    destination.mkdir(parents=True)
    shutil.copy2(result['archive'],destination/Path(result['archive']).name)
    # A trusted recovery source accompanies the platform closure; never a UI path.
    shutil.copy2(archive,destination/'maintained-runtime.tgz')
    manifest=Path(result['manifest']);shutil.copy2(manifest,destination/'runtime-bundle.manifest.json')
    (destination/'runtime-bundle.manifest.sha256').write_text(digest(manifest)+'\n')
    override={'bundle':{'resources':{'resources/LICENSE':'resources/LICENSE','resources/THIRD-PARTY-NOTICES.txt':'resources/THIRD-PARTY-NOTICES.txt','resources/runtime-bundle/':'runtime-bundle/'}}}
    (ROOT/'apps/desktop/tauri/tauri.runtime-bundle.conf.json').write_text(json.dumps(override,indent=2)+'\n')
    return {'resourceDirectory':str(destination),'descriptorSha256':digest(manifest),'tauriConfig':str(ROOT/'apps/desktop/tauri/tauri.runtime-bundle.conf.json')}
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--platform',choices=['darwin-arm64','win32-x64'],required=True)
    p.add_argument('--build-node',type=Path,required=True);p.add_argument('--npm-cli',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True);p.add_argument('--base',type=Path)
    a=p.parse_args();print(json.dumps(stage(a.platform,a.build_node.resolve(),a.npm_cli.resolve(),a.output.resolve(),a.base),indent=2))
