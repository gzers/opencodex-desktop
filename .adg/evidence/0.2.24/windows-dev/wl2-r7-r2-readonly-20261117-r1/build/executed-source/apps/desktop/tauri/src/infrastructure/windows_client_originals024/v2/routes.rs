//! Original H/config bytes, native generation and complete independent routes.
//! No JSON constructor, Clone or lifecycle permission. Current session only.
use super::super::{
    bounded::{native_stamp, BoundedAccessV1},
    tree,
};
use crate::infrastructure::windows_official_runtime024::{validate_local_path, PinnedDirectory};
use crate::modules::runtime::client_route_declaration::declaration;
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    os::windows::fs::OpenOptionsExt,
    path::Path,
    time::Instant,
};
const MAX: u64 = 2 * 1024 * 1024;
mod close;
pub(in crate::infrastructure::windows_client_originals024) use close::RetainedRouteClose;
fn refused() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "original native config/complete route declaration unconfirmed",
    )
}
fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
pub(in crate::infrastructure::windows_client_originals024) struct RoutePin {
    home: PinnedDirectory,
    file: File,
    security: BoundedAccessV1,
    raw: Vec<u8>,
    routes: Vec<String>,
}
impl RoutePin {
    fn read(file: &mut File, end: Instant) -> io::Result<Vec<u8>> {
        before(end)?;
        let len = file.metadata()?.len();
        if len == 0 || len > MAX {
            return Err(refused());
        }
        file.seek(SeekFrom::Start(0))?;
        let mut raw = Vec::new();
        file.take(MAX + 1).read_to_end(&mut raw)?;
        if raw.len() as u64 != len {
            return Err(refused());
        }
        before(end)?;
        Ok(raw)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn capture(
        home: &Path,
        end: Instant,
    ) -> io::Result<Self> {
        before(end)?;
        let home = PinnedDirectory::open(home)?;
        let path = home.path().join("config.json");
        validate_local_path(&path)?;
        let security = BoundedAccessV1::capture(&path, false, end).map_err(io::Error::other)?;
        let mut file = OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(&path)?;
        tree::streams(&path, false)?;
        if native_stamp(&file, end).map_err(io::Error::other)? != security.snapshot_data()? {
            return Err(refused());
        }
        let raw = Self::read(&mut file, end)?;
        // Preserve strict parser Err/None as refusals. Do not log config values
        // or derive a complete declaration from generated catalog contents.
        let routes = declaration(&raw)
            .map_err(|_| refused())?
            .ok_or_else(refused)?;
        let mut out = Self {
            home,
            file,
            security,
            raw,
            routes,
        };
        out.check(end)?;
        Ok(out)
    }
    pub(super) fn check(&mut self, end: Instant) -> io::Result<()> {
        before(end)?;
        self.home.recheck()?;
        self.security.recheck(end).map_err(io::Error::other)?;
        let path = self.home.path().join("config.json");
        let current = OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(&path)?;
        tree::streams(&path, false)?;
        let expected = self.security.snapshot_data()?;
        if native_stamp(&current, end).map_err(io::Error::other)? != expected
            || native_stamp(&self.file, end).map_err(io::Error::other)? != expected
            || Self::read(&mut self.file, end)? != self.raw
        {
            return Err(refused());
        }
        self.home.recheck()?;
        before(end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn declared<'a>(
        &'a mut self,
        home: &Path,
        end: Instant,
    ) -> io::Result<&'a [String]> {
        if self.home.path() != home {
            return Err(refused());
        }
        self.check(end)?;
        Ok(&self.routes)
    }
}
#[cfg(test)]
mod tests {
    use super::super::super::tree::tests::Fixture;
    use super::*;
    use std::time::Duration;
    fn end() -> Instant {
        Instant::now() + Duration::from_secs(10)
    }
    #[test]
    fn wl2_r5_native_config_declares_routes_and_excludes_write_replace_wrong_home() {
        let f = Fixture::new();
        let h = f.path().join("H");
        std::fs::create_dir(&h).unwrap();
        let path = h.join("config.json");
        std::fs::write(
            &path,
            br#"{"providers":{"P":{"liveModels":false,"selectedModels":["native/id"]}}}"#,
        )
        .unwrap();
        let mut pin = RoutePin::capture(&h, end()).unwrap();
        assert_eq!(pin.declared(&h, end()).unwrap(), ["P/native-id"]);
        assert!(std::fs::write(&path, b"{}").is_err());
        assert!(std::fs::rename(&path, h.join("replaced.json")).is_err());
        assert!(pin.declared(&f.path().join("otherH"), end()).is_err());
        assert!(pin.declared(&h, Instant::now()).is_err());
        pin.check(end()).unwrap();
        println!("native config ID/8 projections/read bytes fixed; exact independent route, write/replace/wrong-H/expired refusal; no official or Closed");
    }
    #[test]
    fn wl2_r5_native_config_unknown_duplicate_missing_and_oversize_refuse() {
        let f = Fixture::new();
        let h = f.path().join("H");
        std::fs::create_dir(&h).unwrap();
        let path = h.join("config.json");
        assert!(RoutePin::capture(&h, end()).is_err());
        for raw in [
            br#"{}"#.as_slice(),
            br#"{"providers":{},"providers":{}}"#,
            br#"{"providers":{"P":{"liveModels":true,"selectedModels":[]}}}"#,
        ] {
            std::fs::write(&path, raw).unwrap();
            assert!(RoutePin::capture(&h, end()).is_err());
        }
        std::fs::write(&path, vec![b' '; MAX as usize + 1]).unwrap();
        assert!(RoutePin::capture(&h, end()).is_err());
        std::fs::write(&path, br#"{"providers":{}}"#).unwrap();
        let mut pin = RoutePin::capture(&h, end()).unwrap();
        assert!(pin.declared(&h, end()).unwrap().is_empty());
        println!("native missing/strict duplicate/dynamic unknown/2MiB overflow refuse; independently explicit empty is distinct");
    }
    #[test]
    fn wl2_r5_native_config_hardlink_and_ads_refuse() {
        for ads in [false, true] {
            let f = Fixture::new();
            let h = f.path().join("H");
            std::fs::create_dir(&h).unwrap();
            let path = h.join("config.json");
            std::fs::write(&path, br#"{"providers":{}}"#).unwrap();
            if ads {
                std::fs::write(h.join("config.json:foreign"), b"extra").unwrap();
            } else {
                std::fs::hard_link(&path, h.join("alias.json")).unwrap();
            }
            assert!(RoutePin::capture(&h, end()).is_err());
        }
        println!(
            "native config multiple links and alternate stream refuse; successful rejection fixtures use scoped cleanup"
        );
    }
}
