# NanoCode v2: formats and palettes

Experimental specification, 2026-10-28. Preserves merged round-capped arcs and the open
centre. The v1 eight-ring format remains readable. v0 (the original JS prototype) does not.

## Fixed profiles

| ID | Name | Maximum payload bytes | Rings | Packet bytes | Encoded cells | Stroke |
|---|---|---|---|---|---|---|
| 1 | S | 8 | 4 | 16 | 208 | .040 |
| 2 | M | 16 | 6 | 24 | 312 | .036 |
| 3 | L | 24 | 8 | 32 | 416 | .032 |
| 4 | XL | 40 | 10 | 48 | 624 | .026 |

Auto generation (id=0) selects the smallest capacity >= byte length. Empty payload selects S.
A forced format never truncates data: insufficient capacity is an error. There is no implicit
compression, URL expansion, text normalization or numeric conversion. Leading zeros survive.
UTF-8 text: 16 ASCII characters =16 bytes (M); 16 Cyrillic letters commonly =32 bytes (XL).
Native callers may pass arbitrary bytes, including NUL. 41 bytes are rejected.

Cell counts inside to outside:

- S: `36,46,58,68`
- M: `32,42,50,56,62,70`
- L: `32,36,44,48,56,60,68,72`
- XL: `32,38,44,50,56,62,70,80,90,102`

Ring i radius is `.42+.49*i/(rings-1)`. Coordinates, read positions and cyclic run merging
follow v1. Stroke varies by profile. For every maximal cyclic run of k equal bits from cell j,
centreline endpoints are `2π*j/N+trim` and `2π*(j+k)/N-trim`, with
`trim=(stroke/2+.008)/radius`. Round caps, gap about .016, no external locator marks.
Centre radius <.40 is free of encoded marks. Keep a logo within the conservative square
[-.18,+.18] on each axis (corner radius about .255). The reader preserves a clean-frame fast path and, for logo-aware region refinement,
excludes normalized ellipse radius .32 from ink classification and geometry fitting. This is
tested with synthetic high-contrast brand marks, not every logo/camera combination.
The JS logoDataUrl option embeds PNG/JPEG/WebP; the native SVG APIs leave logo compositing
to the host. Preserve the uniform background and outer margin.

Smaller profiles use fewer rings and thicker strokes at the same outside diameter, improving
radial spacing. This does not quantify a camera error rate: equal ECC is used in every profile,
and angular pitch, rendering resolution, focus and lighting also matter. No minimum print size
or production scan-rate promise is established by the synthetic tests.

## Packet / identification

Packet length n = payload capacity + 8:

| Bytes | Value |
|---|---|
| 0–1 | `4e 43` |
| 2 | `20 + profile_id` hexadecimal: `21`, `22`, `23`, `24` |
| 3 | Payload byte length |
| 4 through n−5 | Payload followed by zero padding |
| n−4 through n−1 | CRC-32/ISO-HDLC of all preceding bytes, big-endian |

Each byte uses the same extended Hamming (13,8) as [v1](SPEC-v1.md). Logical byte i's
Hamming bit j (zero based) maps to physical cell `((5*i) mod n)*13+j`. Five is coprime
to each allowed packet length. Bits inside a codeword stay together, retaining long arcs;
this is a deliberate visual tradeoff against burst-error resistance. Decode validates
ECC, magic, exact version/profile tag, length, padding and CRC before returning bytes.
One bit per codeword is correctable; two are rejected; CRC is not authentication.

The reader ranks candidate ring geometries, fits each as needed, searches orientation and
both ink polarities, and validates the protected tag. It does not trust apparent ring count
alone. L and v1 share geometry but have distinct protected headers. Result format id=0
means legacy v1, 1–4 means v2. A future profile requires an explicit new table/tag entry;
arbitrary user-supplied ring counts are not valid formats.

## Palette API and background advice

