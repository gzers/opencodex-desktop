//! FZ-27 / FZ-29 使用的 SHA-256 摘要原语。

use sha2::{Digest, Sha256};

pub const SHA256_HEX_LENGTH: usize = 64;

/// 计算字节序列的小写十六进制 SHA-256。
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_lower(&hasher.finalize())
}

/// 计算文件的小写十六进制 SHA-256。
pub fn sha256_file(path: &std::path::Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = std::io::Read::read(&mut file, &mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from_digit((byte >> 4) as u32, 16).unwrap_or('0'));
        output.push(char::from_digit((byte & 0x0f) as u32, 16).unwrap_or('0'));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(sha256_hex(b"abc").len(), SHA256_HEX_LENGTH);
    }

    #[test]
    fn sha256_file_matches_memory_digest() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let path = temp.path().join("payload.bin");
        std::fs::write(&path, b"controlled fixture").expect("write fixture");
        assert_eq!(
            sha256_file(&path).expect("hash file"),
            sha256_hex(b"controlled fixture")
        );
    }

    #[test]
    fn streaming_sha256_matches_independent_large_input_vector() {
        // Python hashlib SHA-256 of exactly 256 MiB of ASCII 'a'. This exercises
        // the same chunked Digest interface as full program-image admission.
        let chunk = [b'a'; 64 * 1024];
        let mut hash = Sha256::new();
        for _ in 0..4096 {
            hash.update(chunk);
        }
        assert_eq!(
            hex_lower(&hash.finalize()),
            "b4a0226ee3f9b159ac06a86332dca0d90a04adef7f88934aa2a75be2a011d504"
        );
    }

    #[test]
    fn hmac_sha256_matches_rfc4231_and_refuses_changed_authentication() {
        use hmac::{Hmac, Mac};
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&[0x0b; 20]).unwrap();
        mac.update(b"Hi There");
        let bytes = mac.clone().finalize().into_bytes();
        assert_eq!(
            hex_lower(&bytes),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        mac.clone().verify_slice(&bytes).unwrap();
        let mut changed = bytes;
        changed[0] ^= 1;
        assert!(mac.clone().verify_slice(&changed).is_err());
        mac.update(b"changed payload");
        assert!(mac.verify_slice(&bytes).is_err());
    }
}
