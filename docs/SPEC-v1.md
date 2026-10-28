# NanoCode v1 — experimental specification

Status: implementable reference format, not yet frozen for mass printing. Version byte: 1.
This is an independent symbology, not an App Clip Code and not readable by Apple's scanner.
No external lookup, URL shortener or network is required to recover its bytes.

## Packet and error correction

Input: 0–24 arbitrary bytes. Text adapters use UTF-8; capacity counts bytes, not characters.
A message beginning with zero bytes or the text `00` retains them. No implicit URL handling.

| Byte offsets | Meaning |
| --- | --- |
| 0–1 | Magic `4e 43` (ASCII NC) |
| 2 | Version `01` |
| 3 | Payload length 0–24 |
| 4–27 | Payload, then zero padding |
| 28–31 | CRC32 of bytes 0–27, big-endian |

CRC32 is CRC-32/ISO-HDLC: reflected polynomial `edb88320`, init/xorout `ffffffff`.
Check: ASCII `123456789` → `cbf43926`. CRC is integrity checking, not authentication.

Encode each of the 32 bytes independently using shortened extended Hamming (13,8):

* Positions 1–12 use Hamming indexing; parity occupies 1,2,4,8.
* Data positions 3,5,6,7,9,10,11,12 receive byte bits MSB first.
* Parity is even over indices whose bitwise AND with the parity position is nonzero.
* Position 13 is even parity over positions 1–12.
* Concatenate codewords, first packet byte first, positions 1–13 first.
* Permute whole codewords: packet byte `i`, Hamming bit `j` (0–12) maps to
  physical cell `((5*i) mod 32)*13+j`. Bits within a codeword stay adjacent, preserving
  long same-colour runs. This favours the arc design over burst-error resilience.

Decoder computes syndrome over positions 1–12 and parity over all 13. Odd parity and
syndrome 1–12 correct that bit; odd parity and syndrome zero correct the overall parity.
Nonzero syndrome with even parity, or syndrome >12, reject. Then verify magic, version,
length, zero padding and CRC. Never return an unchecked payload. One bad bit per codeword
is correctable, two are detected. Three or more are not guaranteed detectable by Hamming;
CRC provides an additional check. This is not Reed–Solomon or arbitrary scratch recovery.

## Canonical geometry

Cartesian image coordinates: x right, y down, square `[-1,1]²`, white background.
Map to a raster with `x_pixel=(x+1)*width/2`, similarly y. No arbitrary aspect stretching.

No external markers, locator dots or squares are drawn. The eight concentric rings
are the visual code. Keep the margin to the square edge white and unobstructed.
Centre radius < `.40` is visually free. The current reader expects it blank; logo-aware
localisation is future work. Merely placing artwork there does not guarantee scanning.

| Ring, inside → outside | Radius | Cells | First physical index |
| --- | --- | --- | --- |
| 0 | .42 | 32 | 0 |
| 1 | .49 | 36 | 32 |
| 2 | .56 | 44 | 68 |
| 3 | .63 | 48 | 112 |
| 4 | .70 | 56 | 160 |
| 5 | .77 | 60 | 216 |
| 6 | .84 | 68 | 276 |
| 7 | .91 | 72 | 344 |

Physical cells walk inside to outside, clockwise from +x, with no per-ring rotation.
Cell j's read position is `angle=2π(j+.5)/N`, at the ring's centre radius.
The drawing uses maximal cyclic runs of equal bits, not separate cell strokes.
A run crossing angle zero is merged across that seam. For a run of k cells starting
at cell j, draw a centreline arc from `2π*j/N + trim` to `2π*(j+k)/N - trim`, where
`trim=(.032/2+.008)/radius`. Stroke width is `.032`, round caps. Thus adjacent runs
have approximately `.016` visible white separation along the ring. A single-colour
ring has one almost-full-circle arc with its gap at angle zero.

Bit 1 is teal `#00a6a1`, bit 0 is light teal `#77dddd`, background white. Reference
luminance raster uses 115 and 190 respectively. SVG specifies 768×768 intrinsic size
and viewBox -1 -1 2 2. SVG contains no payload metadata. Browser scaling must keep a
square aspect ratio. Use 384px or more for initial trials; a synthetic test at 256px
is not a reliable minimum camera resolution. Other palettes and inverted codes are
not yet specified.

## Reference reader

Input: packed 8-bit grayscale, 32–2048 pixels per dimension. Convert camera Y planes by
respecting row/pixel stride, or RGB using `(77R+150G+29B)>>8`. Downsample large frames.
The whole code must be visible on a clean light background. A camera may supply a
cropped region of interest. General scene localisation is not solved by this reader.

1. Fit three luminance clusters (two inks and paper), reject inadequate separation.
2. Find ink bounds and a bounded sample of about 3000 ink pixels.
3. Fit an outer ellipse from the convex hull for an initial geometric estimate.
4. Refine a seven-parameter projective mapping against the known concentric radii,
   with additional initialisations if the first fit has high residual error.
5. Search angular orientation at half-degree intervals, project each cell centre,
   classify ink, undo codeword permutation, correct Hamming errors and validate CRC.
6. Return only a verified payload and four virtual reference points in payload order.

The reference points are projections of `(-.72,-.72),(.72,-.72),(.72,.72),(-.72,.72)`;
they are neither printed markers nor the outer border. They define the recovered
homography for a host's overlay. There is no promise of mirrored image support.
These are reference implementation choices; stronger readers may use local contrast,
subpixel contour fitting, temporal tracking and calibrated lens distortion.

## ABI contract

See `include/nanocode.h`. C symbols `nc_svg`, `nc_render`, `nc_scan`, `nc_alloc`, `nc_free`.
No C++ ABI, Rust enum, exception, string terminator or cross-runtime allocator ownership.
Use caller buffers; function-local state makes separate calls thread-safe. Invalid memory
pointers or overlapping buffers violate the C contract and cannot be validated by Rust.
Return value >=0 is a byte length; zero is a successful empty message. `-1` invalid,
`-2` too long, `-3` no readable code. SVG size query uses output NULL/capacity zero.
Native uses native `size_t`; wasm32 uses 32-bit sizes. Wasm memory views must be refreshed
after allocations, as memory growth invalidates old JS views.

## Compatibility and limitations

Legacy v0 is archived in the original JS sources; no silent conversion or auto-detection.
V1 uses eight rings with merged round-capped arcs, preserving the original visual principle.
Format version changes are required for incompatible geometry/packet changes.

No promise of decoding under glare, severe blur, occlusion, inverted colors, extreme
perspective, lens distortion or clutter. Global luminance clustering and reliance on clean ink bounds are known limits. Camera animation is UI behavior, independent of encoded data. The demo requires
three consecutive equal validated frames and processes frames off the UI thread. It never
automatically opens decoded URLs. Production acceptance requires a labelled real-photo
corpus, device testing, latency distribution and false-positive measurements.
