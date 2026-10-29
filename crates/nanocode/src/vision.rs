//! Markerless experimental reader: fit the concentric ring field, then recover orientation.
use crate::{
    decode,
    family::{self, Format, FORMATS},
    Error, CELLS, REFERENCE_CORNERS,
};
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug)]
pub struct Detection {
    /// 0 for legacy v1; 1..4 for v2 S/M/L/XL.
    pub format_id: u8,
    pub payload: Vec<u8>,
    pub corners: [Point; 4],
    pub corrected: usize,
}
/// Projective mapping from canonical code coordinates into image pixels.
#[allow(clippy::needless_range_loop)] // Gaussian elimination uses explicit matrix indices.
pub fn homography(dst: [Point; 4]) -> Option<[f64; 8]> {
    let mut a = [[0.0; 9]; 8];
    for i in 0..4 {
        let (x, y) = REFERENCE_CORNERS[i];
        let u = dst[i].x;
        let v = dst[i].y;
        a[2 * i] = [x, y, 1., 0., 0., 0., -u * x, -u * y, u];
        a[2 * i + 1] = [0., 0., 0., x, y, 1., -v * x, -v * y, v];
    }
    for c in 0..8 {
        let p = (c..8).max_by(|&i, &j| a[i][c].abs().total_cmp(&a[j][c].abs()))?;
        if a[p][c].abs() < 1e-9 {
            return None;
        }
        a.swap(c, p);
        let v = a[c][c];
        for k in c..9 {
            a[c][k] /= v;
        }
        for r in 0..8 {
            if r != c {
                let f = a[r][c];
                for k in c..9 {
                    a[r][k] -= f * a[c][k];
                }
            }
        }
    }
    Some(std::array::from_fn(|i| a[i][8]))
}
pub fn project(h: &[f64; 8], x: f64, y: f64) -> Point {
    let z = h[6] * x + h[7] * y + 1.;
    Point {
        x: (h[0] * x + h[1] * y + h[2]) / z,
        y: (h[3] * x + h[4] * y + h[5]) / z,
    }
}
// Three luminance classes: both inks and the paper. Do not mistake light ink for paper.
fn levels(gray: &[u8]) -> Option<[f64; 3]> {
    let mut hist = [0usize; 256];
    for &b in gray {
        hist[b as usize] += 1;
    }
    let min = *gray.iter().min()? as f64;
    let max = *gray.iter().max()? as f64;
    let mut levels = [min, (min + max) * 0.5, max];
    for _ in 0..20 {
        let mut sums = [0.; 3];
        let mut ns = [0usize; 3];
        for (v, &n) in hist.iter().enumerate() {
            let k = (0..3).min_by(|&a, &b| {
                (v as f64 - levels[a])
                    .abs()
                    .total_cmp(&(v as f64 - levels[b]).abs())
            })?;
            sums[k] += v as f64 * n as f64;
            ns[k] += n;
        }
        for k in 0..3 {
            if ns[k] > 0 {
                levels[k] = sums[k] / ns[k] as f64;
            }
        }
    }
    if levels[1] - levels[0] < 20. || levels[2] - levels[1] < 20. {
        None
    } else {
        Some(levels)
    }
}
// Image -> canonical projective map, up to a rotation (which a circle cannot determine).
fn inverse(p: &[f64; 7], x: f64, y: f64) -> (f64, f64) {
    let u = (x - p[0]) / p[2];
    let v = (y - p[1]) / p[3];
    let z = 1. + p[5] * u + p[6] * v;
    ((u + p[4] * v) / z, v / z)
}
fn forward(p: &[f64; 7], x: f64, y: f64) -> Point {
    let z = 1. - p[5] * x - (p[6] - p[5] * p[4]) * y;
    Point {
        x: p[0] + p[2] * (x - p[4] * y) / z,
        y: p[1] + p[3] * y / z,
    }
}
fn cost(p: &[f64; 7], points: &[(f64, f64)], radii: &[f64]) -> f64 {
    if p[2] < 25. || p[3] < 25. || p[4].abs() > 1. || p[5].abs() > 0.5 || p[6].abs() > 0.5 {
        return f64::INFINITY;
    }
    let first = radii[0];
    let pitch = radii[1] - first;
    let last = (radii.len() - 1) as f64;
    points
        .iter()
        .map(|&(x, y)| {
            let (u, v) = inverse(p, x, y);
            let r = (u * u + v * v).sqrt();
            // All v1/v2 profiles have uniformly spaced radii. Constant-time nearest
            // ring avoids a linear search and the expensive portable hypot per point.
            let ring = ((r - first) / pitch).round().clamp(0., last);
            (r - (first + ring * pitch)).powi(2)
        })
        .sum::<f64>()
        / points.len() as f64
}
fn fit(
    mut p: [f64; 7],
    points: &[(f64, f64)],
    radii: &[f64],
    remaining: &mut usize,
) -> ([f64; 7], f64) {
    if *remaining == 0 {
        return (p, f64::INFINITY);
    }
    *remaining -= 1;
    let mut best = cost(&p, points, radii);
    let mut steps = [
        p[2] * 0.025,
        p[3] * 0.025,
        p[2] * 0.02,
        p[3] * 0.02,
        0.025,
        0.025,
        0.025,
    ];
    for _ in 0..12 {
        for _ in 0..12 {
            let mut improved = false;
            for k in 0..7 {
                let original = p[k];
                let mut value = original;
                for sign in [-1., 1.] {
                    p[k] = original + sign * steps[k];
                    if *remaining == 0 {
                        p[k] = value;
                        return (p, best);
                    }
                    *remaining -= 1;
                    let score = cost(&p, points, radii);
                    if score < best {
                        best = score;
                        value = p[k];
                        improved = true;
                    }
                }
                p[k] = value;
            }
            if !improved {
                break;
            }
        }
        for s in &mut steps {
            *s *= 0.5;
        }
    }
    (p, best)
}
/// Search complete-frame and bounded edge-derived regions, returning full-frame coordinates.
pub fn scan(gray: &[u8], width: usize, height: usize) -> Result<Detection, Error> {
    if !(32..=2048).contains(&width)
        || !(32..=2048).contains(&height)
        || width.checked_mul(height) != Some(gray.len())
    {
        return Err(Error::Invalid);
    }
    if let Ok(found) = scan_pass(gray, width, height, 14000, 6000, false) {
        return Ok(found);
    }
    let min = gray.iter().copied().min().unwrap_or(0);
    let max = gray.iter().copied().max().unwrap_or(0);
    if max.saturating_sub(min) < 35 {
        return Err(Error::NotFound);
    }
    // Flat-field the local background for monitor brightness gradients and banding.
    // Inverting first also handles a dark quiet zone. Every attempt still validates CRC.
    for invert in [false, true] {
        let corrected = crate::vision_preprocess::flatten(gray, width, height, invert);
        if let Ok(found) = scan_pass(&corrected, width, height, 4000, 2000, true) {
            return Ok(found);
        }
    }
    Err(Error::NotFound)
}
fn scan_pass(
    gray: &[u8],
    width: usize,
    height: usize,
    whole_budget: usize,
    mut remaining: usize,
    exclude_logo_first: bool,
) -> Result<Detection, Error> {
    // Preserve the fast path for clean generated images and a close-up camera view.
    if let Ok(found) = scan_region(
        gray,
        width,
        height,
        &mut { whole_budget },
        exclude_logo_first,
    ) {
        return Ok(found);
    }
    let mut correction_budget = if exclude_logo_first { 0 } else { 12000 };
    for region in crate::vision_regions::proposals(gray, width, height) {
        // An edge component may be the code's card border rather than its outer ring.
        // Retry just inside that border to exclude surrounding UI and its antialiasing.
        for inset in [0., 0.04, 0.065] {
            if remaining == 0 {
                return Err(Error::NotFound);
            }
            let dx = (region.width as f64 * inset) as usize;
            let dy = (region.height as f64 * inset) as usize;
            let x = region.x + dx;
            let y = region.y + dy;
            let w = region.width - 2 * dx;
            let h = region.height - 2 * dy;
            let mut crop = Vec::with_capacity(w * h);
            for row in y..y + h {
                crop.extend_from_slice(&gray[row * width + x..row * width + x + w]);
            }
            let mut detected = scan_region(&crop, w, h, &mut remaining, true);
            // Estimate illumination at the code's scale, not at the whole screen's scale.
            if detected.is_err() && correction_budget > 0 {
                for invert in [false, true] {
                    let corrected = crate::vision_preprocess::flatten(&crop, w, h, invert);
                    let limit = correction_budget.min(4000);
                    let mut available = limit;
                    detected = scan_region(&corrected, w, h, &mut available, true);
                    correction_budget -= limit - available;
                    if detected.is_ok() || correction_budget == 0 {
                        break;
                    }
                }
            }
            if let Ok(mut found) = detected {
                for p in &mut found.corners {
                    p.x += x as f64;
                    p.y += y as f64;
                }
                return Ok(found);
            }
        }
    }
    scan_region(gray, width, height, &mut remaining, true)
}
fn scan_region(
    gray: &[u8],
    width: usize,
    height: usize,
    remaining: &mut usize,
    exclude_logo: bool,
) -> Result<Detection, Error> {
    let l = levels(gray).ok_or(Error::NotFound)?;
    let classify = |v: u8| {
        if v as f64 <= (l[0] + l[1]) * 0.5 {
            0
        } else if v as f64 <= (l[1] + l[2]) * 0.5 {
            1
        } else {
            2
        }
    };
    let mut border = [0usize; 3];
    for x in 0..width {
        border[classify(gray[x])] += 1;
        border[classify(gray[(height - 1) * width + x])] += 1;
    }
    for y in 0..height {
        border[classify(gray[y * width])] += 1;
        border[classify(gray[y * width + width - 1])] += 1;
    }
    let background = (0..3).max_by_key(|&i| border[i]).unwrap();
    let is_ink = |v: u8| classify(v) != background;

    let (mut minx, mut miny, mut maxx, mut maxy) = (width, height, 0, 0);
    let mut count = 0;
    for (i, &v) in gray.iter().enumerate() {
        if is_ink(v) {
            let x = i % width;
            let y = i / width;
            minx = minx.min(x);
            maxx = maxx.max(x);
            miny = miny.min(y);
            maxy = maxy.max(y);
            count += 1;
        }
    }
    if count < 100 || count > gray.len() / 2 || maxx <= minx + 64 || maxy <= miny + 64 {
        return Err(Error::NotFound);
    }
    // A brand mark is decoration, not encoded ink. Exclude the safe centre both from
    // palette estimation and the concentric-ring fit, including high-contrast logos.
    let cx = (minx + maxx) as f64 / 2.;
    let cy = (miny + maxy) as f64 / 2.;
    let rx = (maxx - minx) as f64 / (2. * 0.93);
    let ry = (maxy - miny) as f64 / (2. * 0.93);
    let outside_logo = |i: usize| {
        if !exclude_logo {
            return true;
        }
        let x = ((i % width) as f64 - cx) / rx;
        let y = ((i / width) as f64 - cy) / ry;
        x * x + y * y > 0.32 * 0.32
    };
    let palette_pixels: Vec<u8> = if exclude_logo {
        gray.iter()
            .enumerate()
            .step_by((gray.len() / 50000).max(1))
            .filter(|(i, _)| outside_logo(*i))
            .map(|(_, v)| *v)
            .collect()
    } else {
        Vec::new()
    };
    let l = levels(&palette_pixels).unwrap_or(l);
    let classify = |v: u8| {
        if v as f64 <= (l[0] + l[1]) * 0.5 {
            0
        } else if v as f64 <= (l[1] + l[2]) * 0.5 {
            1
        } else {
            2
        }
    };
    let mut border = [0usize; 3];
    for x in 0..width {
        border[classify(gray[x])] += 1;
        border[classify(gray[(height - 1) * width + x])] += 1;
    }
    for y in 0..height {
        border[classify(gray[y * width])] += 1;
        border[classify(gray[y * width + width - 1])] += 1;
    }
    let background = (0..3).max_by_key(|&i| border[i]).unwrap();
    let inks: Vec<_> = (0..3).filter(|&i| i != background).collect();
    let bit_threshold = (l[inks[0]] + l[inks[1]]) / 2.;
    let is_ink = |v: u8| classify(v) != background;
    let mut points = Vec::new();
    let stride = (count / 1200).max(1);
    let mut index = 0;
    for (i, &v) in gray.iter().enumerate() {
        if is_ink(v) && outside_logo(i) {
            if index % stride == 0 {
                points.push(((i % width) as f64, (i / width) as f64));
            }
            index += 1;
        }
    }
    // Rank geometries using their initial radial fit, then validate the protected header.
    let mut candidates: Vec<_> = FORMATS
        .into_iter()
        .map(|format| {
            let extent = 0.91 + format.stroke / 2.;
            let initial = [
                (minx + maxx) as f64 / 2.,
                (miny + maxy) as f64 / 2.,
                (maxx - minx) as f64 / (2. * extent),
                (maxy - miny) as f64 / (2. * extent),
                0.,
                0.,
                0.,
            ];
            // UI edges and a blurred logo can distort the hull ellipse. Keep the
            // axis-aligned seed when it explains the ring pixels more accurately.
            let estimated = ellipse_seed(initial, &points, extent);
            let radii = format.radii();
            let initial = if cost(&estimated, &points, &radii) < cost(&initial, &points, &radii) {
                estimated
            } else {
                initial
            };
            (cost(&initial, &points, &radii), format, initial)
        })
        .collect();
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (coarse_error, format, initial) in candidates {
        if *remaining == 0 {
            break;
        }
        if coarse_error > 0.0025 || !ring_coverage(&initial, &points, &format.radii()) {
            continue;
        }
        let limit = (*remaining).min(3500);
        let mut available = limit;
        let found = scan_format(
            gray,
            width,
            height,
            bit_threshold,
            &points,
            format,
            initial,
            &mut available,
        );
        *remaining -= limit - available;
        if let Some(found) = found {
            return Ok(found);
        }
    }
    Err(Error::NotFound)
}
fn ring_coverage(p: &[f64; 7], points: &[(f64, f64)], radii: &[f64]) -> bool {
    let mut counts = vec![0usize; radii.len()];
    for &(x, y) in points {
        let (u, v) = inverse(p, x, y);
        let r = u.hypot(v);
        let nearest = (0..radii.len())
            .min_by(|&a, &b| (r - radii[a]).abs().total_cmp(&(r - radii[b]).abs()))
            .unwrap();
        counts[nearest] += 1;
    }
    counts.iter().all(|&n| n >= points.len() / radii.len() / 8)
}
#[allow(clippy::too_many_arguments)]
fn scan_format(
    gray: &[u8],
    width: usize,
    height: usize,
    bit_threshold: f64,
    points: &[(f64, f64)],
    format: Format,
    initial: [f64; 7],
    remaining: &mut usize,
) -> Option<Detection> {
    let radii = format.radii();
    let (mut p, mut score) = fit(initial, points, &radii, remaining);
    // Do not spend multiple projective restarts on text, partial codes and arbitrary UI edges.
    if score > format.stroke.powi(2) * 0.5 {
        return None;
    }

    if score > format.stroke.powi(2) / 7. {
        for shear in [-0.25, -0.125, 0.125, 0.25] {
            let mut seed = initial;
            seed[4] = shear;
            let (candidate, error) = fit(seed, points, &radii, remaining);
            if error < score {
                p = candidate;
                score = error;
            }
        }
    }
    if score > format.stroke.powi(2) / 7. {
        for px in [-0.10, 0.10] {
            for py in [-0.10, 0.10] {
                let mut seed = initial;
                seed[5] = px;
                seed[6] = py;
                let (candidate, error) = fit(seed, points, &radii, remaining);
                if error < score {
                    p = candidate;
                    score = error;
                }
            }
        }
    }
    // Blur widens the ink footprint beyond the ideal printed stroke. This is
    // only a proposal gate: header, ECC and CRC still decide acceptance.
    if score > format.stroke.powi(2) / 3. {
        return None;
    }
    decode_pose(gray, width, height, bit_threshold, format, &p)
}

