from pathlib import Path
import json,hashlib
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
ROOT=Path(__file__).resolve().parent
original=(BASE/'before_resume_record/storage.rs').read_text(encoding='utf-8')
native=original.replace('strict attachment DATA','independent strict V2 attachment DATA').replace('AttachmentStorage','AttachmentStorageV2').replace('Record::decode','RecordV2::decode').replace('validate_chain(','validate_chain_v2(').replace('record-{','v2-record-{').replace('injected attachment native','injected V2 attachment native').replace('injected actual half-written attachment','injected actual half-written V2 attachment')
native=native.replace('#[derive(Default)]','const MAX_RECORD: usize = 65536;\n#[derive(Default)]',1)
assert 'before_resume_record::' not in native and 'Record::decode' not in native and 'validate_chain(' not in native
target=BASE/'before_resume_record_v2/storage.rs'
assert not target.exists()
target.parent.mkdir(parents=True,exist_ok=True)
target.write_text(native,encoding='utf-8',newline='\n')
v2setup=(BASE/'before_resume_record_v2/tests.rs').read_text(encoding='utf-8').split('#[test]',1)[0]
v2setup=v2setup.replace('fn record(reference: Value)','fn record_with(reference: Value)').replace('record(reference.clone())','record_with(reference.clone())')
v2setup=v2setup.replace('fn bytes(', 'fn encoded_data(').replace('bytes(&r)','encoded_data(&r)')
v1setup=(BASE/'before_resume_record/tests.rs').read_text(encoding='utf-8')
events=v1setup.split('pub(super) fn events()',1)[1].split('#[test]',1)[0]
helpers=v2setup+'fn record() -> Value { record_with(reference()) }\nfn chain(events: &[Value]) -> Vec<Vec<u8>> { raw_chain(reference(), events) }\nfn events()'+events
tests=(BASE/'before_resume_record/storage/tests.rs').read_text(encoding='utf-8')
tests=tests.replace('use super::super::tests::{bytes as encoded_data, chain, events, record};\n','')
tests=tests.replace('use super::*;\n',helpers+'\n',1).replace('AttachmentStorage','AttachmentStorageV2').replace('wl2_r7_attachment_storage_native_','wl2_r7_v2_storage_native_').replace('r7-attachment','r7-v2-attachment').replace('record-000','v2-record-000')
tests=tests.replace('["replay", "scope", "unknown", "gap"]','["replay", "scope", "unknown", "gap", "v1", "reference", "preparation"]')
needle='            "unknown" => {'
new='''            "v1" => {
                v["schema"] = json!("windows-before-resume-recovery-v1");
                encoded_data(&v)
            }
            "reference" => {
                v["reference"]["manifestDigest"] = json!("0".repeat(64));
                encoded_data(&v)
            }
            "preparation" => {
                v["reference"] = preparation(false);
                encoded_data(&v)
            }
'''+needle
assert needle in tests
tests=tests.replace(needle,new,1)
out=BASE/'before_resume_record_v2/storage/tests.rs'
out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(tests,encoding='utf-8',newline='\n')
module=BASE/'before_resume_record_v2.rs'
text=module.read_text(encoding='utf-8')
assert '\nmod storage;' not in text
text=text.replace('mod chain;','mod chain;\nmod storage;\npub(super) use storage::AttachmentStorageV2;')
module.write_text(text,encoding='utf-8',newline='\n')
(ROOT/'implementation-origin.json').write_text(json.dumps(dict(method='Independent V2 native durability patterned on already verified Windows V1 storage; no V1 code changed/called or record conversion',template='before_resume_record/storage.rs',templateSha256=hashlib.sha256(original.encode()).hexdigest(),currentR2Touched=False),indent=2),encoding='utf-8')
print('created independent V2 storage and native tests; no original V1 implementation edits')
