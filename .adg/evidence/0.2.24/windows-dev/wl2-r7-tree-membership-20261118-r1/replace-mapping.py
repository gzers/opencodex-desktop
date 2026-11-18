from pathlib import Path
p=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code/apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/before_resume.rs')
text=p.read_text(encoding='utf-8');start=text.index('    pub(in crate::infrastructure::windows_client_originals024) fn v2_retained_object_data(');end=text.index('    pub(in crate::infrastructure::windows_client_originals024) fn read_before_resume_prefix(',start)
assert 'pending-native-open' in text[start:end]
text=text[:start]+text[end:];p.write_text(text,encoding='utf-8',newline=chr(10))
print('removed obsolete original-only compact ordinal projection; new complete native membership implemented in child module')
