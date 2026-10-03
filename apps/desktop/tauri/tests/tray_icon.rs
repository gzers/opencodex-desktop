//! 托盘图标素材回归：防止「有效图形」再次被大透明画布稀释。
//!
//! 背景：macOS 菜单栏按图片尺寸缩放，若 36×36 画布里的实际图形只占 10×10，
//! 菜单栏上就会显示成一个几乎看不见的小点。品牌交付目录里的
//! `tray-template@2x.png` 正是这种「大画布 + 小图形」，直接拿来当应用托盘图标会回归。
//! 因此这里断言应用实际使用的托盘素材的图形覆盖率。

use std::fs::File;
use std::path::Path;

/// 图形边长至少占画布的 80%（36px 画布 ≈ 29px 图形）。
const MIN_GLYPH_COVERAGE_PERCENT: u32 = 80;
/// Retina 菜单栏需要 2x 素材：画布至少 32px。
const MIN_CANVAS_PIXELS: u32 = 32;

fn glyph_bbox(path: &Path) -> (u32, u32, u32, u32) {
    let decoder = png::Decoder::new(File::open(path).expect("open tray icon"));
    let mut reader = decoder.read_info().expect("read png info");
    let mut buffer = vec![0_u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).expect("decode png frame");
    assert_eq!(info.color_type, png::ColorType::Rgba, "托盘素材必须是 RGBA");
    assert_eq!(info.bit_depth, png::BitDepth::Eight, "托盘素材必须是 8 位");

    let (width, height) = (info.width, info.height);
    let (mut min_x, mut min_y) = (width, height);
    let (mut max_x, mut max_y) = (0_u32, 0_u32);
    let mut found = false;
    for y in 0..height {
        for x in 0..width {
            let alpha = buffer[((y * width + x) * 4 + 3) as usize];
            if alpha > 16 {
                found = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    assert!(found, "{} 没有任何不透明像素", path.display());
    (width, max_x - min_x + 1, height, max_y - min_y + 1)
}

#[test]
fn tray_icons_fill_their_canvas() {
    for name in ["tray-16.png", "tray-32.png"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("icons")
            .join(name);
        let (canvas_w, glyph_w, canvas_h, glyph_h) = glyph_bbox(&path);
        assert!(
            canvas_w >= MIN_CANVAS_PIXELS && canvas_h >= MIN_CANVAS_PIXELS,
            "{name} 画布 {canvas_w}×{canvas_h} 小于 {MIN_CANVAS_PIXELS}px，Retina 下会模糊"
        );
        let coverage_w = glyph_w * 100 / canvas_w;
        let coverage_h = glyph_h * 100 / canvas_h;
        assert!(
            coverage_w >= MIN_GLYPH_COVERAGE_PERCENT && coverage_h >= MIN_GLYPH_COVERAGE_PERCENT,
            "{name} 有效图形仅 {glyph_w}×{glyph_h}（画布 {canvas_w}×{canvas_h}，覆盖率 \
             {coverage_w}%×{coverage_h}%）；大透明画布会把菜单栏图标显示得过小"
        );
    }
}
