//! Native alternate-stream admission shared by product and read-only collector.
use std::{io, path::Path};
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "original phase chain unknown; preserve pending custody/Required",
    )
}
pub(in crate::infrastructure::windows_client_originals024) fn streams(
    path: &Path,
    dir: bool,
) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Foundation::INVALID_HANDLE_VALUE,
        Storage::FileSystem::{
            FindClose, FindFirstStreamW, FindNextStreamW, FindStreamInfoStandard,
            WIN32_FIND_STREAM_DATA,
        },
    };
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut data = WIN32_FIND_STREAM_DATA::default();
    let h = unsafe {
        FindFirstStreamW(
            name.as_ptr(),
            FindStreamInfoStandard,
            (&mut data as *mut WIN32_FIND_STREAM_DATA).cast(),
            0,
        )
    };
    if h == INVALID_HANDLE_VALUE {
        let e = io::Error::last_os_error();
        return if dir && e.raw_os_error() == Some(38) {
            Ok(())
        } else {
            Err(e)
        };
    }
    struct Search(windows_sys::Win32::Foundation::HANDLE);
    impl Drop for Search {
        fn drop(&mut self) {
            unsafe {
                FindClose(self.0);
            }
        }
    }
    let _search = Search(h);
    let n = data
        .cStreamName
        .iter()
        .position(|c| *c == 0)
        .ok_or_else(invalid)?;
    if String::from_utf16(&data.cStreamName[..n]).map_err(|_| invalid())? != "::$DATA" {
        return Err(invalid());
    }
    if unsafe { FindNextStreamW(h, (&mut data as *mut WIN32_FIND_STREAM_DATA).cast()) } != 0
        || io::Error::last_os_error().raw_os_error() != Some(38)
    {
        return Err(invalid());
    }
    Ok(())
}
