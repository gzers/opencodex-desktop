//! Test-only native pins for the fixed marenben fixture and its launcher.
use super::{
    absent, bootstrap_matches, desktop, invalid, local_account_matches, BASE, BUNDLE, PUBLIC,
};
use crate::infrastructure::windows_client_originals024::coordination;
use crate::infrastructure::windows_official_runtime024 as native;
use crate::infrastructure::windows_owner_host024::empty_host::EmptyBootstrap;
use native::{FileIdentity, PinnedDirectory, RetainedPeer, TokenIdentity};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
};

fn before(end: Option<Instant>) -> io::Result<()> {
    if end.is_some_and(|limit| Instant::now() >= limit) {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}

fn canonical(path: &Path) -> io::Result<()> {
    if dunce::simplified(&fs::canonicalize(path)?) != path {
        return Err(invalid("fixture native canonical path differs"));
    }
    Ok(())
}

pub(super) fn canonical_pin(path: &Path, private: bool) -> io::Result<PinnedDirectory> {
    let pin = if private {
        PinnedDirectory::open_private(path)?
    } else {
        PinnedDirectory::open(path)?
    };
    canonical(path)?;
    pin.owned_file_identities()?;
    Ok(pin)
}

pub(super) fn fixed_leaf(parent: &PinnedDirectory, leaf: &Path) -> io::Result<()> {
    if leaf.parent() != Some(parent.path()) {
        return Err(invalid("fixture leaf has the wrong pinned parent"));
    }
    native::validate_local_path(leaf)?;
    parent.recheck()?;
    absent(leaf)?;
    parent.recheck()
}

pub(super) fn create(parent: &PinnedDirectory, path: &Path) -> io::Result<PinnedDirectory> {
    fixed_leaf(parent, path)?;
    let pin = native::create_private_directory(path)?;
    canonical(path)?;
    let parents = parent.owned_file_identities()?;
    let lineage = pin.owned_file_identities()?;
    if lineage.len() != parents.len() + 1
        || !lineage.starts_with(&parents)
        || pin.identity().volume != parent.identity().volume
        || parents.contains(&pin.identity())
    {
        return Err(invalid(
            "fixture creation was not under the original parent",
        ));
    }
    Ok(pin)
}

fn overlapping(left: &Path, right: &Path) -> io::Result<bool> {
    native::validate_local_path(left)?;
    native::validate_local_path(right)?;
    let components = |path: &Path| {
        path.components()
            .map(|component| component.as_os_str().to_string_lossy().to_uppercase())
            .collect::<Vec<_>>()
    };
    let left = components(left);
    let right = components(right);
    Ok(left.starts_with(&right) || right.starts_with(&left))
}

pub(super) struct Boundary {
    pub(super) profile: PinnedDirectory,
    local: PinnedDirectory,
    pub(super) excluded: [PathBuf; 4],
}
impl Boundary {
    pub(super) fn admit(config: &EmptyBootstrap) -> io::Result<(RetainedPeer, Self)> {
        let peer = RetainedPeer::current()?;
        let profile = crate::infrastructure::windows_startup_root::current_user_profile()?;
        if !bootstrap_matches(config, &peer.identity().token, &profile) {
            return Err(invalid("isolated r7 native bootstrap refused"));
        }
        local_account_matches(&peer.identity().token)?;
        desktop()?;
        peer.no_outer_job()?;
        let profile = canonical_pin(&profile, false)?;
        let local = canonical_pin(&profile.path().join("AppData").join("Local"), false)?;
        let official = coordination::official_runtime_root(&peer.identity().token)?;
        if official
            != local
                .path()
                .join("OpenCodex/Runtime/v1")
                .join(&peer.identity().token.sid)
        {
            return Err(invalid(
                "default Known Folder locator could alias the fixture",
            ));
        }
        let excluded = [
            profile.path().join(".codex"),
            profile.path().join(".grok"),
            local.path().join("OpenCodex"),
            profile.path().join("ocx024-r6-existing-20261117-r2"),
        ];
        let scope = Self {
            profile,
            local,
            excluded,
        };
        scope.recheck()?;
        Ok((peer, scope))
    }

    pub(super) fn recheck(&self) -> io::Result<()> {
        self.profile.owned_file_identities()?;
        self.local.owned_file_identities()?;
        let base = Path::new(BASE);
        if base.parent() != Some(self.profile.path()) {
            return Err(invalid("fixture base escaped the original Profile"));
        }
        for target in &self.excluded {
            if overlapping(base, target)? || overlapping(Path::new(PUBLIC), target)? {
                return Err(invalid("fixture overlaps a protected client or r2 scope"));
            }
        }
        Ok(())
    }
}

pub(super) fn open_file(path: &Path, share: u32, private: bool) -> io::Result<File> {
    native::validate_local_path(path)?;
    let file = OpenOptions::new()
        .access_mode(0x80020000)
        .share_mode(share)
        .custom_flags(0x200000)
        .open(path)?;
    coordination::file_id(&file)?;
    if private {
        native::verify_private_file(&file)?;
    }
    canonical(path)?;
    Ok(file)
}

pub(super) fn checked_image(
    path: &Path,
    digest: &str,
    end: Option<Instant>,
    private: bool,
) -> io::Result<File> {
    before(end)?;
    let mut file = open_file(path, 1, private)?;
    let mut hasher = Sha256::new();
    let mut count = 0;
    let mut buffer = [0u8; 65536];
    loop {
        before(end)?;
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        count += read;
        if count > 512 * 1024 * 1024 {
            return Err(invalid("fixture test image exceeds byte bound"));
        }
        hasher.update(&buffer[..read]);
    }
    if format!("{:x}", hasher.finalize()) != digest {
        return Err(invalid("exact test-only same image hash differs"));
    }
    before(end)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(file)
}

pub(super) fn coordination_paths(client: &Path, root: &Path, digest: &str) -> [PathBuf; 4] {
    [
        client.join(".opencodex-native-main.claim.sqlite"),
        root.join(format!("native-{digest}.sqlite")),
        root.join("client-desktop-lifecycle.sqlite"),
        root.join(format!("catalog-{digest}.sqlite")),
    ]
}

enum ClientPins {
    Strict([PinnedDirectory; 2]),
    Preflight([File; 2]),
}

struct ClientRoots {
    pins: ClientPins,
    children: [(PathBuf, FileIdentity); 2],
}

fn require_ntfs(file: &File, expected_volume: u32) -> io::Result<()> {
    use std::ptr::null_mut;
    use windows_sys::Win32::Storage::FileSystem::GetVolumeInformationByHandleW;

    let mut name = [0u16; 16];
    let mut volume = 0;
    if unsafe {
        GetVolumeInformationByHandleW(
            file.as_raw_handle(),
            null_mut(),
            0,
            &mut volume,
            null_mut(),
            null_mut(),
            name.as_mut_ptr(),
            name.len() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if volume != expected_volume || !name[..5].iter().copied().eq("NTFS\0".encode_utf16()) {
        return Err(invalid(
            "fixture preflight requires the original NTFS volume",
        ));
    }
    Ok(())
}

impl ClientRoots {
    fn pins(&self) -> io::Result<&[PinnedDirectory; 2]> {
        match &self.pins {
            ClientPins::Strict(pins) => Ok(pins),
            ClientPins::Preflight(_) => Err(invalid("fixture client pins unavailable")),
        }
    }

    fn preflight(
        &mut self,
        parent: &PinnedDirectory,
        end: Instant,
        read: impl FnOnce(&Path, Instant) -> io::Result<()>,
    ) -> io::Result<()> {
        before(Some(end))?;
        let [codex, grok] = self.pins()?;
        let paths = [codex.path().to_owned(), grok.path().to_owned()];
        let lineages = [
            codex.owned_file_identities()?,
            grok.owned_file_identities()?,
        ];
        let parent_ids = parent.owned_file_identities()?;
        let fence = |index: usize| -> io::Result<File> {
            let (path, expected) = &self.children[index];
            if paths[index].parent() != Some(parent.path())
                || path.parent() != Some(paths[index].as_path())
                || lineages[index].len() != parent_ids.len() + 1
                || !lineages[index].starts_with(&parent_ids)
                || expected.volume != parent.identity().volume
            {
                return Err(invalid("fixture preflight path or lineage differs"));
            }
            let file = open_file(path, 1, true)?;
            if coordination::file_id(&file)? != *expected {
                return Err(invalid("fixture preflight child identity changed"));
            }
            require_ntfs(&file, expected.volume)?;
            Ok(file)
        };
        // On NTFS the no-delete child handles prevent renaming either client
        // root while the full preflight opens those roots with DELETE access.
        // Keep the strict ancestor pin and both exact children until the strict
        // leaf pins have been restored and checked. On failure keep the child
        // handles and refuse any further actions on this test descriptor.
        let guards = [fence(0)?, fence(1)?];
        before(Some(end))?;
        self.pins = ClientPins::Preflight(guards);
        read(parent.path(), end)?;
        before(Some(end))?;
        let pins = [
            canonical_pin(&paths[0], true)?,
            canonical_pin(&paths[1], true)?,
        ];
        let ClientPins::Preflight(guards) = &self.pins else {
            return Err(invalid("fixture preflight child pins unavailable"));
        };
        for ((pin, expected), (guard, (child, child_id))) in pins
            .iter()
            .zip(&lineages)
            .zip(guards.iter().zip(&self.children))
        {
            if pin.owned_file_identities()? != *expected
                || coordination::file_id(guard)? != *child_id
                || coordination::file_id(&open_file(child, 1, true)?)? != *child_id
            {
                return Err(invalid("fixture preflight object changed"));
            }
            native::verify_private_file(guard)?;
        }
        parent.recheck()?;
        before(Some(end))?;
        self.pins = ClientPins::Strict(pins);
        Ok(())
    }
}

pub(crate) struct Isolation {
    token: TokenIdentity,
    profile: PinnedDirectory,
    installation: PinnedDirectory,
    data: PinnedDirectory,
    home: PinnedDirectory,
    client_parent: PinnedDirectory,
    clients: Mutex<ClientRoots>,
    coordination: PinnedDirectory,
    lock_paths: [PathBuf; 4],
    lock_files: [(File, FileIdentity); 4],
    host: RetainedPeer,
    image: File,
    image_path: PathBuf,
}

impl Isolation {
    pub(super) fn bind(
        config: &EmptyBootstrap,
        clients: [PinnedDirectory; 4],
        lock_paths: [PathBuf; 4],
        grok_config_id: FileIdentity,
    ) -> io::Result<Self> {
        let host = RetainedPeer::current()?;
        let token = host.identity().token.clone();
        if !bootstrap_matches(config, &token, &config.profile) {
            return Err(invalid("fixture descriptor has wrong token or paths"));
        }
        let [parent, codex, grok, coordination] = &clients;
        let base = Path::new(BASE);
        let digest = super::hash::sha256_hex(
            codex
                .path()
                .to_str()
                .ok_or_else(|| invalid("fixture path UTF-8"))?
                .as_bytes(),
        );
        if parent.path() != base.join("clients")
            || codex.path() != parent.path().join(".codex")
            || grok.path() != parent.path().join(".grok")
            || coordination.path() != base.join("coordination")
            || lock_paths != coordination_paths(codex.path(), coordination.path(), &digest)
        {
            return Err(invalid("fixture descriptor escaped its private roots"));
        }
        let profile = canonical_pin(&config.profile, false)?;
        let installation = canonical_pin(&config.installation, true)?;
        let data = canonical_pin(&config.data, true)?;
        let home = canonical_pin(&config.home, true)?;
        let lock_files: [(File, FileIdentity); 4] = lock_paths
            .iter()
            .map(|path| {
                let file = open_file(path, 3, true)?;
                let id = coordination::file_id(&file)?;
                Ok((file, id))
            })
            .collect::<io::Result<Vec<_>>>()?
            .try_into()
            .map_err(|_| invalid("fixture lock count differs"))?;
        let image_path = installation.path().join("ocxd.exe");
        let image = checked_image(&image_path, &config.broker_sha256, None, true)?;
        host.verify_image(&image_path)?;
        let [client_parent, codex, grok, coordination] = clients;
        let children = [
            (lock_paths[0].clone(), lock_files[0].1),
            (grok.path().join("config.json"), grok_config_id),
        ];
        let isolation = Self {
            token,
            profile,
            installation,
            data,
            home,
            client_parent,
            clients: Mutex::new(ClientRoots {
                pins: ClientPins::Strict([codex, grok]),
                children,
            }),
            coordination,
            lock_paths,
            lock_files,
            host,
            image,
            image_path,
        };
        isolation.check_objects()?;
        Ok(isolation)
    }

    pub(crate) fn preflight(&self, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        self.clients()?
            .preflight(&self.client_parent, end, super::super::preflight_bounded)
    }

    fn clients(&self) -> io::Result<std::sync::MutexGuard<'_, ClientRoots>> {
        self.clients
            .try_lock()
            .map_err(|_| invalid("fixture client pins busy or poisoned"))
    }

    pub(crate) fn pin_client_parent(&self) -> io::Result<PinnedDirectory> {
        repeat_pin(&self.client_parent)
    }

    pub(crate) fn pin_client(&self) -> io::Result<PinnedDirectory> {
        repeat_pin(&self.clients()?.pins()?[0])
    }

    pub(crate) fn lock_paths(&self) -> &[PathBuf; 4] {
        &self.lock_paths
    }

    pub(crate) fn verify_owner(
        &self,
        owner: &crate::infrastructure::windows_official_lifecycle024::Owner,
        end: Instant,
    ) -> io::Result<()> {
        if owner.fixture_creator_token() != &self.token {
            return Err(invalid(
                "fixture descriptor does not match the original Owner",
            ));
        }
        self.verify_paths(
            owner.profile_path(),
            owner.installation_path(),
            owner.data_path(),
            owner.home_path(),
            end,
        )
    }

    pub(crate) fn verify_paths(
        &self,
        profile: &Path,
        installation: &Path,
        data: &Path,
        home: &Path,
        end: Instant,
    ) -> io::Result<()> {
        if profile != self.profile.path()
            || installation != self.installation.path()
            || data != self.data.path()
            || home != self.home.path()
        {
            return Err(invalid(
                "fixture descriptor does not match the original paths",
            ));
        }
        self.recheck(end)
    }

    pub(crate) fn recheck(&self, end: Instant) -> io::Result<()> {
        before(Some(end))?;
        self.check_objects()?;
        before(Some(end))
    }

    fn check_objects(&self) -> io::Result<()> {
        if native::current_identity()? != self.token {
            return Err(invalid("fixture token changed"));
        }
        self.host.no_outer_job()?;
        self.host.verify_image(&self.image_path)?;
        let mut ids = Vec::new();
        let mut unique = |id: FileIdentity| -> io::Result<()> {
            if id.volume != self.profile.identity().volume || ids.contains(&id) {
                return Err(invalid(
                    "fixture object aliases or crosses the original volume",
                ));
            }
            ids.push(id);
            Ok(())
        };
        let clients = self.clients()?;
        for pin in [
            &self.profile,
            &self.installation,
            &self.data,
            &self.home,
            &self.client_parent,
            &self.coordination,
        ]
        .into_iter()
        .chain(clients.pins()?)
        {
            pin.owned_file_identities()?;
            unique(pin.identity())?;
        }
        let image_id = coordination::file_id(&self.image)?;
        if image_id != self.host.identity().executable_id {
            return Err(invalid("original host image object changed"));
        }
        native::verify_private_file(&self.image)?;
        unique(image_id)?;
        for (path, (file, expected)) in self.lock_paths.iter().zip(&self.lock_files) {
            if coordination::file_id(file)? != *expected
                || coordination::file_id(&open_file(path, 3, true)?)? != *expected
            {
                return Err(invalid("fixture lock identity changed"));
            }
            unique(*expected)?;
        }
        Ok(())
    }
}

fn repeat_pin(original: &PinnedDirectory) -> io::Result<PinnedDirectory> {
    original.recheck()?;
    let pin = canonical_pin(original.path(), true)?;
    if pin.owned_file_identities()? != original.owned_file_identities()? {
        return Err(invalid("fixture directory lineage changed"));
    }
    Ok(pin)
}

fn test_client_roots() -> (tempfile::TempDir, PinnedDirectory, ClientRoots) {
    let temp = tempfile::tempdir().unwrap();
    let real = fs::canonicalize(temp.path()).unwrap();
    let base = canonical_pin(dunce::simplified(&real), false).unwrap();
    let parent = create(&base, &base.path().join("clients")).unwrap();
    let codex = create(&parent, &parent.path().join(".codex")).unwrap();
    let grok = create(&parent, &parent.path().join(".grok")).unwrap();
    let child = |path: PathBuf| {
        let file = super::platform::create_new_private_file(&path, 0o600).unwrap();
        let id = coordination::file_id(&file).unwrap();
        (path, id)
    };
    let children = [
        child(codex.path().join(".opencodex-native-main.claim.sqlite")),
        child(grok.path().join("config.json")),
    ];
    let roots = ClientRoots {
        pins: ClientPins::Strict([codex, grok]),
        children,
    };
    (temp, parent, roots)
}

#[test]
fn wl2_r7_isolation_preflight_keeps_client_names_and_restores_strict_pins() {
    use super::super::{coordination::NativeLease, tree::AnchoredTree};
    use std::time::Duration;

    let (temp, parent, mut roots) = test_client_roots();
    let children = roots.children.clone();
    let claim_witness = open_file(&children[0].0, 3, true).unwrap();
    let originals = roots
        .pins()
        .unwrap()
        .each_ref()
        .map(PinnedDirectory::identity);
    let deadline = Instant::now() + Duration::from_secs(30);
    let baseline = super::super::preflight_bounded(parent.path(), deadline).unwrap_err();
    println!(
        "strict pin full preflight raw={:?}",
        baseline.raw_os_error()
    );
    assert_eq!(baseline.raw_os_error(), Some(32));
    roots
        .preflight(&parent, deadline, |path, end| {
            assert_eq!(end, deadline);
            for (name, (child, _)) in [".codex", ".grok"].into_iter().zip(&children) {
                let leaf = path.join(name);
                let moved = path.join("moved");
                let file = OpenOptions::new()
                    .access_mode(0x80030000)
                    .share_mode(1)
                    .custom_flags(0x2200000)
                    .open(&leaf)?;
                let rename = native::rename_original_handle(&file, &moved).unwrap_err();
                println!(
                    "{name} original DELETE handle rename raw={:?}",
                    rename.raw_os_error()
                );
                drop(file);
                let leaf_denied = fs::rename(&leaf, &moved).unwrap_err();
                let child_denied = fs::rename(child, leaf.join("moved")).unwrap_err();
                println!(
                    "{name} leaf rename raw={:?}, child rename raw={:?}",
                    leaf_denied.raw_os_error(),
                    child_denied.raw_os_error()
                );
            }
            let ancestor = fs::rename(path, path.with_extension("moved")).unwrap_err();
            println!("ancestor rename raw={:?}", ancestor.raw_os_error());
            super::super::preflight_bounded(path, end)?;
            println!("fenced full preflight raw=0");
            Ok(())
        })
        .unwrap();
    let pins = roots.pins().unwrap();
    assert_eq!(pins.each_ref().map(PinnedDirectory::identity), originals);
    assert_eq!(
        coordination::file_id(&claim_witness).unwrap(),
        children[0].1
    );
    let restored = super::super::preflight_bounded(parent.path(), deadline).unwrap_err();
    println!("restored strict pins raw={:?}", restored.raw_os_error());
    assert_eq!(restored.raw_os_error(), Some(32));
    fs::write(&children[1].0, b"fixture fence released").unwrap();

    let [codex, grok] = pins.each_ref().map(PinnedDirectory::path);
    let coordination = parent.path().with_file_name("coordination");
    let coordination_pin = native::create_private_directory(&coordination).unwrap();
    let paths = coordination_paths(codex, &coordination, &"a".repeat(64));
    let stable = super::ANCHORED_CODEX_STABLE_FILES
        .iter()
        .skip(1)
        .map(|name| codex.join(name));
    for path in stable.chain(paths.iter().skip(1).cloned()) {
        drop(super::platform::create_new_private_file(&path, 0o600).unwrap());
    }
    let witnesses = paths
        .each_ref()
        .map(|path| open_file(path, 3, true).unwrap());
    let custody = ["custody-codex", "custody-grok"].map(|name| {
        let path = parent.path().join(name);
        drop(native::create_private_directory(&path).unwrap());
        path
    });
    let mut lease = NativeLease::acquire_synthetic(&paths, deadline).unwrap();
    {
        let mut claim = lease.anchored_claim(codex, deadline).unwrap();
        AnchoredTree::capture(codex, &custody[0], &mut claim, codex, deadline).unwrap();
        AnchoredTree::capture(grok, &custody[1], &mut claim, codex, deadline).unwrap();
    }
    println!("restored strict pins with native claim/N/L/K and both anchored captures raw=0");
    drop((lease, witnesses, coordination_pin));
    drop((roots, parent, claim_witness));
    let path = temp.path().to_owned();
    temp.close().unwrap();
    assert!(!path.exists());
    println!("native preflight success fixture absent after cleanup");
}

#[test]
fn wl2_r7_isolation_preflight_refuses_missing_identity_and_does_not_retry() {
    use std::time::Duration;

    let (temp, parent, mut roots) = test_client_roots();
    let expired = roots
        .preflight(&parent, Instant::now(), |_, _| panic!("expired preflight"))
        .unwrap_err();
    println!("expired preflight kind={:?}", expired.kind());
    assert_eq!(expired.kind(), io::ErrorKind::TimedOut);
    let deadline = Instant::now() + Duration::from_secs(30);
    let original_id = roots.children[1].1;
    roots.children[1].1 = roots.children[0].1;
    let mismatch = roots
        .preflight(&parent, deadline, |_, _| {
            panic!("mismatched child accepted")
        })
        .unwrap_err();
    println!("child identity mismatch={mismatch}");
    assert!(mismatch.to_string().contains("child identity changed"));
    assert!(roots.pins().is_ok());
    roots.children[1].1 = original_id;
    let failure = roots
        .preflight(&parent, deadline, |_, _| {
            Err(io::Error::from_raw_os_error(31))
        })
        .unwrap_err();
    println!(
        "original rejected preflight raw={:?}",
        failure.raw_os_error()
    );
    assert_eq!(failure.raw_os_error(), Some(31));
    assert!(roots.pins().is_err());
    let denied = fs::rename(parent.path().join(".grok"), parent.path().join("moved")).unwrap_err();
    println!(
        "leaf still pinned after failure raw={:?}",
        denied.raw_os_error()
    );
    assert!(roots
        .preflight(&parent, deadline, |_, _| panic!(
            "failed descriptor retried"
        ))
        .is_err());
    drop((roots, parent));
    let path = temp.path().to_owned();
    temp.close().unwrap();
    assert!(!path.exists());
    println!("native preflight negative fixture absent after cleanup");
}

#[test]
fn wl2_r7_isolation_rejects_overlap_and_refuses_replacement_while_pinned() {
    let temp = tempfile::tempdir().unwrap();
    let real = fs::canonicalize(temp.path()).unwrap();
    let root = dunce::simplified(&real);
    let parent = canonical_pin(root, false).unwrap();
    let owned = root.join("owned");
    let old = root.join("original");
    let pin = create(&parent, &owned).unwrap();
    let original = pin.identity();
    let sibling = root.join("owned-sibling");
    fs::create_dir(&sibling).unwrap();
    assert!(overlapping(&owned, &root.join("OWNED/target")).unwrap());
    assert!(!overlapping(&owned, &sibling).unwrap());
    assert!(overlapping(&owned, &root.join("owned/../other")).is_err());
    assert!(fixed_leaf(&parent, &owned).is_err());
    assert!(fixed_leaf(&parent, &owned.join("child")).is_err());
    let denied = fs::rename(&owned, &old).unwrap_err();
    println!(
        "r7-fixture-directory-replacement raw={:?}",
        denied.raw_os_error()
    );
    assert_eq!(repeat_pin(&pin).unwrap().identity(), original);
    drop(pin);
    fs::rename(&owned, &old).unwrap();
    fs::rename(&sibling, &owned).unwrap();
    assert_ne!(canonical_pin(&owned, false).unwrap().identity(), original);
}

#[test]
fn wl2_r7_isolation_rejects_native_image_alias_and_digest_drift() {
    let temp = tempfile::tempdir().unwrap();
    let real = fs::canonicalize(temp.path()).unwrap();
    let root = dunce::simplified(&real);
    let image = root.join("fixture.exe");
    let data = b"fixed r7 image fixture";
    fs::write(&image, data).unwrap();
    let digest = super::hash::sha256_hex(data);
    let mut pinned = checked_image(&image, &digest, None, false).unwrap();
    let mut read = Vec::new();
    pinned.read_to_end(&mut read).unwrap();
    assert_eq!(read, data);
    assert!(fs::write(&image, b"unexpected image").is_err());
    drop(pinned);
    assert!(checked_image(&image, &"0".repeat(64), None, false).is_err());
    let alias = root.join("alias.exe");
    fs::hard_link(&image, &alias).unwrap();
    assert!(checked_image(&alias, &digest, None, false).is_err());
    assert!(checked_image(&image, &digest, None, false).is_err());
    fs::remove_file(&alias).unwrap();
    checked_image(&image, &digest, None, false).unwrap();
}

#[test]
fn wl2_r7_isolation_rejects_native_junction_before_creation() {
    let temp = tempfile::tempdir().unwrap();
    let real = fs::canonicalize(temp.path()).unwrap();
    let root = dunce::simplified(&real);
    let outside = root.join("target");
    let link = root.join("link");
    fs::create_dir(&outside).unwrap();
    let powershell = std::env::var_os("SystemRoot").unwrap();
    let output = std::process::Command::new(
        PathBuf::from(powershell).join(r"System32/WindowsPowerShell/v1.0/powershell.exe"),
    )
    .args([
        "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
        "$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:OCX_R7_TEST_LINK -Target $env:OCX_R7_TEST_TARGET | Out-Null",
    ])
    .env("OCX_R7_TEST_LINK", &link)
    .env("OCX_R7_TEST_TARGET", &outside)
    .output()
    .unwrap();
    println!(
        "r7-fixture-junction raw={:?}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success());
    let refused = canonical_pin(&link, false).is_err();
    let nested = PinnedDirectory::open(&link.join("missing")).err();
    fs::remove_dir(&link).unwrap();
    assert!(refused);
    assert_ne!(
        nested
            .expect("junction ancestor must be refused")
            .raw_os_error(),
        Some(2)
    );
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
}

/// Keeps the initial parent, executable and output directory pinned while the
/// launcher opens create-only receipts and starts the actual private image.
pub(crate) struct Stage {
    _boundary: Boundary,
    _pins: Vec<PinnedDirectory>,
    _images: [File; 3],
}
impl Stage {
    pub(crate) fn admit(config: &EmptyBootstrap, host: bool) -> io::Result<Self> {
        let (peer, boundary) = Boundary::admit(config)?;
        let public = Path::new(PUBLIC);
        let input = canonical_pin(&public.join("input"), false)?;
        let out = canonical_pin(&public.join("out01"), false)?;
        let volume = boundary.profile.identity().volume;
        if input.identity().volume != volume
            || out.identity().volume != volume
            || input.identity() == out.identity()
            || overlapping(input.path(), out.path())?
        {
            return Err(invalid("fixture input and evidence output overlap"));
        }
        let source_path = input.path().join("fixture-test.exe");
        peer.verify_image(&source_path)?;
        let mut source = checked_image(&source_path, &config.broker_sha256, None, false)?;
        if coordination::file_id(&source)? != peer.identity().executable_id {
            return Err(invalid("fixture source is not the running launcher image"));
        }
        let bundle = checked_image(Path::new(BUNDLE), super::BUNDLE_SHA, None, false)?;
        if coordination::file_id(&source)?.volume != volume
            || coordination::file_id(&bundle)?.volume != volume
        {
            return Err(invalid(
                "fixture source or official bundle crosses the Profile volume",
            ));
        }
        let base_path = Path::new(BASE);
        let image_path = config.installation.join("ocxd.exe");
        let base;
        let installation;
        let data;
        let home;
        let mut created_image_id = None;
        if host {
            base = create(&boundary.profile, base_path)?;
            installation = create(&base, &config.installation)?;
            data = create(&base, &config.data)?;
            home = create(&base, &config.home)?;
            fixed_leaf(&installation, &image_path)?;
            let mut image = super::platform::create_new_private_file(&image_path, 0o600)?;
            if io::copy(&mut source, &mut image)? != source.metadata()?.len() {
                return Err(invalid("fixture image copy length differs"));
            }
            image.sync_all()?;
            created_image_id = Some(coordination::file_id(&image)?);
            installation.recheck()?;
        } else {
            base = canonical_pin(base_path, true)?;
            installation = canonical_pin(&config.installation, true)?;
            data = canonical_pin(&config.data, true)?;
            home = canonical_pin(&config.home, true)?;
        }
        let image = checked_image(&image_path, &config.broker_sha256, None, true)?;
        let image_id = coordination::file_id(&image)?;
        if created_image_id.is_some_and(|created| created != image_id)
            || image_id == coordination::file_id(&source)?
        {
            return Err(invalid("staged and private fixture image alias"));
        }
        let profile_lineage = boundary.profile.owned_file_identities()?;
        let base_lineage = base.owned_file_identities()?;
        if base_lineage.len() != profile_lineage.len() + 1
            || !base_lineage.starts_with(&profile_lineage)
        {
            return Err(invalid(
                "staged fixture base is outside its original Profile",
            ));
        }
        let mut ids = vec![base.identity(), image_id];
        for pin in [&installation, &data, &home] {
            let lineage = pin.owned_file_identities()?;
            if lineage.len() != base_lineage.len() + 1
                || !lineage.starts_with(&base_lineage)
                || pin.identity().volume != volume
                || ids.contains(&pin.identity())
            {
                return Err(invalid(
                    "staged private I/D/H escaped or aliased the fixture",
                ));
            }
            ids.push(pin.identity());
        }
        input.owned_file_identities()?;
        out.owned_file_identities()?;
        boundary.recheck()?;
        Ok(Self {
            _boundary: boundary,
            _pins: vec![input, out, base, installation, data, home],
            _images: [source, image, bundle],
        })
    }
}
