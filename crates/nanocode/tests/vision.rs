use nanocode::{
    arc_pixel, arcs, encode, render_gray,
    vision::{homography, project, scan, Point},
};
#[test]
fn generated_images() {
    for size in [256, 384, 768] {
        let g = render_gray(b"00NanoCode", size).unwrap();
        let d = scan(&g, size, size).unwrap();
        assert_eq!(d.payload, b"00NanoCode");
    }
}
#[test]
fn reject_blank_noise_and_invalid() {
    assert!(scan(&vec![255; 256 * 256], 256, 256).is_err());
    let mut seed = 1u32;
    let noise: Vec<u8> = (0..256 * 256)
        .map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 24) as u8
        })
        .collect();
    assert!(scan(&noise, 256, 256).is_err());
    assert!(scan(&[], usize::MAX, 3).is_err());
}
// Independent inverse projective renderer: geometry, rotation, translation and luminance noise.
fn scene(corners: [Point; 4], size: usize, noise: bool) -> Vec<u8> {
    let h = homography(corners).unwrap();
    let c = arcs(&encode("Привет".as_bytes()).unwrap());
    let mut image = vec![245; size * size];
    let mut seed = 8u32;
    for y in 0..size {
        for x in 0..size {
            let u = x as f64;
            let v = y as f64;
            let a = h[0] - u * h[6];
            let b = h[1] - u * h[7];
            let d = h[3] - v * h[6];
            let e = h[4] - v * h[7];
            let c0 = u - h[2];
            let f = v - h[5];
            let det = a * e - b * d;
            let px = (c0 * e - b * f) / det;
            let py = (a * f - c0 * d) / det;
            let value = arc_pixel(&c, px, py);
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let jitter = if noise { (seed >> 27) as i32 - 16 } else { 0 };
            image[y * size + x] = (value as i32 * 3 / 4 + 25 + jitter).clamp(0, 255) as u8;
        }
    }
    image
}
#[test]
fn rotation_translation_perspective_noise() {
    for angle in [0.0f64, 0.37, 1.57, 2.8, 4.7] {
        let h = [
            220. * angle.cos(),
            -220. * angle.sin(),
            320.,
            220. * angle.sin(),
            220. * angle.cos(),
            285.,
            0.08,
            -0.06,
        ];
        let corners = nanocode::REFERENCE_CORNERS.map(|(x, y)| project(&h, x, y));
        let image = scene(corners, 640, true);
        let d = scan(&image, 640, 640).unwrap_or_else(|e| panic!("angle {angle}: {e:?}"));
        assert_eq!(d.payload, "Привет".as_bytes());
    }
}

#[test]
fn mild_blur_and_occlusion() {
    let size = 512;
    let g = render_gray(b"blur", size).unwrap();
    let mut blurred = g.clone();
    for y in 1..size - 1 {
        for x in 1..size - 1 {
            let mut sum = 0u32;
            for yy in y - 1..=y + 1 {
                for xx in x - 1..=x + 1 {
                    sum += g[yy * size + xx] as u32;
                }
            }
            blurred[y * size + x] = (sum / 9) as u8;
        }
    }
    assert_eq!(scan(&blurred, size, size).unwrap().payload, b"blur");
    for y in 20..256 {
        for x in 20..256 {
            blurred[y * size + x] = 255;
        }
    }
    assert!(scan(&blurred, size, size).is_err());
}

#[test]
fn payload_and_density_variations() {
    let mut cases = vec![
        vec![],
        vec![0; 24],
        vec![255; 24],
        b"0123456789abcdefghijklmn".to_vec(),
    ];
    let mut seed = 7u32;
    for n in [1, 8, 16, 24] {
        cases.push(
            (0..n)
                .map(|_| {
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    (seed >> 24) as u8
                })
                .collect(),
        );
    }
    for data in cases {
        let g = render_gray(&data, 384).unwrap();
        assert_eq!(
            scan(&g, 384, 384)
                .unwrap_or_else(|e| panic!("payload {data:?}: {e:?}"))
                .payload,
            data
        );
    }
}
