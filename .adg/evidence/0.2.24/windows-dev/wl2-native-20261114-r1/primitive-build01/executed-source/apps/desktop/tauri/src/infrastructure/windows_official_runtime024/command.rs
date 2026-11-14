//! Capture live native command/parent facts from retained original Job handles.
//! Missing/short-lived capture is an explicit error, never a guessed argv/role.
use super::*;

#[derive(Clone, Debug)]
pub struct CommandEvidence {
    pub raw_command_line: String,
    pub argv: Vec<String>,
    pub parent: ProcessIdentity,
}
#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtQueryInformationProcess(
        process: HANDLE,
        class: u32,
        data: *mut std::ffi::c_void,
        size: u32,
        needed: *mut u32,
    ) -> i32;
    fn RtlNtStatusToDosError(status: i32) -> u32;
}
#[link(name = "shell32")]
unsafe extern "system" {
    fn CommandLineToArgvW(command: *const u16, count: *mut i32) -> *mut *mut u16;
}
fn nt_error(status: i32) -> io::Error {
    io::Error::other(format!(
        "native process command query NTSTATUS=0x{:08x} Win32={}",
        status as u32,
        unsafe { RtlNtStatusToDosError(status) }
    ))
}
pub(super) fn capture(process: HANDLE, job: HANDLE, sid: &str) -> io::Result<CommandEvidence> {
    if !matches!(state(process)?, ProcessState::Running) {
        return Err(denied("child exited before native argv capture"));
    }
    let mut needed = 0;
    let first = unsafe { NtQueryInformationProcess(process, 60, ptr::null_mut(), 0, &mut needed) };
    if !(16..=131072).contains(&needed) {
        return Err(nt_error(first));
    }
    let mut words = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
    let capacity = words.len() * size_of::<usize>();
    let status = unsafe {
        NtQueryInformationProcess(
            process,
            60,
            words.as_mut_ptr().cast(),
            capacity as u32,
            &mut needed,
        )
    };
    if status != 0 {
        return Err(nt_error(status));
    }
    #[repr(C)]
    struct Unicode {
        length: u16,
        maximum: u16,
        buffer: *const u16,
    }
    let s = unsafe { &*words.as_ptr().cast::<Unicode>() };
    let start = words.as_ptr() as usize;
    let end = start + capacity;
    let p = s.buffer as usize;
    if s.length == 0
        || s.length % 2 != 0
        || s.length > s.maximum
        || p < start + size_of::<Unicode>()
        || p % 2 != 0
        || p.checked_add(s.length as usize).is_none_or(|n| n > end)
    {
        return Err(denied("native command UNICODE_STRING boundary"));
    }
    let raw = unsafe { std::slice::from_raw_parts(s.buffer, s.length as usize / 2) };
    if raw.contains(&0) {
        return Err(denied("NUL in native command"));
    }
    let raw_command_line =
        String::from_utf16(raw).map_err(|_| denied("non-Unicode child command"))?;
    let terminated: Vec<u16> = raw.iter().copied().chain(Some(0)).collect();
    let mut count = 0;
    let args = unsafe { CommandLineToArgvW(terminated.as_ptr(), &mut count) };
    if args.is_null() {
        return Err(io::Error::last_os_error());
    }
    struct Local(*mut std::ffi::c_void);
    impl Drop for Local {
        fn drop(&mut self) {
            unsafe {
                windows_sys::Win32::Foundation::LocalFree(self.0);
            }
        }
    }
    let _args = Local(args.cast());
    if !(1..=256).contains(&count) {
        return Err(denied("native argv count bound"));
    }
    let mut argv = Vec::new();
    for i in 0..count as usize {
        let arg = unsafe { *args.add(i) };
        let length = (0..=32768)
            .find(|&n| unsafe { *arg.add(n) } == 0)
            .ok_or_else(|| denied("native argv string bound"))?;
        argv.push(
            String::from_utf16(unsafe { std::slice::from_raw_parts(arg, length) })
                .map_err(|_| denied("invalid native argv"))?,
        );
    }
    // PROCESS_BASIC_INFORMATION: six pointer-sized slots on the pinned x64
    // target, final slot is InheritedFromUniqueProcessId. No global PID scan.
    let mut basic = [0usize; 6];
    let status = unsafe {
        NtQueryInformationProcess(
            process,
            0,
            basic.as_mut_ptr().cast(),
            size_of::<[usize; 6]>() as u32,
            &mut needed,
        )
    };
    if status != 0 {
        return Err(nt_error(status));
    }
    let parent_pid = u32::try_from(basic[5]).map_err(|_| denied("invalid native parent PID"))?;
    let parent = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            parent_pid,
        )
    };
    if parent.is_null() {
        return Err(io::Error::last_os_error());
    }
    let parent = unsafe { OwnedHandle::from_raw_handle(parent) };
    let mut in_job = 0;
    check(unsafe { IsProcessInJob(parent.as_raw_handle(), job, &mut in_job) })?;
    if in_job == 0 {
        return Err(denied("native parent not in original Job"));
    }
    let parent = observe(parent.as_raw_handle())?;
    if parent.token.sid != sid || !matches!(state(process)?, ProcessState::Running) {
        return Err(denied(
            "parent SID differs or child exited during command capture",
        ));
    }
    Ok(CommandEvidence {
        raw_command_line,
        argv,
        parent,
    })
}
