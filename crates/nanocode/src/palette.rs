//! Palette screening is an engineering heuristic, not a print/camera guarantee.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub one: u32,
    pub zero: u32,
    pub background: u32,
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            one: 0x00a6a1,
            zero: 0x77dddd,
            background: 0xffffff,
        }
    }
}
pub fn luminance(rgb: u32) -> u8 {
    let r = (rgb >> 16) & 255;
    let g = (rgb >> 8) & 255;
    let b = rgb & 255;
    ((77 * r + 150 * g + 29 * b) >> 8) as u8
}
#[derive(Debug, Clone, Copy)]
pub struct Assessment {
    pub score: u8,
    pub flags: u32,
}
pub const INVALID_RGB: u32 = 1;
pub const INKS_TOO_CLOSE: u32 = 2;
pub const BACKGROUND_TOO_CLOSE: u32 = 4;
pub const MIN_SEPARATION: u8 = 35;
pub const RECOMMENDED_SEPARATION: u8 = 55;
impl Assessment {
    pub fn allowed(self) -> bool {
        self.flags == 0
    }
    pub fn recommended(self) -> bool {
        self.allowed() && self.score >= RECOMMENDED_SEPARATION
    }
}
impl Palette {
    pub fn assess(self) -> Assessment {
        if self.one > 0xffffff || self.zero > 0xffffff || self.background > 0xffffff {
            return Assessment {
                score: 0,
                flags: INVALID_RGB,
            };
        }
        let a = luminance(self.one);
        let b = luminance(self.zero);
        let c = luminance(self.background);
        let ink = a.abs_diff(b);
        let bg = a.abs_diff(c).min(b.abs_diff(c));
        Assessment {
            score: ink.min(bg),
            flags: if ink < MIN_SEPARATION {
                INKS_TOO_CLOSE
            } else {
                0
            } | if bg < MIN_SEPARATION {
                BACKGROUND_TOO_CLOSE
            } else {
                0
            },
        }
    }
    pub fn levels(self) -> [u8; 3] {
        [
            luminance(self.zero),
            luminance(self.one),
            luminance(self.background),
        ]
    }
}
/// Up to eight tints/shades of the primary ink, best separation first.
/// Every returned palette has at least 55/255 separation for all three pairs.
/// An empty result means the primary/background pair must change first.
pub fn secondary_colors(one: u32, background: u32) -> Vec<(u32, Assessment)> {
    if one > 0xffffff || background > 0xffffff {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for target in [0u32, 255] {
        for amount in 0u32..=255 {
            let channel = |shift: u32| {
                let c = (one >> shift) & 255;
                (c * (255 - amount) + target * amount + 127) / 255
            };
            let zero = (channel(16) << 16) | (channel(8) << 8) | channel(0);
            let assessment = Palette {
                one,
                zero,
                background,
            }
            .assess();
            if assessment.recommended() {
                candidates.push((zero, assessment));
            }
        }
    }
    candidates.sort_by_key(|&(color, a)| (std::cmp::Reverse(a.score), color));
    let mut result: Vec<(u32, Assessment)> = Vec::new();
    for candidate in candidates {
        // Show distinct brightness choices rather than nearly identical swatches.
        if result
            .iter()
            .all(|&(color, _)| luminance(color).abs_diff(luminance(candidate.0)) >= 12)
        {
            result.push(candidate);
            if result.len() == 8 {
                break;
            }
        }
    }
    result
}
/// Only safe candidates are returned, sorted by weakest pairwise luma separation.
/// The empty result means these inks need changing; do not silently choose a background.
pub fn backgrounds(one: u32, zero: u32) -> Vec<(u32, Assessment)> {
    let mut result: Vec<_> = [
        0xffffff, 0xf4f4f4, 0xe0e0e0, 0xb0b0b0, 0x808080, 0x505050, 0x202020, 0x000000,
    ]
    .into_iter()
    .map(|background| {
        (
            background,
            Palette {
                one,
                zero,
                background,
            }
            .assess(),
        )
    })
    .filter(|(_, a)| a.flags == 0)
    .collect();
    result.sort_by_key(|(_, a)| std::cmp::Reverse(a.score));
    result
}
