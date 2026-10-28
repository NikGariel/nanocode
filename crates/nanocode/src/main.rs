use nanocode::{family, palette::Palette};
use std::{env, fs, process};
fn rgb(s: &str) -> Result<u32, String> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return Err("Expected RRGGBB color".into());
    }
    u32::from_str_radix(s, 16).map_err(|e| e.to_string())
}
fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    if ![4, 5, 8].contains(&args.len()) {
        return Err(
            "Usage: nanocode svg|pgm TEXT OUTPUT [auto|S|M|L|XL [ONE ZERO BACKGROUND]]".into(),
        );
    }
    let id = match args.get(4).map(|s| s.to_ascii_uppercase()).as_deref() {
        None | Some("AUTO") => 0,
        Some("S") => 1,
        Some("M") => 2,
        Some("L") => 3,
        Some("XL") => 4,
        _ => return Err("Unknown format".into()),
    };
    let palette = if args.len() == 8 {
        Palette {
            one: rgb(&args[5])?,
            zero: rgb(&args[6])?,
            background: rgb(&args[7])?,
        }
    } else {
        Palette::default()
    };
    let data = args[2].as_bytes();
    let output = match args[1].as_str() {
        "svg" => family::svg(data, id, palette)
            .map_err(|e| format!("{e:?}"))?
            .into_bytes(),
        "pgm" => {
            let mut v = b"P5\n768 768\n255\n".to_vec();
            v.extend(family::render_gray(data, 768, id, palette).map_err(|e| format!("{e:?}"))?);
            v
        }
        "svg-v1" => nanocode::svg(data)
            .map_err(|e| format!("{e:?}"))?
            .into_bytes(),
        _ => return Err("Expected svg, pgm or svg-v1".into()),
    };
    fs::write(&args[3], output).map_err(|e| e.to_string())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
