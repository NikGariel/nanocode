//! C callers own all buffers. No strings, Rust layouts or heap ownership cross the ABI.
use crate::{render_gray, svg, vision};
use std::alloc::{alloc_zeroed, dealloc, Layout};
fn err(e: crate::Error) -> i32 {
    match e {
        crate::Error::TooLong => -2,
        crate::Error::NotFound => -3,
        crate::Error::Palette => -5,
        _ => -1,
    }
}
/// # Safety
/// Returned allocation must be released exactly once using nc_free with the same size.
#[no_mangle]
pub unsafe extern "C" fn nc_alloc(size: usize) -> *mut u8 {
    if size == 0 || size > 32 * 1024 * 1024 {
        return std::ptr::null_mut();
    }
    alloc_zeroed(Layout::from_size_align(size, 8).unwrap())
}
/// # Safety
/// Pointer/size must identify a live allocation returned by nc_alloc.
#[no_mangle]
pub unsafe extern "C" fn nc_free(ptr: *mut u8, size: usize) {
    if !ptr.is_null() && size > 0 && size <= 32 * 1024 * 1024 {
        dealloc(ptr, Layout::from_size_align(size, 8).unwrap());
    }
}
unsafe fn input<'a>(p: *const u8, n: usize) -> Option<&'a [u8]> {
    if n == 0 {
        Some(&[])
    } else if p.is_null() {
        None
    } else {
        Some(std::slice::from_raw_parts(p, n))
    }
}
/// # Safety
/// data points to len readable bytes, out to cap writable bytes. Buffers must not overlap.
/// Returns required SVG bytes; writes only when cap is sufficient. No trailing NUL.
#[no_mangle]
pub unsafe extern "C" fn nc_svg(data: *const u8, len: usize, out: *mut u8, cap: usize) -> i32 {
    if len > 24 {
        return -2;
    }
    let Some(d) = input(data, len) else { return -1 };
    match svg(d) {
        Ok(s) => {
            if cap >= s.len() && !out.is_null() {
                std::ptr::copy_nonoverlapping(s.as_ptr(), out, s.len());
            }
            s.len() as i32
        }
        Err(e) => err(e),
    }
}
/// # Safety
/// Buffers must be valid and nonoverlapping; output capacity is in bytes.
#[no_mangle]
pub unsafe extern "C" fn nc_render(
    data: *const u8,
    len: usize,
    size: usize,
    out: *mut u8,
    cap: usize,
) -> i32 {
    if len > 24 {
        return -2;
    }
    if !(128..=4096).contains(&size) || cap < size * size || out.is_null() {
        return -1;
    }
    let Some(d) = input(data, len) else { return -1 };
    match render_gray(d, size) {
        Ok(v) => {
            std::ptr::copy_nonoverlapping(v.as_ptr(), out, v.len());
            v.len() as i32
        }
        Err(e) => err(e),
    }
}
/// # Safety
/// gray points to width*height readable bytes; out has cap bytes; corners is null or 8 doubles.
/// Returns payload length (including zero for empty payload), -3 when no valid code is found.
#[no_mangle]
pub unsafe extern "C" fn nc_scan(
    gray: *const u8,
    width: usize,
    height: usize,
    out: *mut u8,
    cap: usize,
    corners: *mut f64,
) -> i32 {
    if width > 2048 || height > 2048 || width < 32 || height < 32 || out.is_null() || cap < 24 {
        return -1;
    }
    let Some(g) = input(gray, width * height) else {
        return -1;
    };
    match vision::scan(g, width, height) {
        Ok(d) => {
            if d.payload.len() > cap {
                return -4;
            }
            std::ptr::copy_nonoverlapping(d.payload.as_ptr(), out, d.payload.len());
            if !corners.is_null() {
                for i in 0..4 {
                    *corners.add(i * 2) = d.corners[i].x;
                    *corners.add(i * 2 + 1) = d.corners[i].y;
                }
            }
            d.payload.len() as i32
        }
        Err(e) => err(e),
    }
}

