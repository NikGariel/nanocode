//! Bounded image-region proposals from connected, dilated edges. No payload assumptions.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Region {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
pub(crate) fn proposals(gray: &[u8], width: usize, height: usize) -> Vec<Region> {
    let scale = (width.max(height) as f64 / 360.).max(1.);
    let w = (width as f64 / scale) as usize;
    let h = (height as f64 / scale) as usize;
    if w < 40 || h < 40 {
        return Vec::new();
    }
    let mut small = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let sx = ((x as f64 + 0.5) * scale) as usize;
            let sy = ((y as f64 + 0.5) * scale) as usize;
            small[y * w + x] = gray[sy.min(height - 1) * width + sx.min(width - 1)];
        }
    }
    let mut edges = vec![false; w * h];
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            let gx = (small[i + 1] as i16 - small[i - 1] as i16).abs();
            let gy = (small[i + w] as i16 - small[i - w] as i16).abs();
            edges[i] = gx + gy >= 24;
        }
    }
    let mut regions = Vec::new();
    // Small gaps between arc caps are bridged; larger separation from surrounding text stays open.
    for radius in [1usize, 3] {
        let mut mask = vec![false; w * h];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                if edges[y * w + x] {
                    for yy in y.saturating_sub(radius)..=(y + radius).min(h - 1) {
                        for xx in x.saturating_sub(radius)..=(x + radius).min(w - 1) {
                            mask[yy * w + xx] = true;
                        }
                    }
                }
            }
        }
        let mut queue = Vec::new();
        for start in 0..w * h {
            if !mask[start] {
                continue;
            }
            mask[start] = false;
            queue.clear();
            queue.push(start);
            let (mut minx, mut miny, mut maxx, mut maxy) = (w, h, 0, 0);
            let mut head = 0;
            while head < queue.len() {
                let i = queue[head];
                head += 1;
                let x = i % w;
                let y = i / w;
                minx = minx.min(x);
                maxx = maxx.max(x);
                miny = miny.min(y);
                maxy = maxy.max(y);
                for next in [
                    if x > 0 { Some(i - 1) } else { None },
                    if x + 1 < w { Some(i + 1) } else { None },
                    if y > 0 { Some(i - w) } else { None },
                    if y + 1 < h { Some(i + w) } else { None },
                ]
                .into_iter()
                .flatten()
                {
                    if mask[next] {
                        mask[next] = false;
                        queue.push(next);
                    }
                }
            }
            let bw = maxx - minx + 1;
            let bh = maxy - miny + 1;
            let ratio = bw as f64 / bh as f64;
            if bw < 36 || bh < 36 || !(0.55..1.8).contains(&ratio) || bw * bh > w * h * 9 / 10 {
                continue;
            }
            // Dilated component box already contains a little margin. Add ~2% for the quiet zone.
            let pad = (bw.max(bh) / 50).max(2);
            let x = (minx.saturating_sub(pad) as f64 * scale) as usize;
            let y = (miny.saturating_sub(pad) as f64 * scale) as usize;
            let right = (((maxx + pad + 1).min(w) as f64 * scale).ceil() as usize).min(width);
            let bottom = (((maxy + pad + 1).min(h) as f64 * scale).ceil() as usize).min(height);
            let region = Region {
                x,
                y,
                width: right - x,
                height: bottom - y,
            };
            if region.width >= 100
                && region.height >= 100
                && !regions.iter().any(|r: &Region| {
                    r.x.abs_diff(x) < 12
                        && r.y.abs_diff(y) < 12
                        && r.width.abs_diff(region.width) < 20
                        && r.height.abs_diff(region.height) < 20
                })
            {
                regions.push(region);
            }
        }
    }
    // Large circular candidates first; bounded worst-case decoding work.
    regions.sort_by_key(|r| std::cmp::Reverse(r.width * r.height));
    regions.truncate(8);
    regions
}
