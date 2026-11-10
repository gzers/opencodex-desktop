use super::{
    paths::{identity, validate, wide},
    security::{
        check, current_identity, denied, process_identity, verify_private_job, PrivateDescriptor,
    },
    FileIdentity, PinnedDirectory, TokenIdentity,
};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::{OsStr, OsString},
    fs::{File, OpenOptions},
    io,
    mem::size_of,
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Path, PathBuf},
    ptr,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        DuplicateHandle, DUPLICATE_SAME_ACCESS, FILETIME, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    System::{
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob,
            JobObjectBasicAccountingInformation, JobObjectBasicProcessIdList,
            JobObjectExtendedLimitInformation, QueryInformationJobObject, TerminateJobObject,
            JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        },
        Threading::{
            CreateProcessW, DeleteProcThreadAttributeList, GetCurrentProcess, GetExitCodeProcess,
            GetProcessId, GetProcessTimes, InitializeProcThreadAttributeList, OpenProcess,
            QueryFullProcessImageNameW, ResumeThread, TerminateProcess, UpdateProcThreadAttribute,
            WaitForSingleObject, CREATE_NO_WINDOW, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT,
            EXTENDED_STARTUPINFO_PRESENT, PROCESS_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
            PROCESS_SYNCHRONIZE, PROC_THREAD_ATTRIBUTE_HANDLE_LIST, STARTF_USESTDHANDLES,
            STARTUPINFOEXW,
        },
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub creation_filetime: u64,
    pub token: TokenIdentity,
    pub executable: PathBuf,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessState {
    Running,
    Exited { code: u32 },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootBinding {
    pub process: ProcessIdentity,
    pub home: PathBuf,
    pub home_id: FileIdentity,
    pub executable_id: FileIdentity,
}
#[derive(Clone, Debug)]
pub struct FamilyState {
    pub root: ProcessState,
    pub active_processes: u32,
    pub active_members: Vec<ProcessIdentity>,
    pub retained_members: Vec<(ProcessIdentity, ProcessState)>,
}
impl FamilyState {
    pub fn complete_exit(&self) -> bool {
        matches!(self.root, ProcessState::Exited { .. })
            && self.active_processes == 0
            && self.active_members.is_empty()
            && self
                .retained_members
                .iter()
                .all(|(_, s)| matches!(s, ProcessState::Exited { .. }))
    }
}
/// This fact is deliberately distinct from a normal exit observation.
#[derive(Clone, Debug)]
pub struct ForcedCleanup {
    pub forced: bool,
    pub final_state: FamilyState,
}

/// Explicit already-admitted inputs. The shared caller selects the executable,
/// verifies package hashes and supplies a minimal environment. No process/global
/// user environment is inherited; HOME and OPENCODEX_HOME are pinned here.
pub struct LaunchSpec<'a> {
    pub executable: &'a Path,
    pub arguments: &'a [OsString],
    pub working_directory: &'a Path,
    pub home: &'a Path,
    pub environment: &'a BTreeMap<OsString, OsString>,
    pub stdout: &'a File,
    pub stderr: &'a File,
}

