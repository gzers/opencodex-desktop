//! Create the original owner host suspended without assigning an outer Job.
//! Bootstrap input must be an anonymous/private pipe, never a credential file.
use super::*;
pub struct SuspendedOwnerHost {
    peer: super::super::RetainedPeer,
    thread: Option<OwnedHandle>,
    image: File,
    parent: PinnedDirectory,
    cwd: PinnedDirectory,
}
impl SuspendedOwnerHost {
    pub fn create(spec: LaunchSpec<'_>, bootstrap: &File) -> io::Result<Self> {
        let creator = super::super::RetainedPeer::current()?;
        creator.no_outer_job()?;
        if creator.identity().token.elevated {
            return Err(denied("ordinary bootstrap token required"));
        }
        if spec.arguments != [OsString::from("--internal-runtime-owner024")]
            || spec.executable.file_name() != Some(OsStr::new("ocxd.exe"))
        {
            return Err(denied("exact internal owner executable/mode required"));
        }
        if unsafe {
            windows_sys::Win32::Storage::FileSystem::GetFileType(bootstrap.as_raw_handle())
        } != 3
        {
            return Err(denied(
                "bootstrap must be a private inherited pipe, not disk/argv",
            ));
        }
        validate(spec.executable)?;
        let parent = PinnedDirectory::open(
            spec.executable
                .parent()
                .ok_or_else(|| denied("owner image parent"))?,
        )?;
        let cwd = PinnedDirectory::open(spec.working_directory)?;
        let image = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(spec.executable)?;
        let image_id = identity(&image, false)?;
        let input = duplicate(bootstrap)?;
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
        let env = environment(spec.environment, spec.user_profile, spec.opencodex_home)?;
        let mut cmd = Vec::new();
        quote(spec.executable.as_os_str(), &mut cmd)?;
        cmd.push(32);
        quote(OsStr::new("--internal-runtime-owner024"), &mut cmd)?;
        cmd.push(0);
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
        let admitted = (|| {
            let observed = observe(process.as_raw_handle())?;
            if observed.pid != pi.dwProcessId
                || observed.token != creator.identity().token
                || observed.executable_id != image_id
            {
                return Err(denied(
                    "suspended host native generation/token/image differs",
                ));
            }
            let mut inside = 0;
            check(unsafe {
                IsProcessInJob(process.as_raw_handle(), ptr::null_mut(), &mut inside)
            })?;
            if inside != 0 {
                return Err(denied("suspended original broker inherits unproven Job"));
            }
            creator.no_outer_job()?;
            parent.recheck()?;
            cwd.recheck()?;
            Ok(())
        })();
        if let Err(e) = admitted {
            // This exclusively created process is still suspended, with no
            // official launcher or client mutation. Cleanup is never normal Stop.
            check(unsafe { TerminateProcess(process.as_raw_handle(), 241) })?;
            if unsafe { WaitForSingleObject(process.as_raw_handle(), 3000) } != WAIT_OBJECT_0 {
                return Err(denied("suspended bootstrap failure cleanup unknown"));
            }
            return Err(e);
        }
        Ok(Self {
            peer: super::super::RetainedPeer::from_original_handle(process)?,
            thread: Some(thread),
            image,
            parent,
            cwd,
        })
    }
    pub fn original_peer(&self) -> &super::super::RetainedPeer {
        &self.peer
    }
    pub fn resume(&mut self) -> io::Result<()> {
        self.peer.no_outer_job()?;
        self.parent.recheck()?;
        self.cwd.recheck()?;
        if identity(&self.image, false)? != self.peer.identity().executable_id {
            return Err(denied("original broker image drift"));
        }
        let thread = self
            .thread
            .as_ref()
            .ok_or_else(|| denied("host already resumed"))?;
        if unsafe { ResumeThread(thread.as_raw_handle()) } != 1 {
            return Err(io::Error::last_os_error());
        }
        self.thread = None;
        self.peer.no_outer_job()
    }
}
