//! Exact owned file close inventory. No JSON/path/handle-number constructor.
//! Unknown close results are never retried or reconstructed into File objects.
use super::phase::invalid;
use crate::infrastructure::windows_official_runtime024::{self as native, FileIdentity};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io,
    os::windows::io::{AsRawHandle, IntoRawHandle},
    time::Instant,
};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct HandleRow {
    pub(super) key: String,
    pub(super) volume: u32,
    pub(super) file_id: String,
    pub(super) directory: bool,
    pub(super) historical_work: bool,
}
enum OwnedState {
    Held(File),
    ConfirmedClosed,
    Unknown {
        raw_value: usize,
        raw_error: Option<i32>,
    },
}
struct Item {
    row: HandleRow,
    state: OwnedState,
}
pub(super) struct NativeHandleBatch {
    items: Vec<Item>,
    poisoned: bool,
    started: bool,
    completed_at: Option<Instant>,
    deadline: Instant,
}
/// Constructed only by observing actual retained Files/pins before transfer.
/// No serde/Clone, parsed-row constructor, handle reopening or native permit.
pub(super) struct ExpectedHandleInventory {
    rows: Vec<HandleRow>,
    deadline: Instant,
}
impl ExpectedHandleInventory {
    pub(super) fn new(end: Instant) -> Self {
        Self {
            rows: Vec::new(),
            deadline: end,
        }
    }
    fn push(
        &mut self,
        key: String,
        id: FileIdentity,
        directory: bool,
        historical_work: bool,
        end: Instant,
    ) -> io::Result<()> {
        before(end)?;
        if end != self.deadline || self.rows.len() >= 4096 || self.rows.iter().any(|r| r.key == key)
        {
            return Err(invalid());
        }
        self.rows.push(HandleRow {
            key,
            volume: id.volume,
            file_id: id.index.to_string(),
            directory,
            historical_work,
        });
        Ok(())
    }
    pub(super) fn observe(
        &mut self,
        key: String,
        file: &File,
        expected: FileIdentity,
        directory: bool,
        historical_work: bool,
        end: Instant,
    ) -> io::Result<()> {
        before(end)?;
        if identity(file, directory, historical_work)? != expected {
            return Err(invalid());
        }
        self.push(key, expected, directory, historical_work, end)
    }
    pub(super) fn observe_directory(
        &mut self,
        prefix: &str,
        pin: &native::PinnedDirectory,
        end: Instant,
    ) -> io::Result<()> {
        before(end)?;
        for (i, id) in pin.owned_file_identities()?.into_iter().enumerate() {
            self.push(format!("{prefix}/{i}"), id, true, false, end)?;
        }
        before(end)
    }
    pub(super) fn matches(&self, batch: &NativeHandleBatch, end: Instant) -> io::Result<()> {
        before(end)?;
        if end != self.deadline
            || self.rows.is_empty()
            || batch.poisoned
            || batch.deadline != self.deadline
            || self.rows.len() != batch.items.len()
        {
            return Err(invalid());
        }
        // Exact ordered role/key/native object/kind/history flag for every
        // handle, independently observed BEFORE any object ownership transfer.
        if self
            .rows
            .iter()
            .zip(&batch.items)
            .any(|(expected, item)| *expected != item.row)
        {
            return Err(invalid());
        }
        before(end)
    }
}
fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
impl NativeHandleBatch {
    pub(super) fn new(end: Instant) -> Self {
        Self {
            items: Vec::new(),
            poisoned: false,
            started: false,
            completed_at: None,
            deadline: end,
        }
    }
    pub(super) fn retain(
        &mut self,
        key: String,
        file: File,
        id: FileIdentity,
        directory: bool,
        historical_work: bool,
    ) {
        if self.started || self.items.iter().any(|i| i.row.key == key) || self.items.len() >= 4096 {
            self.poisoned = true;
        }
        self.items.push(Item {
            row: HandleRow {
                key,
                volume: id.volume,
                file_id: id.index.to_string(),
                directory,
                historical_work,
            },
            state: OwnedState::Held(file),
        });
    }
    pub(super) fn retain_observed(
        &mut self,
        key: String,
        file: File,
        directory: bool,
        historical_work: bool,
    ) {
        let id = identity(&file, directory, historical_work);
        self.retain(
            key,
            file,
            id.as_ref().copied().unwrap_or(FileIdentity {
                volume: 0,
                index: 0,
            }),
            directory,
            historical_work,
        );
        if id.is_err() {
            self.poisoned = true;
        }
    }
    pub(super) fn invalidate(&mut self) {
        self.poisoned = true;
    }
    pub(super) fn inventory(&self) -> io::Result<serde_json::Value> {
        if self.poisoned || self.items.is_empty() {
            return Err(invalid());
        }
        Ok(
            serde_json::json!({"rows":self.items.iter().map(|i|&i.row).collect::<Vec<_>>(),"nativeAuthority":false}),
        )
    }
    pub(super) fn check_held(&self, end: Instant) -> io::Result<()> {
        before(end)?;
        if self.poisoned || self.started || end > self.deadline {
            return Err(invalid());
        }
        let token = native::current_identity()?;
        if token.elevated || token.elevation_type == 2 {
            return Err(invalid());
        }
        for item in &self.items {
            let OwnedState::Held(file) = &item.state else {
                return Err(invalid());
            };
            let observed = identity(file, item.row.directory, item.row.historical_work)?;
            if observed.volume != item.row.volume || observed.index.to_string() != item.row.file_id
            {
                return Err(invalid());
            }
            before(end)?;
        }
        Ok(())
    }
    pub(super) fn close_all(&mut self, fault: u8, end: Instant) -> io::Result<()> {
        let result = (|| {
            self.check_held(end)?;
            self.started = true;
            for (index, item) in self.items.iter_mut().enumerate() {
                before(end)?;
                let state = std::mem::replace(
                    &mut item.state,
                    OwnedState::Unknown {
                        raw_value: 0,
                        raw_error: None,
                    },
                );
                let OwnedState::Held(file) = state else {
                    self.poisoned = true;
                    return Err(invalid());
                };
                let raw = file.into_raw_handle();
                // Install uncertainty BEFORE invoking the native close. On
                // failure, never rebuild File or retry a potentially reused value.
                item.state = OwnedState::Unknown {
                    raw_value: raw as usize,
                    raw_error: None,
                };
                let ok = unsafe { windows_sys::Win32::Foundation::CloseHandle(raw) };
                if ok == 0 {
                    let error = io::Error::last_os_error();
                    item.state = OwnedState::Unknown {
                        raw_value: raw as usize,
                        raw_error: error.raw_os_error(),
                    };
                    self.poisoned = true;
                    return Err(error);
                }
                item.state = OwnedState::ConfirmedClosed;
                #[cfg(test)]
                if fault == 81 && index == 0 {
                    return Err(io::Error::other(
                        "injected after first confirmed native original-handle close",
                    ));
                }
            }
            before(end)?;
            self.completed_at = Some(Instant::now());
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(super) fn completed(&self) -> bool {
        !self.poisoned
            && self.completed_at.is_some_and(|t| t < self.deadline)
            && !self.items.is_empty()
            && self
                .items
                .iter()
                .all(|i| matches!(i.state, OwnedState::ConfirmedClosed))
    }
    pub(super) fn trace(&self) -> serde_json::Value {
        serde_json::json!({"started":self.started,"poisoned":self.poisoned,"complete":self.completed(),"rows":self.items.iter().map(|i|{
            let (state,error)=match &i.state {OwnedState::Held(_)=>("Held",None),OwnedState::ConfirmedClosed=>("ConfirmedClosed",None),
                OwnedState::Unknown{raw_value,raw_error}=>{let _=raw_value;("Unknown",*raw_error)}};
            serde_json::json!({"row":i.row,"state":state,"rawError":error})
        }).collect::<Vec<_>>(),"nativeAuthority":false})
    }
    #[cfg(test)]
    pub(super) fn protect_first_for_test(&self) -> io::Result<()> {
        let Some(Item {
            state: OwnedState::Held(file),
            ..
        }) = self.items.first()
        else {
            return Err(invalid());
        };
        if unsafe {
            windows_sys::Win32::Foundation::SetHandleInformation(file.as_raw_handle(), 2, 2)
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}
fn identity(file: &File, directory: bool, historical_work: bool) -> io::Result<FileIdentity> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, GetFileType, BY_HANDLE_FILE_INFORMATION, FILE_TYPE_DISK,
    };
    let mut data: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut data) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || data.dwFileAttributes & 0x400 != 0
        || (data.dwFileAttributes & 0x10 != 0) != directory
        || !directory && (data.nNumberOfLinks > 1 || !historical_work && data.nNumberOfLinks != 1)
    {
        return Err(invalid());
    }
    Ok(FileIdentity {
        volume: data.dwVolumeSerialNumber,
        index: ((data.nFileIndexHigh as u64) << 32) | data.nFileIndexLow as u64,
    })
}
#[cfg(test)]
mod tests {
    use super::super::tree::tests::Fixture;
    use super::*;
    use native::{create_private_directory, PrivateRecord};
    #[test]
    fn wl2_r6_inventory_native_exact_before_transfer_rejects_missing_extra_role_or_other_object_without_close(
    ) {
        for case in 0..5 {
            let mut fixture = Fixture::new();
            let directory = create_private_directory(&fixture.path().join("inventory")).unwrap();
            let path = directory.path().join("original.bin");
            let other = directory.path().join("other.bin");
            drop(PrivateRecord::create(&path, b"synthetic original exact handle", 1024).unwrap());
            drop(PrivateRecord::create(&other, b"synthetic distinct native object", 1024).unwrap());
            let file = File::open(&path).unwrap();
            let original = identity(&file, false, false).unwrap();
            let end = Instant::now() + std::time::Duration::from_secs(10);
            let mut expected = ExpectedHandleInventory::new(end);
            expected
                .observe(
                    "tree/0/original/0000".into(),
                    &file,
                    original,
                    false,
                    false,
                    end,
                )
                .unwrap();
            let mut batch = NativeHandleBatch::new(end);
            if case != 1 {
                let actual = if case == 4 {
                    File::open(&other).unwrap()
                } else {
                    file.try_clone().unwrap()
                };
                let id = identity(&actual, false, false).unwrap();
                batch.retain(
                    if case == 3 {
                        "tree/0/initial-work/0000".into()
                    } else {
                        "tree/0/original/0000".into()
                    },
                    actual,
                    id,
                    false,
                    false,
                );
                if case == 2 {
                    batch.retain(
                        "tree/0/old-work/0000/0000".into(),
                        file.try_clone().unwrap(),
                        original,
                        false,
                        true,
                    );
                }
            }
            if case == 0 {
                batch.check_held(end).unwrap();
                expected.matches(&batch, end).unwrap();
                assert!(expected
                    .matches(&batch, end + std::time::Duration::from_nanos(1))
                    .is_err());
                assert!(!batch.completed());
            } else {
                assert!(expected.matches(&batch, end).is_err());
                assert!(!batch.started && batch.completed_at.is_none());
                assert!(batch
                    .items
                    .iter()
                    .all(|item| matches!(item.state, OwnedState::Held(_))));
            }
            assert_eq!(identity(&file, false, false).unwrap(), original);
            assert_eq!(
                std::fs::read(&path).unwrap(),
                b"synthetic original exact handle"
            );
            println!("r6 inventory-native-before-transfer case={case}: complete independent expected rows, missing/extra/role/actualID substitutions refuse before native close; original objects/bytes retained, no parsed authority");
            fixture.retain();
        }
    }
    #[test]
    fn wl2_r6_inventory_native_original_ancestor_pin_set_is_observed_without_transfer() {
        let mut fixture = Fixture::new();
        let directory =
            create_private_directory(&fixture.path().join("inventory-ancestors")).unwrap();
        let end = Instant::now() + std::time::Duration::from_secs(10);
        let mut expected = ExpectedHandleInventory::new(end);
        let before = directory.owned_file_identities().unwrap();
        expected
            .observe_directory("lease/final/bound/profile", &directory, end)
            .unwrap();
        assert_eq!(expected.rows.len(), directory.path().ancestors().count());
        assert_eq!(before, directory.owned_file_identities().unwrap());
        assert_eq!(before.last(), Some(&directory.identity()));
        for (i, row) in expected.rows.iter().enumerate() {
            assert_eq!(row.key, format!("lease/final/bound/profile/{i}"));
            assert!(row.directory && !row.historical_work);
            assert_eq!(row.volume, before[i].volume);
            assert_eq!(row.file_id, before[i].index.to_string());
        }
        directory.recheck().unwrap();
        println!("r6 native expected ancestor inventory retains every original drive-to-leaf handle ordinal/actualID without reopening/cloning/transfer, count={} original pin unchanged",before.len());
        fixture.retain();
    }
    #[test]
    fn wl2_r6_handles_native_exact_own_file_close_and_protected_unknown_without_retry() {
        for protected in [false, true] {
            let mut f = Fixture::new();
            let directory = create_private_directory(&f.path().join("handle-test")).unwrap();
            let path = directory.path().join("original.bin");
            drop(PrivateRecord::create(&path, b"synthetic original handle", 1024).unwrap());
            let file = File::open(&path).unwrap();
            let id = identity(&file, false, false).unwrap();
            let end = Instant::now() + std::time::Duration::from_secs(10);
            let mut batch = NativeHandleBatch::new(end);
            batch.retain("tree/0/original/0000".into(), file, id, false, false);
            batch.check_held(end).unwrap();
            if protected {
                batch.protect_first_for_test().unwrap();
                assert!(batch.close_all(0, end).is_err());
                let trace = batch.trace();
                assert_eq!(trace["rows"][0]["state"], "Unknown");
                assert!(trace["rows"][0]["rawError"].is_number());
                assert!(batch.close_all(0, end).is_err());
                assert_eq!(batch.trace(), trace, "no retry/clear flag/reconstruct File");
            } else {
                batch.close_all(0, end).unwrap();
                assert!(batch.completed());
                assert_eq!(batch.trace()["rows"][0]["state"], "ConfirmedClosed");
            }
            assert_eq!(std::fs::read(&path).unwrap(), b"synthetic original handle");
            println!(
                "r6-handles-native-primitive={}",
                serde_json::json!({"protected":protected,"batch":batch.trace(),"originalFileRetained":true})
            );
            f.retain();
        }
    }
}