fn observe(h: HANDLE) -> io::Result<ProcessIdentity> {
    let mut c = FILETIME::default();
    let mut e = FILETIME::default();
    let mut k = FILETIME::default();
    let mut u = FILETIME::default();
    check(unsafe { GetProcessTimes(h, &mut c, &mut e, &mut k, &mut u) })?;
    let pid = unsafe { GetProcessId(h) };
    if pid == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut b = vec![0u16; 32768];
    let mut n = b.len() as u32;
    check(unsafe { QueryFullProcessImageNameW(h, 0, b.as_mut_ptr(), &mut n) })?;
    Ok(ProcessIdentity {
        pid,
        creation_filetime: ((c.dwHighDateTime as u64) << 32) | c.dwLowDateTime as u64,
        token: process_identity(h)?,
        executable: PathBuf::from(
            String::from_utf16(&b[..n as usize])
                .map_err(|_| denied("invalid process executable path"))?,
        ),
    })
}
fn state(h: HANDLE) -> io::Result<ProcessState> {
    match unsafe { WaitForSingleObject(h, 0) } {
        WAIT_TIMEOUT => Ok(ProcessState::Running),
        WAIT_OBJECT_0 => {
            let mut code = 0;
            check(unsafe { GetExitCodeProcess(h, &mut code) })?;
            Ok(ProcessState::Exited { code })
        }
        _ => Err(io::Error::last_os_error()),
    }
}
fn duplicate(file: &File) -> io::Result<OwnedHandle> {
    let mut h = ptr::null_mut();
    check(unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            file.as_raw_handle(),
            GetCurrentProcess(),
            &mut h,
            0,
            1,
            DUPLICATE_SAME_ACCESS,
        )
    })?;
    Ok(unsafe { OwnedHandle::from_raw_handle(h) })
}
fn quote(arg: &OsStr, out: &mut Vec<u16>) -> io::Result<()> {
    let s: Vec<u16> = arg.encode_wide().collect();
    if s.contains(&0) {
        return Err(denied("nul argument"));
    }
    out.push(34);
    let mut slash = 0;
    for c in s {
        if c == 92 {
            slash += 1;
            continue;
        }
        if c == 34 {
            out.extend(std::iter::repeat_n(92, slash * 2 + 1));
        } else {
            out.extend(std::iter::repeat_n(92, slash));
        }
        slash = 0;
        out.push(c);
    }
    out.extend(std::iter::repeat_n(92, slash * 2));
    out.push(34);
    Ok(())
}
fn environment(input: &BTreeMap<OsString, OsString>, home: &Path) -> io::Result<Vec<u16>> {
    let mut pairs = BTreeMap::new();
    for (k, v) in input {
        let key = k
            .to_str()
            .ok_or_else(|| denied("environment key must be Unicode"))?;
        if key.is_empty() || key.contains(['=', '\0']) || v.encode_wide().any(|x| x == 0) {
            return Err(denied("invalid environment pair"));
        }
        let normalized = key.to_uppercase();
        if pairs.insert(normalized, (k.clone(), v.clone())).is_some() {
            return Err(denied("case-duplicate environment key"));
        }
    }
    for name in ["HOME", "OPENCODEX_HOME"] {
        pairs.insert(name.into(), (name.into(), home.as_os_str().to_owned()));
    }
    let mut out = Vec::new();
    for (_, (k, v)) in pairs {
        out.extend(k.encode_wide());
        out.push(61);
        out.extend(v.encode_wide());
        out.push(0);
    }
    out.push(0);
    if out.len() > 32767 {
        return Err(denied("environment exceeds bound"));
    }
    Ok(out)
}
struct Attributes {
    storage: Vec<usize>,
    initialized: bool,
}
impl Attributes {
    fn new(handles: &[HANDLE]) -> io::Result<Self> {
        let mut size = 0;
        unsafe {
            InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut size);
        }
        if size == 0 || size > 65536 {
            return Err(denied("attribute list size unavailable"));
        }
        let mut this = Self {
            storage: vec![0; size.div_ceil(size_of::<usize>())],
            initialized: false,
        };
        check(unsafe { InitializeProcThreadAttributeList(this.raw(), 1, 0, &mut size) })?;
        this.initialized = true;
        check(unsafe {
            UpdateProcThreadAttribute(
                this.raw(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast(),
                std::mem::size_of_val(handles),
                ptr::null_mut(),
                ptr::null(),
            )
        })?;
        Ok(this)
    }
    fn raw(&mut self) -> *mut std::ffi::c_void {
        self.storage.as_mut_ptr().cast()
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                DeleteProcThreadAttributeList(self.raw());
            }
        }
    }
}

