"""Adopted runtime delivery must fail closed before publication."""
import base64
import copy
import hashlib
import importlib.util
import json
import io
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]

def load(name, relative):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

publisher = load('runtime_publisher', '.github/scripts/publish_release.py')
builder = load('maintained_builder', '.github/scripts/build_maintained_runtime.py')

class MaintainedSignatureTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.key, self.secret = self.root/'key.pub', self.root/'key.secret'
        subprocess.run(['minisign', '-G', '-W', '-p', str(self.key), '-s', str(self.secret)], check=True, capture_output=True)
        self.payload = b'adopted runtime immutable archive bytes'
        self.name = 'opencodex-2.97.0-post021.ownerquery.2.tgz'
        target = self.root/'fixture'
        target.write_bytes(self.payload)
        subprocess.run(['minisign', '-S', '-m', str(target), '-s', str(self.secret)], check=True, capture_output=True)
        self.policy = {'archive': self.name, 'version': '2.97.0-post021.ownerquery.2', 'bytes': len(self.payload),
                       'sha256': hashlib.sha256(self.payload).hexdigest(), 'maintainer': 'example/desktop',
                       'installation': 'explicit-offline-import'}
        self.files = {self.name:self.payload, self.name+'.sig':base64.b64encode((self.root/'fixture.minisig').read_bytes()),
                      'maintained-runtime.json':json.dumps(self.policy).encode(),
                      'maintained-runtime.sha256':(self.policy['sha256']+'  '+self.name+'\n').encode()}
        self.release = {'draft':True,'assets':[{'name':n,'browser_download_url':'https://github.com/example/desktop/releases/download/v0.2.22/'+n,
                           'state':'uploaded','size':len(b)} for n,b in self.files.items()]}
        self.real_run = subprocess.run

    def verify(self):
        def execute(args, **kwargs):
            if args[0] != 'gh':
                return self.real_run(args, **kwargs)
            Path(args[args.index('--output')+1]).write_bytes(self.files[args[args.index('--pattern')+1]])
            return subprocess.CompletedProcess(args,0,'','')
        with patch.object(publisher.subprocess,'run',side_effect=execute):
            publisher.verify_maintained_runtime('example/desktop','v0.2.22',self.release,self.policy,self.root,self.key)

    def test_exact_adopted_signed_archive_and_manifest_pass(self):
        self.verify()

    def test_same_size_tampering_rejected_by_tagged_hash(self):
        self.files[self.name] = b'X'+self.payload[1:]
        with self.assertRaisesRegex(publisher.PublishError,'adopted bytes'):
            self.verify()

    def test_missing_signature_blocks_delivery(self):
        self.release['assets']=[a for a in self.release['assets'] if not a['name'].endswith('.sig')]
        with self.assertRaisesRegex(publisher.PublishError,'Missing maintained runtime asset'):
            self.verify()

    def test_metadata_cannot_change_adopted_version(self):
        other={**self.policy,'version':'2.97.0-post021.ownerquery.3'}
        self.files['maintained-runtime.json']=json.dumps(other).encode()
        with self.assertRaisesRegex(publisher.PublishError,'manifest differs'):
            self.verify()

    def test_hash_file_must_match_adopted_archive(self):
        self.files['maintained-runtime.sha256']=b'0'*64+b'  '+self.name.encode()+b'\n'
        with self.assertRaisesRegex(publisher.PublishError,'checksum file'):
            self.verify()

    def test_wrong_signing_key_rejected(self):
        subprocess.run(['minisign','-G','-W','-p',str(self.root/'wrong.pub'),'-s',str(self.root/'wrong.secret')],check=True,capture_output=True)
        self.key=self.root/'wrong.pub'
        with self.assertRaises(subprocess.CalledProcessError):
            self.verify()

class OriginalPinTests(unittest.TestCase):
    def test_changed_original_rejected_before_any_output(self):
        policy=json.loads(builder.POLICY.read_bytes())
        registry={'name':'@bitkyc08/opencodex','version':policy['baseVersion'],'gitHead':policy['baseGitHead']}
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);metadata=root/'registry.json';archive=root/'original.tgz';output=root/'output'
            metadata.write_text(json.dumps(registry));archive.write_bytes(b'not-the-reviewed-original')
            with self.assertRaisesRegex(AssertionError,'original archive changed'):
                builder.build(output,metadata,archive)
            self.assertFalse(output.exists())

class PublicRuntimeTests(unittest.TestCase):
    def test_public_bytes_checked_without_authentication(self):
        data = b"public adopted runtime"
        policy = {"archive":"opencodex-2.97.0-post021.ownerquery.2.tgz",
                  "bytes":len(data),"sha256":hashlib.sha256(data).hexdigest()}
        with patch.object(publisher.urllib.request, "urlopen", return_value=io.BytesIO(data)) as get:
            publisher.verify_public_runtime("example/desktop", "v0.2.22", policy)
        self.assertFalse(get.call_args.args[0].has_header("Authorization"))

    def test_public_corruption_is_not_success(self):
        policy = {"archive":"opencodex-2.97.0-post021.ownerquery.2.tgz", "bytes":3,"sha256":"0"*64}
        with patch.object(publisher.urllib.request, "urlopen", side_effect=lambda *a,**k:io.BytesIO(b"bad")), \
             patch.object(publisher.time,"sleep"):
            with self.assertRaisesRegex(publisher.PublishError,"public runtime differs"):
                publisher.verify_public_runtime("example/desktop", "v0.2.22", policy)

if __name__ == '__main__':
    unittest.main()
