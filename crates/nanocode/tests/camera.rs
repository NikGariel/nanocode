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

#[test]
fn monitor_exposure_gradient_and_refresh_bands() {
    for format in 1..=4 {
        let size = 384;
        let base = family::render_gray(b"ID000123", size, format, Palette::default()).unwrap();
        for bands in [false, true] {
            let mut frame = base.clone();
            for y in 0..size {
                for x in 0..size {
                    let gain = if bands {
                        0.72 + 0.20 * (y as f64 / 13.).sin()
                    } else {
                        0.45 + 0.55 * x as f64 / (size - 1) as f64
                    };
                    frame[y * size + x] = (base[y * size + x] as f64 * gain).round() as u8;
                }
            }
            let original = frame.clone();
            for y in 1..size - 1 {
                for x in 1..size - 1 {
                    let mut sum = 0u32;
                    for yy in y - 1..=y + 1 {
                        for xx in x - 1..=x + 1 {
                            sum += original[yy * size + xx] as u32;
                        }
                    }
                    frame[y * size + x] = (sum / 9) as u8;
                }
            }
            let found = scan(&frame, size, size)
                .unwrap_or_else(|e| panic!("format={format} bands={bands}: {e:?}"));
            assert_eq!(found.payload, b"ID000123");
            assert_eq!(found.format_id, format);
        }
    }
}

#[test]
fn actual_landing_n_logo() {
    let polygon = [
        (-0.13, 0.12),
        (-0.13, -0.12),
        (-0.075, -0.12),
        (0.075, 0.055),
        (0.075, -0.12),
        (0.13, -0.12),
        (0.13, 0.12),
        (0.075, 0.12),
        (-0.075, -0.055),
        (-0.075, 0.12),
    ];
    for size in [256, 384, 512] {
        let mut image = family::render_gray(b"ORBIT:EVT:042", size, 2, Palette::default()).unwrap();
        for y in 0..size {
            for x in 0..size {
                let px = 2. * x as f64 / size as f64 - 1.;
                let py = 2. * y as f64 / size as f64 - 1.;
                let mut inside = false;
                for i in 0..polygon.len() {
                    let (a, b) = polygon[i];
                    let (c, d) = polygon[(i + polygon.len() - 1) % polygon.len()];
                    if (b > py) != (d > py) && px < (c - a) * (py - b) / (d - b) + a {
                        inside = !inside;
                    }
                }
                if inside {
                    image[y * size + x] = 46;
                }
            }
        }
        for bands in [false, true] {
            let mut frame = image.clone();
            if bands {
                for y in 0..size {
                    for x in 0..size {
                        frame[y * size + x] = (image[y * size + x] as f64
                            * (0.72 + 0.20 * (y as f64 / 13.).sin()))
                        .round() as u8;
                    }
                }
                let source = frame.clone();
                for y in 0..size {
                    for x in 0..size {
                        let mut sum = 0u32;
                        for dy in -1isize..=1 {
                            for dx in -1isize..=1 {
                                let yy = (y as isize + dy).clamp(0, size as isize - 1) as usize;
                                let xx = (x as isize + dx).clamp(0, size as isize - 1) as usize;
                                sum += source[yy * size + xx] as u32;
                            }
                        }
                        frame[y * size + x] = ((sum + 4) / 9) as u8;
                    }
                }
            }
            let d = scan(&frame, size, size)
                .unwrap_or_else(|e| panic!("landing N size={size} bands={bands}: {e:?}"));
            assert_eq!(d.payload, b"ORBIT:EVT:042");
        }
    }
}
