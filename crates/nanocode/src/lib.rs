//! NanoCode v1: byte codec, vector/raster rendering and projective image reader.
pub mod family;
mod ffi;
pub mod palette;
pub mod vision;
mod vision_preprocess;
mod vision_regions;
use std::fmt::Write;
pub const MAX_PAYLOAD: usize = 24;
pub const CELLS: usize = 416;
pub const COUNTS: [usize; 8] = [32, 36, 44, 48, 56, 60, 68, 72];
pub const RADII: [f64; 8] = [0.42, 0.49, 0.56, 0.63, 0.70, 0.77, 0.84, 0.91];
pub const REFERENCE_CORNERS: [(f64, f64); 4] =
    [(-0.72, -0.72), (0.72, -0.72), (0.72, 0.72), (-0.72, 0.72)];
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    TooLong,
    Invalid,
    Checksum,
    NotFound,
    Palette,
}
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320u32 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}
const DATA_POS: [usize; 8] = [3, 5, 6, 7, 9, 10, 11, 12];
fn hamming(b: u8) -> [u8; 13] {
    let mut w = [0; 13];
    for (i, &p) in DATA_POS.iter().enumerate() {
        w[p - 1] = (b >> (7 - i)) & 1;
    }
    for p in [1, 2, 4, 8] {
        let mut parity = 0;
        for j in 1..=12 {
            if j & p != 0 {
                parity ^= w[j - 1];
            }
        }
        w[p - 1] = parity;
    }
    w[12] = w[..12].iter().fold(0, |a, b| a ^ b);
    w
}
fn unhamming(mut w: [u8; 13]) -> Result<(u8, usize), Error> {
    if w.iter().any(|&x| x > 1) {
        return Err(Error::Invalid);
    }
    let mut s = 0;
    for j in 1..=12 {
        if w[j - 1] != 0 {
            s ^= j;
        }
    }
    let odd = w.iter().fold(0, |a, b| a ^ b);
    let corrected = if odd != 0 {
        if s > 12 {
            return Err(Error::Checksum);
        }
        if s > 0 {
            w[s - 1] ^= 1;
        }
        1
    } else {
        if s != 0 {
            return Err(Error::Checksum);
        }
        0
    };
    let b = DATA_POS.iter().fold(0, |a, &p| (a << 1) | w[p - 1]);
    Ok((b, corrected))
}
pub fn encode(data: &[u8]) -> Result<[u8; CELLS], Error> {
    if data.len() > MAX_PAYLOAD {
        return Err(Error::TooLong);
    }
    let mut packet = [0u8; 32];
    packet[..4].copy_from_slice(&[0x4e, 0x43, 1, data.len() as u8]);
    packet[4..4 + data.len()].copy_from_slice(data);
    let crc = crc32(&packet[..28]);
    packet[28..].copy_from_slice(&crc.to_be_bytes());
    let mut cells = [0; CELLS];
    for (i, &b) in packet.iter().enumerate() {
        for (j, bit) in hamming(b).iter().enumerate() {
            cells[((i * 5) % 32) * 13 + j] = *bit;
        }
    }
    Ok(cells)
}
pub fn decode(cells: &[u8; CELLS]) -> Result<(Vec<u8>, usize), Error> {
    let mut packet = [0; 32];
    let mut corrected = 0;
    for (i, b) in packet.iter_mut().enumerate() {
        let mut w = [0; 13];
        for j in 0..13 {
            w[j] = cells[((i * 5) % 32) * 13 + j];
        }
        let (v, c) = unhamming(w)?;
        *b = v;
        corrected += c;
        if i < 3 && v != [0x4e, 0x43, 1][i] {
            return Err(Error::Invalid);
        }
    }
    if packet[..3] != [0x4e, 0x43, 1] || packet[3] as usize > MAX_PAYLOAD {
        return Err(Error::Invalid);
    }
    if crc32(&packet[..28]) != u32::from_be_bytes(packet[28..].try_into().unwrap()) {
        return Err(Error::Checksum);
    }
    let end = 4 + packet[3] as usize;
    if packet[end..28].iter().any(|&b| b != 0) {
        return Err(Error::Invalid);
    }
    Ok((packet[4..end].to_vec(), corrected))
}
pub fn cell_points() -> Vec<(f64, f64)> {
    let mut p = Vec::with_capacity(CELLS);
    for ring in 0..COUNTS.len() {
        for i in 0..COUNTS[ring] {
            let a = std::f64::consts::TAU * (i as f64 + 0.5) / COUNTS[ring] as f64;
            p.push((RADII[ring] * a.cos(), RADII[ring] * a.sin()));
        }
    }
    p
}
/// A run is rendered as one circular arc, including runs crossing the angular origin.
#[derive(Clone, Copy)]
pub struct Arc {
    pub radius: f64,
    pub start: f64,
    pub end: f64,
    pub bit: u8,
}
pub const STROKE: f64 = 0.032;
pub fn arcs(cells: &[u8; CELLS]) -> Vec<Arc> {
    let mut result = Vec::new();
    let mut offset = 0;
    for ring in 0..COUNTS.len() {
        let n = COUNTS[ring];
        let r = RADII[ring];
        // Start at a colour transition so the 0/2π seam never splits a run.
        let first = (0..n)
            .find(|&i| cells[offset + i] != cells[offset + (i + n - 1) % n])
            .unwrap_or(0);
        let mut used = 0;
        while used < n {
            let bit = cells[offset + (first + used) % n];
            let mut count = 1;
            while used + count < n && cells[offset + (first + used + count) % n] == bit {
                count += 1;
            }
            let pitch = std::f64::consts::TAU / n as f64;
            // Reserve a visible white gap plus the two round caps at transitions.
            let trim = (STROKE * 0.5 + 0.008) / r;
            let start = (first + used) as f64 * pitch + trim;
            let end = (first + used + count) as f64 * pitch - trim;
            result.push(Arc {
                radius: r,
                start,
                end,
                bit,
            });
            used += count;
        }
        offset += n;
    }
    result
}
pub fn arc_pixel(arcs: &[Arc], x: f64, y: f64) -> u8 {
    let radius = x.hypot(y);
    let angle = y.atan2(x).rem_euclid(std::f64::consts::TAU);
    for arc in arcs {
        if (radius - arc.radius).abs() > STROKE * 0.5 {
            continue;
        }
        let unwrapped = arc.start + (angle - arc.start).rem_euclid(std::f64::consts::TAU);
        let on_body = unwrapped <= arc.end;
        let near_start = (x - arc.radius * arc.start.cos()).hypot(y - arc.radius * arc.start.sin())
            <= STROKE * 0.5;
        let near_end =
            (x - arc.radius * arc.end.cos()).hypot(y - arc.radius * arc.end.sin()) <= STROKE * 0.5;
        if on_body || near_start || near_end {
            return if arc.bit == 1 { 115 } else { 190 };
        }
    }
    255
}
/// Reference luminance renderer for the teal palette; centre radius < .40 is free.
pub fn pixel(cells: &[u8; CELLS], x: f64, y: f64) -> u8 {
    arc_pixel(&arcs(cells), x, y)
}
pub fn render_gray(data: &[u8], size: usize) -> Result<Vec<u8>, Error> {
    if !(128..=4096).contains(&size) {
        return Err(Error::Invalid);
    }
    let c = arcs(&encode(data)?);
    Ok((0..size * size)
        .map(|i| {
            arc_pixel(
                &c,
                2.0 * (i % size) as f64 / size as f64 - 1.0,
                2.0 * (i / size) as f64 / size as f64 - 1.0,
            )
        })
        .collect())
}
pub fn svg(data: &[u8]) -> Result<String, Error> {
    let runs = arcs(&encode(data)?);
    let mut s=String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"768\" height=\"768\" viewBox=\"-1 -1 2 2\"><rect x=\"-1\" y=\"-1\" width=\"2\" height=\"2\" fill=\"white\"/>");
    for arc in runs {
        let r = arc.radius;
        let a = arc.start;
        let b = arc.end;
        let color = if arc.bit == 1 { "#00a6a1" } else { "#77dddd" };
        let large = u8::from(b - a > std::f64::consts::PI);
        write!(s,"<path d=\"M {:.7} {:.7} A {r} {r} 0 {large} 1 {:.7} {:.7}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{STROKE}\" stroke-linecap=\"round\"/>",r*a.cos(),r*a.sin(),r*b.cos(),r*b.sin()).unwrap();
    }
    s.push_str("</svg>");
    Ok(s)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checksum_vector() {
        assert_eq!(crc32(b"123456789"), 0xcbf43926);
    }
    #[test]
    fn roundtrips() {
        for n in 0..=24 {
            let d: Vec<u8> = (0..n).map(|i| (i * 19) as u8).collect();
            assert_eq!(decode(&encode(&d).unwrap()).unwrap().0, d);
        }
        assert_eq!(encode(&[0; 25]), Err(Error::TooLong));
    }
    #[test]
    fn exhaustive_ecc() {
        for b in 0..=255u8 {
            let w = hamming(b);
            for i in 0..13 {
                let mut e = w;
                e[i] ^= 1;
                assert_eq!(unhamming(e), Ok((b, 1)));
                for j in i + 1..13 {
                    let mut e = w;
                    e[i] ^= 1;
                    e[j] ^= 1;
                    assert_eq!(unhamming(e), Err(Error::Checksum));
                }
            }
        }
    }
    #[test]
    fn every_cell_error() {
        let original = encode(b"00NanoCode").unwrap();
        for i in 0..CELLS {
            let mut c = original;
            c[i] ^= 1;
            assert_eq!(decode(&c).unwrap(), (b"00NanoCode".to_vec(), 1));
        }
    }
}
