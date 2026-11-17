from pathlib import Path
ROOT=Path(__file__).resolve().parent
text=(ROOT.parent/'wl2-r7-full-history-20261118-r1/run-regressions.py').read_text(encoding='utf-8')
start=text.index('runs=[');end=text.index('\nfor name,case in runs:',start)
text=text[:start]+"runs=[('regression02','wl2_r7_worker_native_'),('regression03','wl2_r7_concrete_driver_native_'),('regression04','wl2_r7_return_validation_native_'),('regression05','wl2_r7_recovery_origin_'),('regression06','infrastructure::windows_client_originals024::adapter::tests::wl2_r6_typed_adapter_retained_v3_slot_and_private_observation_types_are_send')]"+text[end:]
text=text.replace("if name=='regression04':","if name=='regression02':").replace("if name=='regression09':args.insert(1,'--exact')","if name=='regression06':args.insert(1,'--exact')").replace('affected original returned-slot/worker/release/pipeline/origin/strictV1/Send/QPC only; noW2 or official run','affected originalworker/pipeline/returnedvalidation/origin/Send after canonicaltrait only; noW2 or officialrun')
(ROOT/'run-regressions.py').write_text(text,encoding='utf-8',newline='\n')
print('prepared affected original worker/pipeline/returnedvalidation/origin/Send regressions')