// Bilinear sampling suppresses subpixel phase changes and screen-pixel moiré.
fn luminance_at(gray: &[u8], width: usize, height: usize, q: Point) -> Option<f64> {
    if !q.x.is_finite()
        || !q.y.is_finite()
        || q.x < 0.
        || q.y < 0.
        || q.x >= (width - 1) as f64
        || q.y >= (height - 1) as f64
    {
        return None;
    }
    let x = q.x as usize;
    let y = q.y as usize;
    let fx = q.x - x as f64;
    let fy = q.y - y as f64;
    let row = y * width + x;
    Some(
        (gray[row] as f64 * (1. - fx) + gray[row + 1] as f64 * fx) * (1. - fy)
            + (gray[row + width] as f64 * (1. - fx) + gray[row + width + 1] as f64 * fx) * fy,
    )
}

fn decode_pose(
    gray: &[u8],
    width: usize,
    height: usize,
    threshold: f64,
    format: Format,
    p: &[f64; 7],
) -> Option<Detection> {
    let samples = format.points();
    let mut cells = vec![0u8; format.cells()];
    for step in 0..720 {
        let angle = step as f64 * std::f64::consts::TAU / 720.;
        let (sin, cos) = angle.sin_cos();
        let sample = |i: usize| {
            let (x, y) = samples[i];
            luminance_at(
                gray,
                width,
                height,
                forward(p, x * cos - y * sin, x * sin + y * cos),
            )
            .map(|v| u8::from(v <= threshold))
        };
        // Read the protected magic before sampling hundreds of payload cells.
        // Both versions share the first two words and interleaving. Never accept
        // a header alone: the complete packet still has to pass ECC and CRC.
        let mut header = [0u8; 13];
        let mut valid = true;
        for (i, bit) in header.iter_mut().enumerate() {
            if let Some(v) = sample(i) {
                *bit = v;
            } else {
                valid = false;
                break;
            }
        }
        if !valid {
            continue;
        }
        for polarity in 0..2 {
            if crate::unhamming(header.map(|v| v ^ polarity)).map(|v| v.0) != Ok(0x4e) {
                continue;
            }
            let mut second = [0u8; 13];
            for (j, bit) in second.iter_mut().enumerate() {
                if let Some(v) = sample(5 * 13 + j) {
                    *bit = v ^ polarity;
                } else {
                    valid = false;
                    break;
                }
            }
            if !valid || crate::unhamming(second).map(|v| v.0) != Ok(0x43) {
                continue;
            }
            for (i, bit) in cells.iter_mut().enumerate() {
                if let Some(v) = sample(i) {
                    *bit = v ^ polarity;
                } else {
                    valid = false;
                    break;
                }
            }
            if !valid {
                continue;
            }
            let mut decoded = family::decode(&cells, format).map(|(p, c)| (p, c, format.id));
            if decoded.is_err() && format.id == 3 {
                let legacy: &[u8; CELLS] = cells.as_slice().try_into().unwrap();
                decoded = decode(legacy).map(|(p, c)| (p, c, 0));
            }
            if let Ok((payload, corrected, format_id)) = decoded {
                let corners = REFERENCE_CORNERS
                    .map(|(x, y)| forward(p, x * cos - y * sin, x * sin + y * cos));
                return Some(Detection {
                    payload,
                    corrected,
                    corners,
                    format_id,
                });
            }
        }
    }
    None
}