Palette consists of three opaque sRGB integers `0xRRGGBB`: `one`, `zero`, `background`.
Default inks: `00a6a1` and `77dddd`, background `ffffff`. Colours do not change data bits.
Transparent, gradient, textured and photographic backgrounds are not supported by the API.
A consumer can pass its two brand colours to `backgrounds()` and present the returned swatches.
The chosen background is rendered into SVG; don't change it without evaluating the new palette.

Reference grayscale signal: `Y=(77R+150G+29B)>>8` in [0,255]. This approximates the reader's
RGB adapter, not linear-light luminance and not WCAG contrast. Three pairwise differences:
`abs(Y1-Y0)`, `abs(Y1-Ybg)`, `abs(Y0-Ybg)`.

- Score = smallest of these differences, in levels out of 255.
- Any difference <35: reject generation with `Error::Palette` / C code −5.
- Score 35–54: allowed with reduced margin.
- Score >=55: recommended by the heuristic, still requires print/camera validation.
- Flags: 1 invalid RGB; 2 inks too close; 4 background too close (flags can combine).

Candidates are white `ffffff`, off-white `f4f4f4`, and neutral `e0e0e0`, `b0b0b0`,
`808080`, `505050`, `202020`, `000000`, filtered to allowed and sorted by decreasing score.
An empty list means the current inks have no safe suggestion; the host must ask to change
inks, not silently choose white. The host can evaluate any other RGB background with assess().
These are explicitly conservative engineering heuristics, not measured probabilities.

Reader infers three intensity classes, identifies background by the majority class at frame
borders, treats the other two as ink, and tries both bit assignments. Therefore background
may be brighter, darker, or between the inks in grayscale, and the caller need not supply
colours. A candidate ROI border must predominantly show the same uniform background as the code.
The scanner now first tries the complete image, then proposes up to eight regions from
connected dilated edges on a downsampled image. Each proposal can be inset to discard
card/screen borders. Pose coordinates are translated back to the original frame.
A ring-coverage check rejects profiles whose expected rings are absent. Refinement has
an evaluation budget so an unrecognisable frame cannot cause unlimited fitting work.
This handles tested code tiles surrounded by monitor UI, not arbitrary clutter, glare or
strong uneven illumination.
Two different hues with nearly equal grayscale brightness are intentionally not accepted.

## Library contracts

Rust: `family::select`, `family::encode/decode`, `family::svg/render_gray`, `vision::scan`,
`palette::Palette::assess`, `palette::backgrounds`. Old root codec/generator functions stay v1.

C/WASM: declarations in `include/nanocode.h`. New entry points:
`nc_format_for_len`, `nc_format_capacity`, `nc_format_rings`, `nc_svg_ex`, `nc_render_ex`,
`nc_scan_ex`, `nc_palette_score`, `nc_palette_flags`, `nc_backgrounds`.

`nc_scan_ex` accepts packed grayscale and returns length or error; use >=40-byte output.
Its optional metadata contains two uint32 values: format id and corrected bit count.
Old `nc_scan` also recognises all formats but returns −4 if output capacity is insufficient.
−4 leaves output/corners/metadata untouched. Existing `nc_svg`/`nc_render` keep v1 output.
C pointer validity, non-overlap, ownership and size-query rules follow the header/v1 ABI.

JS wrapper exposes `formats`, `formatFor(data, id=0)`, `svg(data, options)`, `assess(palette)`,
`backgrounds(one,zero)`, `scan(gray,width,height)`. `scan` returns format id/name and correction
count as well as payload, decoded UTF-8 display text and pose points. Binary callers should
use bytes, not the display text. The camera Worker uses the same autodetecting WASM scanner.


### Camera demo behaviour

The demo requests up to 1920×1080 capture and processes frames downsampled to at most
1280px on the long side. Actual capture settings depend on the camera/browser. Two
matching CRC-validated results among the last six frames within 2.5 seconds confirm a
read; intervening misses no longer reset all progress. A confirmed result stays visible
for two seconds, but stale pose coordinates are not drawn on a new missed frame.
“Сохранить кадр” downloads one original-resolution camera PNG locally for diagnosis.
The image is not uploaded. This does not change the code format or its ECC.