/// Owns the original handles and anonymous no-breakaway Job. Drop never silently
/// force-stops a running service. Callers must retain this owner through normal
/// Stop observation or explicitly request separately reported owned cleanup.
pub struct OwnedRuntime {
    process: OwnedHandle,
    thread: Option<OwnedHandle>,
    job: OwnedHandle,
    binding: RootBinding,
    home: PinnedDirectory,
    _program_parent: PinnedDirectory,
    _cwd: PinnedDirectory,
    _executable: File,
    members: HashMap<u32, (OwnedHandle, ProcessIdentity)>,
    resumed: bool,
}
impl OwnedRuntime {
    pub fn create_suspended(spec: LaunchSpec<'_>) -> io::Result<Self> {
        validate(spec.executable)?;
        let program_parent = PinnedDirectory::open(
            spec.executable
                .parent()
                .ok_or_else(|| denied("program parent required"))?,
        )?;
        let cwd = PinnedDirectory::open(spec.working_directory)?;
        let home = PinnedDirectory::open_private(spec.home)?;
        let current = current_identity()?;
        let exe = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(0x00200000)
            .open(spec.executable)?;
        let executable_id = identity(&exe, false)?;
        let sd = PrivateDescriptor::new(&current.sid, false)?;
        let sa = sd.attributes();
        let raw = unsafe { CreateJobObjectW(&sa, ptr::null()) };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        let job = unsafe { OwnedHandle::from_raw_handle(raw) };
        verify_private_job(raw, &current.sid)?;
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        check(unsafe {
            QueryInformationJobObject(
                raw,
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                ptr::null_mut(),
            )
        })?;
        if limits.BasicLimitInformation.LimitFlags & 0x1800 != 0 {
            return Err(denied("Job breakaway is not permitted"));
        }
        let nul = File::open("NUL")?;
        let input = duplicate(&nul)?;
        let output = duplicate(spec.stdout)?;
        let error = duplicate(spec.stderr)?;
        let handles = [
            input.as_raw_handle(),
            output.as_raw_handle(),
            error.as_raw_handle(),
        ];
        let mut attrs = Attributes::new(&handles)?;
        let mut si = STARTUPINFOEXW::default();
        si.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        si.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        si.StartupInfo.hStdInput = handles[0];
        si.StartupInfo.hStdOutput = handles[1];
        si.StartupInfo.hStdError = handles[2];
        si.lpAttributeList = attrs.raw();
        let app = wide(spec.executable)?;
        let work = wide(spec.working_directory)?;
        let env = environment(spec.environment, spec.home)?;
        let mut cmd = Vec::new();
        quote(spec.executable.as_os_str(), &mut cmd)?;
        for arg in spec.arguments {
            cmd.push(32);
            quote(arg, &mut cmd)?;
        }
        cmd.push(0);
        if cmd.len() > 32767 {
            return Err(denied("command line exceeds bound"));
        }
        let mut pi = PROCESS_INFORMATION::default();
        check(unsafe {
            CreateProcessW(
                app.as_ptr(),
                cmd.as_mut_ptr(),
                ptr::null(),
                ptr::null(),
                1,
                CREATE_SUSPENDED
                    | CREATE_NO_WINDOW
                    | CREATE_UNICODE_ENVIRONMENT
                    | EXTENDED_STARTUPINFO_PRESENT,
                env.as_ptr().cast(),
                work.as_ptr(),
                &si.StartupInfo,
                &mut pi,
            )
        })?;
        let process = unsafe { OwnedHandle::from_raw_handle(pi.hProcess) };
        let thread = unsafe { OwnedHandle::from_raw_handle(pi.hThread) };
        let admission = (|| {
            let fact = observe(process.as_raw_handle())?;
            if fact.pid != pi.dwProcessId
                || fact.token != current
                || state(process.as_raw_handle())? != ProcessState::Running
            {
                return Err(denied("new suspended process identity differs"));
            }
            let observed = OpenOptions::new()
                .read(true)
                .share_mode(1)
                .custom_flags(0x00200000)
                .open(&fact.executable)?;
            if identity(&observed, false)? != executable_id {
                return Err(denied(
                    "original process image differs from pinned executable",
                ));
            }
            check(unsafe {
                AssignProcessToJobObject(job.as_raw_handle(), process.as_raw_handle())
            })?;
            let mut yes = 0;
            check(unsafe {
                IsProcessInJob(process.as_raw_handle(), job.as_raw_handle(), &mut yes)
            })?;
            if yes == 0 {
                return Err(denied("process absent from owned Job before resume"));
            }
            home.recheck()?;
            Ok(fact)
        })();
        let fact = match admission {
            Ok(f) => f,
            Err(e) => {
                check(unsafe { TerminateProcess(process.as_raw_handle(), 241) })?;
                if unsafe { WaitForSingleObject(process.as_raw_handle(), 3000) } != WAIT_OBJECT_0 {
                    return Err(io::Error::other(
                        "suspended admission failed and cleanup exit is unknown",
                    ));
                }
                return Err(e);
            }
        };
        Ok(Self {
            process,
            thread: Some(thread),
            job,
            binding: RootBinding {
                process: fact,
                home: spec.home.into(),
                home_id: home.identity(),
                executable_id,
            },
            home,
            _program_parent: program_parent,
            _cwd: cwd,
            _executable: exe,
            members: HashMap::new(),
            resumed: false,
        })
    }
    pub fn binding(&self) -> &RootBinding {
        &self.binding
    }
    pub fn root_state(&self) -> io::Result<ProcessState> {
        state(self.process.as_raw_handle())
    }
    pub fn verify_binding(&self, expected: &RootBinding) -> io::Result<()> {
        if expected != &self.binding {
            return Err(denied(
                "runtime generation, HOME, SID or image binding differs",
            ));
        }
        self.home.recheck()?;
        if observe(self.process.as_raw_handle())? != self.binding.process {
            return Err(denied("original process facts changed"));
        }
        Ok(())
    }
    pub fn resume(&mut self) -> io::Result<()> {
        if self.resumed {
            return Err(denied("runtime was already resumed"));
        }
        self.verify_binding(&self.binding)?;
        let h = self
            .thread
            .as_ref()
            .ok_or_else(|| denied("missing primary thread"))?;
        let n = unsafe { ResumeThread(h.as_raw_handle()) };
        if n != 1 {
            return Err(if n == u32::MAX {
                io::Error::last_os_error()
            } else {
                denied("unexpected primary-thread suspension count")
            });
        }
        self.resumed = true;
        self.thread.take();
        Ok(())
    }
    fn member_ids(&self) -> io::Result<Vec<u32>> {
        #[repr(C)]
        struct Ids {
            assigned: u32,
            listed: u32,
            ids: [usize; 4096],
        }
        let mut b = Box::new(Ids {
            assigned: 0,
            listed: 0,
            ids: [0; 4096],
        });
        check(unsafe {
            QueryInformationJobObject(
                self.job.as_raw_handle(),
                JobObjectBasicProcessIdList,
                (&mut *b as *mut Ids).cast(),
                size_of::<Ids>() as u32,
                ptr::null_mut(),
            )
        })?;
        if b.assigned != b.listed || b.listed as usize > b.ids.len() {
            return Err(denied("incomplete owned Job member list"));
        }
        b.ids[..b.listed as usize]
            .iter()
            .map(|&n| u32::try_from(n).map_err(|_| denied("invalid Job PID")))
            .collect()
    }
    pub fn family_state(&mut self) -> io::Result<FamilyState> {
        let ids = self.member_ids()?;
        let mut active = Vec::new();
        for id in ids {
            if id == self.binding.process.pid {
                active.push(self.binding.process.clone());
                continue;
            }
            if let Some((h, _)) = self.members.get(&id) {
                if matches!(state(h.as_raw_handle())?, ProcessState::Exited { .. }) {
                    return Err(denied("PID reuse observed in owned member set"));
                }
            }
            if !self.members.contains_key(&id) {
                let raw = unsafe {
                    OpenProcess(
                        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                        0,
                        id,
                    )
                };
                if raw.is_null() {
                    return Err(io::Error::last_os_error());
                }
                let h = unsafe { OwnedHandle::from_raw_handle(raw) };
                let mut yes = 0;
                check(unsafe { IsProcessInJob(raw, self.job.as_raw_handle(), &mut yes) })?;
                if yes == 0 {
                    return Err(denied(
                        "listed PID generation no longer belongs to owned Job",
                    ));
                }
                let fact = observe(raw)?;
                if fact.token.sid != self.binding.process.token.sid
                    || fact.creation_filetime < self.binding.process.creation_filetime
                {
                    return Err(denied("owned descendant identity differs"));
                }
                self.members.insert(id, (h, fact));
            }
            active.push(self.members[&id].1.clone());
        }
        let mut accounting = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        check(unsafe {
            QueryInformationJobObject(
                self.job.as_raw_handle(),
                JobObjectBasicAccountingInformation,
                (&mut accounting as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                ptr::null_mut(),
            )
        })?;
        if accounting.ActiveProcesses != active.len() as u32 {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "owned Job changed across bounded snapshot",
            ));
        }
        let retained = self
            .members
            .values()
            .map(|(h, f)| Ok((f.clone(), state(h.as_raw_handle())?)))
            .collect::<io::Result<_>>()?;
        Ok(FamilyState {
            root: self.root_state()?,
            active_processes: accounting.ActiveProcesses,
            active_members: active,
            retained_members: retained,
        })
    }
    /// Observes only. No signals, HTTP, PID killing, timeout extension or forced fallback.
    pub fn wait_normal_exit(&mut self, deadline: Instant) -> io::Result<FamilyState> {
        loop {
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "original observation deadline elapsed",
                ));
            }
            let fact = self.family_state();
            match fact {
                Ok(s) if s.complete_exit() && Instant::now() < deadline => return Ok(s),
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "original process or owned Job is not completely exited",
                ));
            }
            std::thread::sleep(
                Duration::from_millis(5).min(deadline.saturating_duration_since(Instant::now())),
            );
        }
    }
    /// Explicit failure cleanup for the exclusively retained owned Job. Its return
    /// type cannot be mistaken for a successful normal stop by this module.
    pub fn force_owned_cleanup(&mut self, deadline: Instant) -> io::Result<ForcedCleanup> {
        check(unsafe { TerminateJobObject(self.job.as_raw_handle(), 242) })?;
        Ok(ForcedCleanup {
            forced: true,
            final_state: self.wait_normal_exit(deadline)?,
        })
    }
}
