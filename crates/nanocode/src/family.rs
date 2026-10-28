//! Version 2 profiles. Geometry is fixed per profile, never inferred from message length alone.
use crate::{crc32, hamming, palette::Palette, unhamming, Arc, Error};
use std::fmt::Write;
pub const MAX_PAYLOAD: usize = 40;
#[derive(Debug, Clone, Copy)]
pub struct Format {
    pub id: u8,
    pub name: &'static str,
    pub capacity: usize,
    pub counts: &'static [usize],
    pub stroke: f64,
}
pub const FORMATS: [Format; 4] = [
    Format {
        id: 1,
        name: "S",
        capacity: 8,
        counts: &[36, 46, 58, 68],
        stroke: 0.040,
    },
    Format {
        id: 2,
        name: "M",
        capacity: 16,
        counts: &[32, 42, 50, 56, 62, 70],
        stroke: 0.036,
    },
    Format {
        id: 3,
        name: "L",
        capacity: 24,
        counts: &[32, 36, 44, 48, 56, 60, 68, 72],
        stroke: 0.032,
    },
    Format {
        id: 4,
        name: "XL",
        capacity: 40,
        counts: &[32, 38, 44, 50, 56, 62, 70, 80, 90, 102],
        stroke: 0.026,
    },
];
impl Format {
    pub fn cells(self) -> usize {
        (self.capacity + 8) * 13
    }
    pub fn radii(self) -> Vec<f64> {
        (0..self.counts.len())
            .map(|i| 0.42 + 0.49 * i as f64 / (self.counts.len() - 1) as f64)
            .collect()
    }
    pub fn points(self) -> Vec<(f64, f64)> {
        let mut p = Vec::new();
        for (&r, &n) in self.radii().iter().zip(self.counts) {
            for i in 0..n {
                let a = std::f64::consts::TAU * (i as f64 + 0.5) / n as f64;
                p.push((r * a.cos(), r * a.sin()));
            }
        }
        p
    }
}
pub fn select(len: usize, id: u8) -> Result<Format, Error> {
    let f = if id == 0 {
        FORMATS.iter().find(|f| len <= f.capacity)
    } else {
        FORMATS.iter().find(|f| f.id == id)
    }
    .ok_or(if id > 4 {
        Error::Invalid
    } else {
        Error::TooLong
    })?;
    if len > f.capacity {
        return Err(Error::TooLong);
    }
    Ok(*f)
}
pub fn encode(data: &[u8], format: Format) -> Result<Vec<u8>, Error> {
    let format = select(data.len(), format.id)?;
    let n = format.capacity + 8;
    let mut packet = vec![0; n];
    packet[..4].copy_from_slice(&[0x4e, 0x43, 0x20 + format.id, data.len() as u8]);
    packet[4..4 + data.len()].copy_from_slice(data);
    let checksum = crc32(&packet[..n - 4]);
    packet[n - 4..].copy_from_slice(&checksum.to_be_bytes());
    let mut cells = vec![0; format.cells()];
    for (i, &b) in packet.iter().enumerate() {
        for (j, bit) in hamming(b).iter().enumerate() {
            cells[((5 * i) % n) * 13 + j] = *bit;
        }
    }
    Ok(cells)
}
pub fn decode(cells: &[u8], format: Format) -> Result<(Vec<u8>, usize), Error> {
    if cells.len() != format.cells() {
        return Err(Error::Invalid);
    }
    let n = format.capacity + 8;
    let mut packet = vec![0; n];
    let mut corrected = 0;
    for (i, b) in packet.iter_mut().enumerate() {
        let w = std::array::from_fn(|j| cells[((5 * i) % n) * 13 + j]);
        let (v, c) = unhamming(w)?;
        if i < 3 && v != [0x4e, 0x43, 0x20 + format.id][i] {
            return Err(Error::Invalid);
        }
        *b = v;
        corrected += c;
    }
    let length = packet[3] as usize;
    if length > format.capacity || packet[4 + length..n - 4].iter().any(|&b| b != 0) {
        return Err(Error::Invalid);
    }
    if crc32(&packet[..n - 4]) != u32::from_be_bytes(packet[n - 4..].try_into().unwrap()) {
        return Err(Error::Checksum);
    }
    Ok((packet[4..4 + length].to_vec(), corrected))
}
pub fn arcs(cells: &[u8], format: Format) -> Vec<Arc> {
    let mut out = Vec::new();
    let mut offset = 0;
    for (&r, &n) in format.radii().iter().zip(format.counts) {
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
            let trim = (format.stroke / 2. + 0.008) / r;
            out.push(Arc {
                radius: r,
                start: (first + used) as f64 * pitch + trim,
                end: (first + used + count) as f64 * pitch - trim,
                bit,
            });
            used += count;
        }
        offset += n;
    }
    out
}
pub fn sample(runs: &[Arc], format: Format, levels: [u8; 3], x: f64, y: f64) -> u8 {
    let radius = x.hypot(y);
    let angle = y.atan2(x).rem_euclid(std::f64::consts::TAU);
    let half = format.stroke / 2.;
    for arc in runs {
        if (radius - arc.radius).abs() > half {
            continue;
        }
        let a = arc.start + (angle - arc.start).rem_euclid(std::f64::consts::TAU);
        if a <= arc.end
            || (x - arc.radius * arc.start.cos()).hypot(y - arc.radius * arc.start.sin()) <= half
            || (x - arc.radius * arc.end.cos()).hypot(y - arc.radius * arc.end.sin()) <= half
        {
            return levels[arc.bit as usize];
        }
    }
    levels[2]
}
pub fn render_gray(data: &[u8], size: usize, id: u8, palette: Palette) -> Result<Vec<u8>, Error> {
    if !(128..=4096).contains(&size) {
        return Err(Error::Invalid);
    }
    if palette.assess().flags != 0 {
        return Err(Error::Palette);
    }
    let f = select(data.len(), id)?;
    let runs = arcs(&encode(data, f)?, f);
    let levels = palette.levels();
    Ok((0..size * size)
        .map(|i| {
            sample(
                &runs,
                f,
                levels,
                2. * (i % size) as f64 / size as f64 - 1.,
                2. * (i / size) as f64 / size as f64 - 1.,
            )
        })
        .collect())
}
pub fn svg(data: &[u8], id: u8, palette: Palette) -> Result<String, Error> {
    if palette.assess().flags != 0 {
        return Err(Error::Palette);
    }
    let f = select(data.len(), id)?;
    let runs = arcs(&encode(data, f)?, f);
    let mut s=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"768\" height=\"768\" viewBox=\"-1 -1 2 2\"><rect x=\"-1\" y=\"-1\" width=\"2\" height=\"2\" fill=\"#{:06x}\"/>",palette.background);
    for arc in runs {
        let r = arc.radius;
        let a = arc.start;
        let b = arc.end;
        let color = if arc.bit == 1 {
            palette.one
        } else {
            palette.zero
        };
        let large = u8::from(b - a > std::f64::consts::PI);
        write!(s,"<path d=\"M {:.7} {:.7} A {r} {r} 0 {large} 1 {:.7} {:.7}\" fill=\"none\" stroke=\"#{color:06x}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",r*a.cos(),r*a.sin(),r*b.cos(),r*b.sin(),f.stroke).unwrap();
    }
    s.push_str("</svg>");
    Ok(s)
}
