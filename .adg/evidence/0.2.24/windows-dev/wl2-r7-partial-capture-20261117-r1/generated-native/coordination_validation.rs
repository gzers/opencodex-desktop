use std::io;
fn refused()->io::Error{io::ErrorKind::InvalidData.into()}
const MAX:u64=2*1024*1024;
pub(super) fn validate_stable_bytes(raw: &[u8]) -> io::Result<()> {
    if raw.len() > MAX as usize {
        return Err(refused());
    }
    if !raw.is_empty() {
        if raw.len() < 100 || &raw[..16] != b"SQLite format 3\0" || raw[18] != 1 || raw[19] != 1 {
            return Err(refused());
        }
        let page = u16::from_be_bytes([raw[16], raw[17]]) as u32;
        if page != 1 && (!(512..=32768).contains(&page) || !page.is_power_of_two()) {
            return Err(refused());
        }
    }
    Ok(())
}
