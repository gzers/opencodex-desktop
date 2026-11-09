"""Compare actual old/new archived validators without native calls or service startup."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('record_path_fix', ROOT / '.github/scripts/runtime_record_path_023.py')
fix = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fix)


def read(archive):
    with tarfile.open(archive) as source:
        return {m.name: source.extractfile(m).read() for m in source if m.isfile()}


def verify(before, after):
    old, new = read(before), read(after)
    member = 'package/' + fix.MEMBER
    assert hashlib.sha256(old[member]).hexdigest() == fix.BEFORE_SHA256
    assert new[member] == fix.apply(old[member])
    # Metadata and exactly one source guard may differ. Native calls and all
    # owner/DACL/identity/ancestor/atomic bodies must remain byte-for-byte intact.
    changed = {name for name in old if old[name] != new[name]}
    assert set(old) == set(new)
    assert changed == {member, 'package/package.json', 'package/src/generated/compatibility-version.json'}
    package = json.loads(new['package/package.json'])
    assert package['version'] == '2.97.0-desktop023.lifecycle.3'
    for row in json.loads(new['package/src/generated/compatibility-version.json'])['files']:
        name = 'package/' + row['path']
        if name in new:
            assert hashlib.sha256(new[name]).hexdigest() == row['sha256'], name
    for altered in [old[member] + b' ', new[member]]:
        try:
            fix.apply(altered)
        except ValueError:
            pass
        else:
            raise AssertionError('Unreviewed/repeated source transform accepted')

    def validator(data):
        text = data.decode()
        return text[text.index('function validate('):text.index('function info(')].replace('(path:string):void', '(path)').replace('[0]!', '[0]')

    slash = chr(92)
    prefix = slash * 2 + '?' + slash
    device = slash * 2 + '.' + slash
    ordinary = 'C:' + slash + 'Users' + slash + 'ocxaccu1' + slash + '.codex-desktop' + slash + 'opencodex' + slash + 'home' + slash + 'runtime-port.json'
    cases = [
        ('canonical-local-drive', prefix + ordinary, False, True),
        ('ordinary-equivalent', ordinary, True, True),
        ('extended-UNC', prefix + 'UNC' + slash + 'host' + slash + 'share' + slash + 'runtime-port.json', False, False),
        ('device-namespace', device + ordinary, False, False),
        ('drive-relative', prefix + 'C:relative' + slash + 'runtime-port.json', False, False),
        ('parent-traversal', prefix + 'C:' + slash + 'x' + slash + '..' + slash + 'runtime-port.json', False, False),
        ('alternate-stream', prefix + ordinary + ':secret', False, False),
        ('trailing-space', prefix + ordinary + ' ', False, False),
        ('reserved-name', prefix + 'C:' + slash + 'NUL' + slash + 'runtime-port.json', False, False),
    ]
    # Type-only annotations are erased; both versions execute their actual
    # archived predicate with Node's platform-independent path.win32.
    script = '''
const assert=require('node:assert/strict');
const {win32}=require('node:path');
const payload=JSON.parse(require('node:fs').readFileSync(0,'utf8'));
const build=s=>new Function('win32','refusal','return ('+s+')')(win32,()=>{throw Error('refused')});
const old=build(payload.old), next=build(payload.next);
const result=[];
for(const [name,path,expectedOld,expectedNew] of payload.cases){
 const run=f=>{try{f(path);return true}catch{return false}};
 const before=run(old),after=run(next);
 assert.equal(before,expectedOld,name+' before');
 assert.equal(after,expectedNew,name+' after');
 result.push({name,before,after});
}
process.stdout.write(JSON.stringify(result));
'''
    output = subprocess.run(['node', '-e', script], input=json.dumps({'old': validator(old[member]), 'next': validator(new[member]), 'cases': cases}), text=True, capture_output=True, check=True)
    return {'accepted': False, 'nativeWindowsVerified': False, 'changedMembers': sorted(changed),
            'guardCases': json.loads(output.stdout), 'sourceDriftRejected': True, 'repeatedPatchRejected': True,
            'nativeCheckBodiesUnchanged': True, 'compatibilityPresentMembersVerified': True,
            'beforeMemberSha256': fix.BEFORE_SHA256, 'afterMemberSha256': hashlib.sha256(new[member]).hexdigest()}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('before', type=Path)
    parser.add_argument('after', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.before, args.after), ensure_ascii=False, indent=2))