// The convex hull estimates the outside ellipse without requiring a continuous finder ring.
#[allow(clippy::needless_range_loop)] // Normal equations and Gaussian elimination.
fn ellipse_seed(initial: [f64; 7], points: &[(f64, f64)], extent: f64) -> [f64; 7] {
    let mut pts = points.to_vec();
    pts.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let cross = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    };
    let mut hull: Vec<(f64, f64)> = Vec::new();
    for &p in &pts {
        while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0. {
            hull.pop();
        }
        hull.push(p);
    }
    let lower = hull.len();
    for &p in pts.iter().rev().skip(1) {
        while hull.len() > lower && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0. {
            hull.pop();
        }
        hull.push(p);
    }
    hull.pop();
    let mut m = [[0.; 6]; 5];
    for (x, y) in hull {
        let u = (x - initial[0]) / initial[2];
        let v = (y - initial[1]) / initial[3];
        let row = [u * u, u * v, v * v, u, v];
        for i in 0..5 {
            for j in 0..5 {
                m[i][j] += row[i] * row[j];
            }
            m[i][5] += row[i];
        }
    }
    for c in 0..5 {
        let pivot = (c..5)
            .max_by(|&a, &b| m[a][c].abs().total_cmp(&m[b][c].abs()))
            .unwrap();
        if m[pivot][c].abs() < 1e-10 {
            return initial;
        }
        m.swap(c, pivot);
        let d = m[c][c];
        for k in c..6 {
            m[c][k] /= d;
        }
        for i in 0..5 {
            if i != c {
                let f = m[i][c];
                for k in c..6 {
                    m[i][k] -= f * m[c][k];
                }
            }
        }
    }
    let (a, b, c, d, e) = (m[0][5], m[1][5], m[2][5], m[3][5], m[4][5]);
    let det = 4. * a * c - b * b;
    if a <= 0. || det <= 0. {
        return initial;
    }
    let cx = (b * e - 2. * c * d) / det;
    let cy = (b * d - 2. * a * e) / det;
    let z = 1. + a * cx * cx + b * cx * cy + c * cy * cy;
    let alpha = extent * (a / z).sqrt();
    let beta = extent * b / (2. * (a * z).sqrt());
    let gamma = extent * ((c - b * b / (4. * a)) / z).sqrt();
    if !alpha.is_finite() || !gamma.is_finite() {
        return initial;
    }
    [
        initial[0] + cx * initial[2],
        initial[1] + cy * initial[3],
        initial[2] / alpha,
        initial[3] / gamma,
        beta / gamma,
        0.,
        0.,
    ]
}
