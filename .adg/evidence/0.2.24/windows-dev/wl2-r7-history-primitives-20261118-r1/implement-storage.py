from pathlib import Path
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code/apps/desktop/tauri/src/infrastructure/windows_client_originals024')
for version in ['before_resume_record','before_resume_record_v2']:
    p=CODE/version/'storage.rs';text=p.read_text(encoding='utf-8')
    assert 'last_complete_at' not in text
    text=text.replace('    end: Option<Instant>,','    end: Option<Instant>,\n    last_complete_at: Option<Instant>,',1)
    start='''    fn read_inner(&mut self, end: Instant) -> io::Result<Vec<Vec<u8>>> {
        self.cutoff(end)?;
'''
    assert start in text
    text=text.replace(start,'''    fn read_inner(&mut self, end: Instant) -> io::Result<Vec<Vec<u8>>> {
        self.cutoff(end)?;
        self.read_retained_files(end)
    }
    // Original native files only. Every operation still bounded by its caller
    // read end; this private helper has no write or deadline-reset authority.
    fn read_retained_files(&mut self, end: Instant) -> io::Result<Vec<Vec<u8>>> {
        if Instant::now() >= end { return Err(io::ErrorKind::TimedOut.into()); }
''',1)
    boundary='''        self.cutoff(end)?;
        Ok(out)
    }
'''
    assert boundary in text
    text=text.replace(boundary,'''        if Instant::now() >= end { return Err(io::ErrorKind::TimedOut.into()); }
        Ok(out)
    }
    /// DATA-only historical read of the same successfully completed native
    /// prefix. No existing directory adoption or renewed append permission.
    pub(in crate::infrastructure::windows_client_originals024) fn read_history_data(
        &mut self, read_end: Instant,
    ) -> io::Result<Vec<Vec<u8>>> {
        let result = (|| {
            let original_end = self.end.ok_or_else(invalid)?;
            let completed = self.last_complete_at.ok_or_else(invalid)?;
            if completed >= original_end || completed > Instant::now() || self.files.is_empty() {
                return Err(invalid());
            }
            self.read_retained_files(read_end)
        })();
        if result.is_err() { self.failed = true; }
        result
    }
''',1)
    tail='''            self.checkpoint(5, end)?;
            self.read_inner(end).map(|_| ())
'''
    assert tail in text
    text=text.replace(tail,'''            self.checkpoint(5, end)?;
            self.read_inner(end)?;
            let completed = Instant::now();
            self.cutoff(end)?;
            self.last_complete_at = Some(completed);
            Ok(())
''',1)
    p.write_text(text,encoding='utf-8',newline='\n')
    tests=CODE/version/'storage/tests.rs'
    text=tests.read_text(encoding='utf-8')
    assert 'mod history_tests;' not in text
    tests.write_text(text+'\nmod history_tests;\n',encoding='utf-8',newline='\n')
    new=CODE/version/'storage/tests/history_tests.rs';new.parent.mkdir(parents=True,exist_ok=True)
    source='''use super::*;
#[test]
fn wl2_r7_history_primitives_native_ATTACHMENT_same_files_after_write_cutoff_no_new_permission() {
    for mode in ["valid", "member", "file-security", "expired-read", "pending-write", "missing-completion"] {
        let scene = Scene::new();
        let original_end = Instant::now()+Duration::from_millis(600);
        let mut storage = scene.initialized(original_end);
        let raw = encoded_data(&record());
        if mode == "pending-write" {
            storage.fault = 3;
            assert!(storage.append_data(&raw, original_end).is_err());
        } else if mode != "missing-completion" {
            storage.append_data(&raw, original_end).unwrap();
        }
        let identities = storage.files.iter().map(|r| r.id).collect::<Vec<_>>();
        while Instant::now()<original_end { std::thread::sleep(Duration::from_millis(1)); }
        match mode {
            "member" => std::fs::write(scene.path().join("records/foreign-after-cutoff.json"),b"preserve").unwrap(),
            "file-security" => change_dacl(&storage.files[0].path),
            _ => {}
        }
        let read_end=if mode=="expired-read" {Instant::now()} else {Instant::now()+Duration::from_secs(5)};
        let result = storage.read_history_data(read_end);
        if mode=="valid" {
            assert_eq!(result.unwrap(),vec![raw.clone()]);
            assert_eq!(storage.read_history_data(read_end).unwrap(),vec![raw.clone()]);
            assert!(!storage.failed);
        } else { assert!(result.is_err(),"{mode}");assert!(storage.failed); }
        assert_eq!(storage.end,Some(original_end));
        assert_eq!(storage.files.iter().map(|r|r.id).collect::<Vec<_>>(),identities);
        assert!(storage.append_data(&raw,original_end).is_err());
        assert!(storage.append_data(&raw,read_end).is_err());
        assert!(!scene.path().join("records/RECORD000002.json").exists());
        println!("r7 historical ATTACHMENT native {mode}: same original files/native prefix, original write cutoff expired/no newappend or rewindow; DATA only");
        drop(storage);drop(scene);
    }
}
'''
    source=source.replace('ATTACHMENT','v2attachment' if version.endswith('v2') else 'v1attachment').replace('RECORD','v2-record-' if version.endswith('v2') else 'record-')
    new.write_text(source,encoding='utf-8',newline='\n')
print('separate original append cutoff and retained DATA-only historical file read; V1/V2 remain independent')
