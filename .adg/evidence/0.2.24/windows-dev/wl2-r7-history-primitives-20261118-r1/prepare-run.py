from pathlib import Path
import json,re
ROOT=Path(__file__).resolve().parent
base='apps/desktop/tauri/src/infrastructure/'
files=[base+'windows_official_runtime024/before_resume.rs']+[base+'windows_client_originals024/'+p for p in [
 'before_resume_record/storage.rs','before_resume_record_v2/storage.rs',
 'before_resume_record/storage/tests.rs','before_resume_record_v2/storage/tests.rs',
 'before_resume_record/storage/tests/history_tests.rs','before_resume_record_v2/storage/tests/history_tests.rs',
 'tree/anchored/before_resume/quiesce_tests.rs','tree/anchored/before_resume/quiesce_tests/history_tests.rs']]
assert len(files)==9
(ROOT/'sources.json').write_text(json.dumps(files,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-return-validation-20261117-r1/run-product.ps1').read_text(encoding='utf-8')
text=text.replace('wl2_r7_return_validation_native_','wl2_r7_history_primitives_native_')
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline='\n')
native=text
native,n=re.subn(r'^\$arguments=@\(.*\)$',"$arguments=@('check','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1')",native,flags=re.M);assert n==1
native=native.replace("'test/windows024-native/target/w1c-product'","'test/windows024-native/target/wl2-primitives'")
native=native.replace("[string]$Run='product01'","[string]$Run='backend02'")
(ROOT/'run-backend.ps1').write_text(native,encoding='utf-8',newline='\n')
print('9 actual sources, full product native tests and independent native backend check prepared')