use crate::{
    family,
    palette::{self, Palette},
};
#[no_mangle]
pub extern "C" fn nc_format_for_len(len: usize) -> i32 {
    family::select(len, 0).map(|f| f.id as i32).unwrap_or(-2)
}
#[no_mangle]
pub extern "C" fn nc_format_capacity(id: u32) -> i32 {
    if !(1..=4).contains(&id) {
        return -1;
    }
    family::FORMATS[id as usize - 1].capacity as i32
}
#[no_mangle]
pub extern "C" fn nc_format_rings(id: u32) -> i32 {
    if !(1..=4).contains(&id) {
        return -1;
    }
    family::FORMATS[id as usize - 1].counts.len() as i32
}
#[no_mangle]
pub extern "C" fn nc_palette_score(one: u32, zero: u32, background: u32) -> i32 {
    Palette {
        one,
        zero,
        background,
    }
    .assess()
    .score as i32
}
#[no_mangle]
pub extern "C" fn nc_palette_flags(one: u32, zero: u32, background: u32) -> u32 {
    Palette {
        one,
        zero,
        background,
    }
    .assess()
    .flags
}
/// # Safety
/// out is null for sizing, or points to cap writable uint32 values. Returns total recommendations.
#[no_mangle]
pub unsafe extern "C" fn nc_backgrounds(one: u32, zero: u32, out: *mut u32, cap: usize) -> i32 {
    let list = palette::backgrounds(one, zero);
    if !out.is_null() && cap >= list.len() {
        for (i, (color, _)) in list.iter().enumerate() {
            *out.add(i) = *color;
        }
    }
    list.len() as i32
}
/// # Safety
/// Valid, non-overlapping buffers as for nc_svg. id=0 auto, 1..4 explicit. RGB is 0xRRGGBB.
#[no_mangle]
pub unsafe extern "C" fn nc_svg_ex(
    data: *const u8,
    len: usize,
    id: u32,
    one: u32,
    zero: u32,
    background: u32,
    out: *mut u8,
    cap: usize,
) -> i32 {
    if len > family::MAX_PAYLOAD {
        return -2;
    }
    if id > 4 {
        return -1;
    }
    let Some(d) = input(data, len) else {
        return -1;
    };
    match family::svg(
        d,
        id as u8,
        Palette {
            one,
            zero,
            background,
        },
    ) {
        Ok(s) => {
            if !out.is_null() && cap >= s.len() {
                std::ptr::copy_nonoverlapping(s.as_ptr(), out, s.len());
            }
            s.len() as i32
        }
        Err(e) => err(e),
    }
}
/// # Safety
/// Non-overlapping readable input and writable size*size output; cap is in bytes.
#[no_mangle]
pub unsafe extern "C" fn nc_render_ex(
    data: *const u8,
    len: usize,
    id: u32,
    size: usize,
    one: u32,
    zero: u32,
    background: u32,
    out: *mut u8,
    cap: usize,
) -> i32 {
    if len > family::MAX_PAYLOAD {
        return -2;
    }
    if id > 4 || !(128..=4096).contains(&size) || out.is_null() || cap < size * size {
        return -1;
    }
    let Some(d) = input(data, len) else {
        return -1;
    };
    match family::render_gray(
        d,
        size,
        id as u8,
        Palette {
            one,
            zero,
            background,
        },
    ) {
        Ok(v) => {
            std::ptr::copy_nonoverlapping(v.as_ptr(), out, v.len());
            v.len() as i32
        }
        Err(e) => err(e),
    }
}
/// # Safety
/// Gray is width*height bytes. out has cap bytes; corners is null or eight doubles;
/// metadata is null or two uint32 values: format id (0=legacy), corrected bit count.
/// All buffers must be valid and non-overlapping. Insufficient out returns -4 without writing.
#[no_mangle]
pub unsafe extern "C" fn nc_scan_ex(
    gray: *const u8,
    width: usize,
    height: usize,
    out: *mut u8,
    cap: usize,
    corners: *mut f64,
    metadata: *mut u32,
) -> i32 {
    if !(32..=2048).contains(&width) || !(32..=2048).contains(&height) || out.is_null() {
        return -1;
    }
    let Some(g) = input(gray, width * height) else {
        return -1;
    };
    match vision::scan(g, width, height) {
        Ok(d) => {
            if cap < d.payload.len() {
                return -4;
            }
            std::ptr::copy_nonoverlapping(d.payload.as_ptr(), out, d.payload.len());
            if !corners.is_null() {
                for i in 0..4 {
                    *corners.add(2 * i) = d.corners[i].x;
                    *corners.add(2 * i + 1) = d.corners[i].y;
                }
            }
            if !metadata.is_null() {
                *metadata = d.format_id as u32;
                *metadata.add(1) = d.corrected as u32;
            }
            d.payload.len() as i32
        }
        Err(e) => err(e),
    }
}
