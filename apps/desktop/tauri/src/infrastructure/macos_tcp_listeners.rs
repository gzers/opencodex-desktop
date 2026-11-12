//! Exact live generation's complete FD table, twice observed with target-SDK
//! socket classification. Covers TCP listeners only; not descendants, escaped
//! sockets, transferred descriptors or ongoing writer exclusion/restore rights.
use super::macos_process_exit::Generation;
use crate::errors::{AppError, AppResult};
use std::time::Instant;
const MAX_FDS: usize = 65_536;
#[repr(C)]
#[derive(Default)]
struct NativeRow {
    object: u64,
    family: u32,
    port: u32,
}
unsafe extern "C" {
    fn ocxd_macos_tcp_listener(pid: i32, fd: i32, out: *mut NativeRow) -> i32;
}
fn fail(code: &str) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: "原服务进程的TCP监听集合未核对，未发送停止请求或宣称完整停止；请查看诊断。".into(),
    }
}
#[derive(Clone, PartialEq, Eq)]
struct Listener {
    fd: i32,
    object: u64,
    family: u32,
    port: u16,
}
type ListenerObservation = (Vec<(i32, u32)>, Vec<Listener>);
pub struct TcpListeners {
    generation: Generation,
    ports: Vec<u16>,
    _listeners: Vec<Listener>,
}
impl std::fmt::Debug for TcpListeners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TcpListeners")
            .field("pid", &self.generation.pid)
            .field("count", &self.ports.len())
            .finish_non_exhaustive()
    }
}
fn time(deadline: Instant) -> AppResult<()> {
    if Instant::now() >= deadline {
        Err(fail("lifecycle_timeout"))
    } else {
        Ok(())
    }
}
fn descriptors(expected: Generation) -> AppResult<Vec<(i32, u32)>> {
    let row = std::mem::size_of::<libc::proc_fdinfo>();
    let bytes = unsafe {
        libc::proc_pidinfo(
            expected.pid as i32,
            libc::PROC_PIDLISTFDS,
            0,
            std::ptr::null_mut(),
            0,
        )
    };
    if bytes <= 0 || bytes as usize > MAX_FDS * row || !(bytes as usize).is_multiple_of(row) {
        return Err(fail("lifecycle_listener_fd_table"));
    }
    let capacity = (bytes as usize / row).saturating_add(32).min(MAX_FDS);
    let mut table: Vec<libc::proc_fdinfo> = (0..capacity)
        .map(|_| unsafe { std::mem::zeroed() })
        .collect();
    let count = unsafe {
        libc::proc_pidinfo(
            expected.pid as i32,
            libc::PROC_PIDLISTFDS,
            0,
            table.as_mut_ptr().cast(),
            (capacity * row) as i32,
        )
    };
    if count <= 0 || count as usize >= capacity * row || !(count as usize).is_multiple_of(row) {
        return Err(fail("lifecycle_listener_fd_table"));
    }
    let mut result = Vec::new();
    for entry in &table[..count as usize / row] {
        if entry.proc_fd < 0 {
            return Err(fail("lifecycle_listener_fd_table"));
        }
        result.push((entry.proc_fd, entry.proc_fdtype));
    }
    result.sort_unstable();
    if result.windows(2).any(|w| w[0].0 == w[1].0) {
        return Err(fail("lifecycle_listener_fd_table"));
    }
    Ok(result)
}
impl TcpListeners {
    pub fn capture(expected: Generation, deadline: Instant) -> AppResult<Self> {
        Self::capture_checked(expected, deadline, &|| Ok(()))
    }
    fn capture_checked(
        expected: Generation,
        deadline: Instant,
        checkpoint: &impl Fn() -> AppResult<()>,
    ) -> AppResult<Self> {
        let one = || -> AppResult<ListenerObservation> {
            time(deadline)?;
            if Generation::observe(expected.pid)? != expected {
                return Err(fail("lifecycle_listener_generation"));
            }
            let table = descriptors(expected)?;
            let mut listeners = Vec::new();
            for (fd, kind) in &table {
                time(deadline)?;
                if *kind != libc::PROX_FDTYPE_SOCKET as u32 {
                    continue;
                }
                let mut row = NativeRow::default();
                match unsafe { ocxd_macos_tcp_listener(expected.pid as i32, *fd, &mut row) } {
                    0 => (),
                    1 => {
                        let port = u16::try_from(row.port)
                            .ok()
                            .filter(|p| *p != 0)
                            .ok_or_else(|| fail("lifecycle_listener_socket"))?;
                        if row.object == 0
                            || ![libc::AF_INET as u32, libc::AF_INET6 as u32].contains(&row.family)
                        {
                            return Err(fail("lifecycle_listener_socket"));
                        }
                        listeners.push(Listener {
                            fd: *fd,
                            object: row.object,
                            family: row.family,
                            port,
                        });
                    }
                    _ => return Err(fail("lifecycle_listener_socket")),
                }
            }
            let sockets = |rows: Vec<(i32, u32)>| {
                rows.into_iter()
                    .filter(|(_, kind)| *kind == libc::PROX_FDTYPE_SOCKET as u32)
                    .collect::<Vec<_>>()
            };
            let socket_table = sockets(table);
            if Generation::observe(expected.pid)? != expected
                || sockets(descriptors(expected)?) != socket_table
            {
                return Err(fail("lifecycle_listener_changed"));
            }
            time(deadline)?;
            Ok((socket_table, listeners))
        };
        let before = one()?;
        checkpoint()?;
        let after = one()?;
        if before != after {
            return Err(fail("lifecycle_listener_changed"));
        }
        let mut ports: Vec<_> = after.1.iter().map(|s| s.port).collect();
        ports.sort_unstable();
        ports.dedup();
        Ok(Self {
            generation: expected,
            ports,
            _listeners: after.1,
        })
    }
    pub fn generation(&self) -> Generation {
        self.generation
    }
    pub fn ports(&self) -> &[u16] {
        &self.ports
    }
}

#[cfg(test)]
#[path = "macos_tcp_listeners_tests.rs"]
mod tests;
