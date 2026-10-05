//! 覆盖实际用于 Windows EXE 的 ICO，防止仅更新 PNG 后快捷方式仍显示小点。
use std::io::Cursor;

fn u32_at(bytes: &[u8], offset: usize) -> usize {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize
}

#[test]
fn windows_ico_has_all_sizes_and_visible_artwork_fills_each_frame() {
    let bytes = include_bytes!("../icons/icon.ico");
    assert_eq!(&bytes[..4], &[0, 0, 1, 0]);
    let count = u16::from_le_bytes(bytes[4..6].try_into().unwrap()) as usize;
    let mut sizes = Vec::new();
    for index in 0..count {
        let offset = 6 + index * 16;
        let width = if bytes[offset] == 0 {
            256
        } else {
            bytes[offset] as u32
        };
        let height = if bytes[offset + 1] == 0 {
            256
        } else {
            bytes[offset + 1] as u32
        };
        assert_eq!(width, height);
        sizes.push(width);
        let size = u32_at(bytes, offset + 8);
        let start = u32_at(bytes, offset + 12);
        let decoder = png::Decoder::new(Cursor::new(&bytes[start..start + size]));
        let mut reader = decoder.read_info().expect("ICO frames use lossless PNG");
        let mut buffer = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buffer).unwrap();
        assert_eq!((info.width, info.height), (width, height));
        assert_eq!(info.color_type, png::ColorType::Rgba);
        let mut x0 = width;
        let mut y0 = height;
        let mut x1 = 0;
        let mut y1 = 0;
        for y in 0..height {
            for x in 0..width {
                if buffer[((y * width + x) * 4 + 3) as usize] > 16 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        assert!(
            (x1 - x0) * 100 / width >= 80,
            "{width}px icon artwork is too narrow"
        );
        assert!(
            (y1 - y0) * 100 / height >= 80,
            "{width}px icon artwork is too short"
        );
    }
    sizes.sort_unstable();
    assert_eq!(sizes, [16, 24, 32, 48, 64, 128, 256]);
}
