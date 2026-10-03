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
}
