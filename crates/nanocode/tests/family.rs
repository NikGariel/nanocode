use nanocode::{
    family::{self, FORMATS},
    palette::{self, Palette},
    vision::scan,
    Error,
};
#[test]
fn boundaries_and_ecc() {
    for f in FORMATS {
        assert_eq!(f.counts.iter().sum::<usize>(), f.cells());
        assert!(family::encode(&vec![0; f.capacity + 1], f).is_err());
        for n in 0..=f.capacity {
            let data: Vec<u8> = (0..n).map(|i| (i * 37) as u8).collect();
            let c = family::encode(&data, f).unwrap();
            assert_eq!(family::decode(&c, f).unwrap().0, data);
        }
        let data = vec![42; f.capacity];
        let c = family::encode(&data, f).unwrap();
        for i in 0..c.len() {
            let mut bad = c.clone();
            bad[i] ^= 1;
            assert_eq!(family::decode(&bad, f).unwrap(), (data.clone(), 1));
        }
    }
    for (n, id) in [
        (0, 1),
        (8, 1),
        (9, 2),
        (16, 2),
        (17, 3),
        (24, 3),
        (25, 4),
        (40, 4),
    ] {
        assert_eq!(family::select(n, 0).unwrap().id, id);
    }
    assert!(matches!(family::select(41, 0), Err(Error::TooLong)));
}
#[test]
fn palettes() {
    assert_eq!(Palette::default().assess().flags, 0);
    assert_ne!(
        Palette {
            one: 0x101010,
            zero: 0x111111,
            background: 0xffffff
        }
        .assess()
        .flags,
        0
    );
    assert!(palette::backgrounds(0x101010, 0x111111).is_empty());
    assert!(palette::backgrounds(0x00a6a1, 0x77dddd)
        .iter()
        .all(|(_, a)| a.flags == 0));
    assert_ne!(
        Palette {
            one: 0x1000000,
            ..Palette::default()
        }
        .assess()
        .flags,
        0
    );
}
#[test]
fn automatic_detection_all_formats_and_polarities() {
    let palettes = [
        Palette::default(),
        Palette {
            one: 0xdddddd,
            zero: 0x777777,
            background: 0x101010,
        },
        Palette {
            one: 0x222222,
            zero: 0xeeeeee,
            background: 0x888888,
        },
        Palette {
            one: 0xcc4466,
            zero: 0xffcccc,
            background: 0x161616,
        },
    ];
    for f in FORMATS {
        for p in palettes {
            let data: Vec<_> = (0..f.capacity).map(|i| i as u8 * 3).collect();
            let image = family::render_gray(&data, 512, f.id, p).unwrap();
            let d = scan(&image, 512, 512).unwrap_or_else(|e| panic!("{} {:?}: {e:?}", f.name, p));
            assert_eq!(d.format_id, f.id);
            assert_eq!(d.payload, data);
        }
    }
}

#[test]
fn transformed_profiles() {
    for f in FORMATS {
        let data = vec![b'A'; f.capacity];
        let runs = family::arcs(&family::encode(&data, f).unwrap(), f);
        let size = 640;
        let mut image = vec![0; size * size];
        let a = 0.71f64;
        let co = a.cos();
        let si = a.sin();
        let p = Palette {
            one: 0xdddddd,
            zero: 0x666666,
            background: 0x101010,
        };
        let levels = p.levels();
        let mut seed = 17u32;
        // Inverse mapping from translated, rotated, perspective image to canonical code.
        for y in 0..size {
            for x in 0..size {
                let u = (x as f64 - 330.) / 260.;
                let v = (y as f64 - 305.) / 240.;
                let z = 1. + 0.04 * u - 0.03 * v;
                let xx = u / z;
                let yy = v / z;
                let value = family::sample(&runs, f, levels, xx * co + yy * si, -xx * si + yy * co);
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                image[y * size + x] = (value as i32 + (seed >> 29) as i32 - 4).clamp(0, 255) as u8;
            }
        }
        let result = scan(&image, size, size).unwrap_or_else(|e| panic!("{}: {e:?}", f.name));
        assert_eq!(result.payload, data);
        assert_eq!(result.format_id, f.id);
    }
}

#[test]
fn central_brand_marks() {
    for f in FORMATS {
        for dark in [false, true] {
            let p = Palette {
                background: if dark { 0x101010 } else { 0xffffff },
                ..Palette::default()
            };
            let size = 512;
            let mut image = family::render_gray(b"BRAND001", size, f.id, p).unwrap();
            for y in 210..302 {
                for x in 210..302 {
                    let dx = x as i32 - 256;
                    let dy = y as i32 - 256;
                    if dx * dx + dy * dy < 44 * 44 {
                        image[y * size + x] = if x % 18 < 9 { 0 } else { 245 };
                    }
                }
            }
            let d = scan(&image, size, size)
                .unwrap_or_else(|e| panic!("{} logo dark={dark}: {e:?}", f.name));
            assert_eq!(d.payload, b"BRAND001");
        }
    }
}
