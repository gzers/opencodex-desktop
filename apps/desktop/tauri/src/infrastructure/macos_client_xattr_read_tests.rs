//! Fresh complete native xattrs; no metadata cache or source mutation permit.
use super::*;
use std::{
    fs,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    time::Instant,
};
fn fail(errno: i32) -> isize {
    unsafe {
        *libc::__error() = errno;
    }
    -1
}
#[test]
fn bounded_xattr_read_requires_complete_lengths_and_bounded_nonretrying_fallback() {
    for length in [0, 1, 256, 257, 4096, MAX_SECURITY] {
        let raw = vec![b'x'; length];
        let calls = std::cell::Cell::new(0);
        let result = bounded_xattr_read(MAX_SECURITY, |buffer, capacity| {
            calls.set(calls.get() + 1);
            if buffer.is_null() {
                return raw.len() as isize;
            }
            if capacity < raw.len() {
                return fail(libc::ERANGE);
            }
            unsafe {
                std::ptr::copy_nonoverlapping(raw.as_ptr(), buffer.cast(), raw.len());
            }
            raw.len() as isize
        })
        .unwrap();
        assert_eq!(result, raw);
        assert_eq!(calls.get(), if length < 256 { 1 } else { 3 });
    }
    for errno in [libc::EACCES, libc::EPERM, libc::ENOATTR, libc::EIO] {
        let calls = std::cell::Cell::new(0);
        let result = bounded_xattr_read(MAX_SECURITY, |_, _| {
            calls.set(calls.get() + 1);
            fail(errno)
        });
        assert_eq!(result.unwrap_err().raw_os_error(), Some(errno));
        assert_eq!(calls.get(), 1);
    }
    for mode in [
        "oversize",
        "query-error",
        "read-error",
        "shrunk",
        "grew",
        "empty",
        "impossible",
    ] {
        let calls = std::cell::Cell::new(0);
        let result = bounded_xattr_read(MAX_SECURITY, |buffer, capacity| {
            let n = calls.get() + 1;
            calls.set(n);
            match n {
                1 => {
                    if mode == "impossible" {
                        return capacity as isize + 1;
                    }
                    fail(libc::ERANGE)
                }
                2 => {
                    assert!(buffer.is_null() && capacity == 0);
                    match mode {
                        "oversize" => MAX_SECURITY as isize + 1,
                        "query-error" => fail(libc::EIO),
                        "empty" => 0,
                        _ => 512,
                    }
                }
                3 => {
                    assert!(!buffer.is_null());
                    match mode {
                        "empty" => {
                            assert_eq!(capacity, 1);
                            0
                        }
                        "shrunk" => 511,
                        "grew" => fail(libc::ERANGE),
                        _ => fail(libc::EACCES),
                    }
                }
                _ => panic!("unbounded retry"),
            }
        });
        assert_eq!(result.is_ok(), mode == "empty", "{mode}");
        assert_eq!(
            calls.get(),
            if mode == "oversize" || mode == "query-error" {
                2
            } else if mode == "impossible" {
                1
            } else {
                3
            }
        );
    }
    let calls = std::cell::Cell::new(0);
    assert!(bounded_xattr_read(12, |_, capacity| {
        calls.set(calls.get() + 1);
        if capacity == 12 {
            fail(libc::ERANGE)
        } else {
            13
        }
    })
    .is_err());
    assert_eq!(calls.get(), 2);
    assert_eq!(
        bounded_xattr_read(0, |buffer, size| {
            assert!(!buffer.is_null() && size == 1);
            0
        })
        .unwrap(),
        b""
    );
    assert!(bounded_xattr_read(0, |_, _| 1).is_err());
}
fn set(file: &File, name: &str, bytes: &[u8]) {
    let name = CString::new(name).unwrap();
    assert_eq!(
        unsafe {
            libc::fsetxattr(
                file.as_raw_fd(),
                name.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len(),
                0,
                0,
            )
        },
        0
    );
}
#[test]
fn bounded_xattr_resource_fork_retains_tail_and_refuses_aggregate_overflow() {
    let temp = tempfile::tempdir().unwrap();
    let file = fs::OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(temp.path().join("owned-resource-fork"))
        .unwrap();
    let key = b"com.apple.ResourceFork".as_slice();
    for length in [1, 255, 256, 257, 1024, 4096] {
        let mut bytes = vec![b'r'; length];
        bytes[length - 1] = b'x';
        set(&file, "com.apple.ResourceFork", &bytes);
        let first = xattrs(&file).unwrap();
        assert_eq!(first, xattrs_sized_reference(&file).unwrap());
        assert_eq!(first.get(key).unwrap(), &bytes);
        bytes[length - 1] = b'y';
        set(&file, "com.apple.ResourceFork", &bytes);
        let second = xattrs(&file).unwrap();
        assert_eq!(second.get(key).unwrap(), &bytes);
        assert_ne!(first, second, "tail beyond small buffer must be observed");
        assert_eq!(second, xattrs_sized_reference(&file).unwrap());
    }
    let too_large = vec![b'z'; MAX_SECURITY + 1];
    set(&file, "com.apple.ResourceFork", &too_large);
    assert!(xattrs(&file).is_err());
    assert!(xattrs_sized_reference(&file).is_err());
    let name = CString::new("com.apple.ResourceFork").unwrap();
    assert_eq!(
        get_xattr(file.as_raw_fd(), &name, std::ptr::null_mut(), 0),
        too_large.len() as isize
    );
}
#[test]
fn bounded_xattr_filled_buffer_and_stream_read_need_size_certification() {
    // Model a stream API returning a successful prefix instead of ERANGE.
    let raw = vec![b'f'; 1024];
    let calls = std::cell::Cell::new(0);
    let result = bounded_xattr_read(MAX_SECURITY, |buffer, capacity| {
        calls.set(calls.get() + 1);
        if buffer.is_null() {
            return raw.len() as isize;
        }
        let count = capacity.min(raw.len());
        unsafe {
            std::ptr::copy_nonoverlapping(raw.as_ptr(), buffer.cast(), count);
        }
        count as isize
    })
    .unwrap();
    assert_eq!(result, raw);
    assert_eq!(calls.get(), 3);
    let calls = std::cell::Cell::new(0);
    let result = sized_xattr_read(512, |buffer, _| {
        calls.set(calls.get() + 1);
        assert!(
            buffer.is_null(),
            "over-limit fork must refuse before data read"
        );
        1024
    });
    assert!(result.is_err());
    assert_eq!(calls.get(), 1);
}
#[test]
fn bounded_xattr_native_matches_all_original_bytes_and_aggregate_limit_without_repair() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("owned");
    let file = fs::OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(&path)
        .unwrap();
    for length in [0, 1, 256, 257, 4096] {
        let bytes = vec![b'v'; length];
        set(&file, "user.ocxd-xattr", &bytes);
        let direct = xattrs(&file).unwrap();
        let reference = xattrs_sized_reference(&file).unwrap();
        assert_eq!(direct, reference);
        assert_eq!(direct.get(b"user.ocxd-xattr".as_slice()).unwrap(), &bytes);
    }
    for i in 0..12 {
        set(
            &file,
            &format!("user.ocxd-names-long-{i:03}-abcdefghijklmnopqrstuvwxyz"),
            b"x",
        );
    }
    assert_eq!(
        xattrs(&file).unwrap(),
        xattrs_sized_reference(&file).unwrap()
    );
    let large = vec![b'z'; 48 * 1024];
    for i in 0..6 {
        set(&file, &format!("user.ocxd-large-{i}"), &large);
    }
    assert!(xattrs(&file).is_err() && xattrs_sized_reference(&file).is_err());
    let k = CString::new("user.ocxd-large-0").unwrap();
    assert_eq!(
        unsafe { libc::fgetxattr(file.as_raw_fd(), k.as_ptr(), std::ptr::null_mut(), 0, 0, 0) },
        large.len() as isize
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}
#[test]
fn bounded_xattr_small_native_reads_reduce_calls_but_reobserve_every_invocation() {
    let temp = tempfile::tempdir().unwrap();
    let file = fs::OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(temp.path().join("owned"))
        .unwrap();
    set(&file, "user.ocxd-small", b"fresh");
    let mut rows = Vec::new();
    for reference in [true, false, false, true] {
        let start = Instant::now();
        let (observed, metrics) = observation_metrics::measure(|| {
            if reference {
                observation_metrics::xattr_reference(|| xattrs(&file))
            } else {
                xattrs(&file)
            }
        });
        assert_eq!(
            observed
                .unwrap()
                .get(b"user.ocxd-small".as_slice())
                .unwrap(),
            b"fresh"
        );
        rows.push(serde_json::json!({"reference":reference,"elapsedNanos":start.elapsed().as_nanos(),"metrics":metrics}));
    }
    assert!(
        rows[1]["metrics"]["xattr_native_call"]["calls"]
            .as_u64()
            .unwrap()
            < rows[0]["metrics"]["xattr_native_call"]["calls"]
                .as_u64()
                .unwrap()
    );
    set(&file, "user.ocxd-small", b"changed");
    assert_eq!(
        xattrs(&file)
            .unwrap()
            .get(b"user.ocxd-small".as_slice())
            .unwrap(),
        b"changed"
    );
    println!(
        "XATTR_SMALL_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":rows,"scope":"owned native file, same whole xattrs, no cache or Stop permission"})
    );
}
