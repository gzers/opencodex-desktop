use super::{security::denied, FamilyState, OwnedRuntime, ProcessIdentity};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ptr,
};
#[link(name = "iphlpapi")]
unsafe extern "system" {
    fn GetExtendedTcpTable(
        table: *mut std::ffi::c_void,
        size: *mut u32,
        order: i32,
        family: u32,
        class: i32,
        reserved: u32,
    ) -> u32;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ListenerFamily {
    Ipv4,
    Ipv6,
}
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Listener {
    pub family: ListenerFamily,
    pub address: IpAddr,
    pub scope_id: u32,
    pub port: u16,
    pub pid: u32,
}
#[derive(Clone, Debug)]
pub struct OwnedListeners {
    pub listeners: Vec<(Listener, ProcessIdentity)>,
    pub family: FamilyState,
}
fn word(b: &[u8], n: usize) -> u32 {
    u32::from_ne_bytes(b[n..n + 4].try_into().unwrap())
}
/// Reads only native TCP tables, not process command lines or global owners.
/// Both families must be readable; an incomplete/oversized table returns error.
pub fn tcp_listeners() -> io::Result<Vec<Listener>> {
    let mut rows = Vec::new();
    for af in [2, 23] {
        let mut size = 0;
        let initial = unsafe { GetExtendedTcpTable(ptr::null_mut(), &mut size, 0, af, 3, 0) };
        if initial != 0 && initial != 122 {
            return Err(io::Error::from_raw_os_error(initial as i32));
        }
        let mut complete = None;
        for _ in 0..4 {
            if !(4..=16 * 1024 * 1024).contains(&size) {
                return Err(denied("TCP table size outside bound"));
            }
            let mut data = vec![0u32; (size as usize).div_ceil(4)];
            let capacity = data.len() * 4;
            let code =
                unsafe { GetExtendedTcpTable(data.as_mut_ptr().cast(), &mut size, 0, af, 3, 0) };
            if code == 122 {
                continue;
            }
            if code != 0 {
                return Err(io::Error::from_raw_os_error(code as i32));
            }
            if size < 4 || size as usize > capacity {
                return Err(denied("TCP table exceeds returned allocation"));
            }
            let bytes =
                unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), size as usize) };
            let count = word(bytes, 0) as usize;
            let stride = if af == 2 { 24 } else { 56 };
            if count
                .checked_mul(stride)
                .and_then(|n| n.checked_add(4))
                .is_none_or(|n| n > bytes.len())
            {
                return Err(denied("incomplete native TCP table"));
            }
            let mut local = Vec::new();
            for i in 0..count {
                let start = 4 + i * stride;
                let state = word(bytes, start + if af == 2 { 0 } else { 48 });
                if state != 2 {
                    return Err(denied("non-listener in listener table"));
                }
                let p = start + if af == 2 { 8 } else { 20 };
                let port = u16::from_be_bytes([bytes[p], bytes[p + 1]]);
                let (family, address, scope) = if af == 2 {
                    (
                        ListenerFamily::Ipv4,
                        IpAddr::V4(Ipv4Addr::new(
                            bytes[start + 4],
                            bytes[start + 5],
                            bytes[start + 6],
                            bytes[start + 7],
                        )),
                        0,
                    )
                } else {
                    (
                        ListenerFamily::Ipv6,
                        IpAddr::V6(Ipv6Addr::from(
                            <[u8; 16]>::try_from(&bytes[start..start + 16]).unwrap(),
                        )),
                        word(bytes, start + 16),
                    )
                };
                local.push(Listener {
                    family,
                    address,
                    scope_id: scope,
                    port,
                    pid: word(bytes, start + stride - 4),
                });
            }
            complete = Some(local);
            break;
        }
        rows.extend(complete.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "native listener table kept changing",
            )
        })?);
    }
    Ok(rows)
}
impl OwnedRuntime {
    /// Bind table rows to original retained handles, then recheck the owned set.
    /// A changed set is unknown; callers may retry inside their existing deadline.
    pub fn listeners(&mut self) -> io::Result<OwnedListeners> {
        let before = self.family_state()?;
        let rows = tcp_listeners()?;
        let after = self.family_state()?;
        let mut a: Vec<_> = before
            .active_members
            .iter()
            .map(|x| (x.pid, x.creation_filetime))
            .collect();
        let mut b: Vec<_> = after
            .active_members
            .iter()
            .map(|x| (x.pid, x.creation_filetime))
            .collect();
        a.sort_unstable();
        b.sort_unstable();
        if a != b {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Job membership changed during TCP snapshot",
            ));
        }
        let mut found = Vec::new();
        for row in rows {
            if let Some(f) = before.active_members.iter().find(|x| x.pid == row.pid) {
                found.push((row, f.clone()));
            }
        }
        Ok(OwnedListeners {
            listeners: found,
            family: after,
        })
    }
}
/// Conservative final readback. A replacement listener on the old endpoint also
/// prevents success, even if its PID is different. Never terminates that owner.
pub fn listener_exit_readback(original: &OwnedListeners) -> io::Result<Vec<Listener>> {
    Ok(tcp_listeners()?
        .into_iter()
        .filter(|row| {
            original.listeners.iter().any(|(old, _)| {
                old.family == row.family
                    && old.address == row.address
                    && old.scope_id == row.scope_id
                    && old.port == row.port
            })
        })
        .collect())
}
