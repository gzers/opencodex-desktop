//! Verify the independently selected Windows tray artwork at notification sizes.
use std::io::Cursor;

#[test]
fn windows_tray_is_monochrome_transparent_and_readable_at_small_sizes() {
    for (name, color) in [("light", [255, 255, 255]), ("dark", [26, 41, 72])] {
        for size in [16_u32, 20, 24, 32, 40, 48, 64] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("icons/windows")
                .join(format!("tray-{name}-{size}.png"));
            let bytes = std::fs::read(path).unwrap();
            let decoder = png::Decoder::new(Cursor::new(bytes));
            let mut reader = decoder.read_info().unwrap();
            let mut pixels = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut pixels).unwrap();
            assert_eq!((info.width, info.height), (size, size));
            assert_eq!(info.color_type, png::ColorType::Rgba);
            let (mut x0, mut y0, mut x1, mut y1) = (size, size, 0, 0);
            let mut transparent_inside = 0;
            for y in 0..size {
                for x in 0..size {
                    let pixel = &pixels[((y * size + x) * 4) as usize..][..4];
                    if pixel[3] > 16 {
                        assert_eq!(pixel[..3], color, "tray foreground is a single color");
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x + 1);
                        y1 = y1.max(y + 1);
                    } else if x > size / 4 && x < size * 3 / 4 && y > size / 4 && y < size * 3 / 4 {
                        transparent_inside += 1;
                    }
                }
            }
            assert!((x1 - x0) * 100 / size >= 85);
            assert!((y1 - y0) * 100 / size >= 85);
            assert!(x0 > 0 && y0 > 0 && x1 < size && y1 < size);
            assert!(
                transparent_inside > 0,
                "the aggregation cutout remains open"
            );
        }
    }
}
