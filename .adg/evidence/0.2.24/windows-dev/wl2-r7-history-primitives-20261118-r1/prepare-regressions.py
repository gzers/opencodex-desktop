from pathlib import Path
ROOT=Path(__file__).resolve().parent
text=(ROOT.parent/'wl2-r7-return-validation-20261117-r1/run-regressions.py').read_text(encoding='utf-8')
start=text.index('runs=[');stop=text.index('\nfor name,case in runs:',start)
text=text[:start]+"runs=[('native05','wl2_r7_history_primitives_native_foreign_actual_job_'),('regression06','wl2_r7_attachment_storage_native_'),('regression07','wl2_r7_v2_storage_native_'),('regression08','wl2_r7_quiesce_join_native_'),('regression09','wl2_r7_concrete_driver_native_')]"+text[stop:]
text=text.replace("if name=='regression04':","if False:").replace("if name=='regression07':args.insert(1,'--exact')","# All filters are explicit affected native test prefixes, not W2")
text=text.replace("    if False:\n        child=root/'subprocess';child.mkdir();env['OCX024_WORKER_TEST_EVIDENCE']=str(child)\n","").replace('affected original native worker/release/concretepipeline/Send only; noW2 or official run','affected native Job history, original V1/V2 storage, sameclaim quiescence and native pipeline; noW2 or official run')
(ROOT/'run-regressions.py').write_text(text,encoding='utf-8',newline='\n')
print('prepared one unrun affected nativeJob test and old11storage/1quiesce/1pipeline regressions; no duplicate passed DATA histories')
