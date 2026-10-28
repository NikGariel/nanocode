use nanocode::{family, palette::Palette, vision::scan};
fn screen_scene(format: u8, dark: bool) -> (Vec<u8>, usize, usize) {
    let (w, h) = (960, 640);
    let mut scene = vec![44u8; w * h];
    // UI panels, text-like bars and a bright unrelated rectangle, around a code tile.
    for y in 20..620 {
        for x in 15..320 {
            scene[y * w + x] = if y % 28 < 9 && x < 280 { 190 } else { 31 };
        }
    }
    for y in 25..90 {
        for x in 340..940 {
            scene[y * w + x] = 220;
        }
    }
    for y in 550..620 {
        for x in 400..920 {
            scene[y * w + x] = 110;
        }
    }
    let palette = if dark {
        Palette {
            background: 0x101010,
            ..Palette::default()
        }
    } else {
        Palette::default()
    };
    let code = family::render_gray(b"ID000123", 384, format, palette).unwrap();
    for y in 0..384 {
        for x in 0..384 {
            scene[(y + 130) * w + x + 445] = code[y * 384 + x];
        }
    }
    (scene, w, h)
}
#[test]
fn code_inside_monitor_ui() {
    for format in 1..=4 {
        for dark in [false, true] {
            let (scene, w, h) = screen_scene(format, dark);
            let decoded =
                scan(&scene, w, h).unwrap_or_else(|e| panic!("format {format} dark {dark}: {e:?}"));
            assert_eq!(decoded.payload, b"ID000123");
            assert_eq!(decoded.format_id, format);
            assert!(decoded.corners.iter().all(|p| p.x > 445. && p.y > 130.));
        }
    }
}
#[test]
fn screen_with_blur_and_uneven_lighting() {
    for dark in [false, true] {
        let (mut scene, w, h) = screen_scene(2, dark);
        let original = scene.clone();
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let mut sum = 0u32;
                for yy in y - 1..=y + 1 {
                    for xx in x - 1..=x + 1 {
                        sum += original[yy * w + xx] as u32;
                    }
                }
                let shade = (x as i32 - 600) / 24;
                scene[y * w + x] = (sum as i32 / 9 + shade).clamp(0, 255) as u8;
            }
        }
        assert_eq!(
            scan(&scene, w, h)
                .unwrap_or_else(|e| panic!("dark={dark}: {e:?}"))
                .payload,
            b"ID000123"
        );
    }
}
