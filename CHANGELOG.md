# Changelog

Localized release and usage guidance: [language index](README.md).

## 0.2.2

- Exclude the safe logo centre from the first geometry/palette fit on illumination-corrected fallback frames.
- Match the illumination envelope to arc thickness so larger codes with display bands do not acquire artificial ink.
- Add the exact landing-page N logo to the benchmark and regression tests, including blur/banding at 256, 384 and 512 pixels.

## 0.2.1

- Faster markerless geometry fitting and protected-header-first orientation search; bilinear sampling.
- Local background-envelope correction for brightness gradients and screen refresh bands, without changing the encoded format.
- Browser: fresh-frame callbacks, one frame in flight, reusable buffers, tracked-region confirmation and 640/960-pixel analysis.
- Camera requests 720p/15 fps; completion-driven duty limits, slower search after 15 seconds, camera shutdown on confirmation and a 60-second idle cutoff.
- Versioned site asset URLs prevent mixing cached workers/WASM with an updated page.
- Reproducible synthetic benchmark, screen-degradation regression fixtures and worker/scheduling integration checks. See docs/SCANNER-PERFORMANCE.md.

## 0.2.0

- S/M/L/XL profiles: 8/16/24/40 bytes, 4/6/8/10 rings, automatic format detection.
- Merged rounded arcs, configurable palettes, neutral-background recommendations and a safe centre for logos.
- Rust core, additive C ABI, packaged JavaScript/WASM, Swift, Android JNI and native Dart FFI adapters.
- Camera selection, monitor-region search, bounded perspective refinement and temporal confirmation.
- Static project site: live scenario codes, standard QR link, generator, scanner and documentation in nine languages.
- Localized READMEs, sitemap, reciprocal language links, structured metadata and AI-readable documentation.
- CI, GitHub Pages deployment and tagged draft releases with per-platform artifacts and SHA-256 checksums.

## Legacy

v1 remains readable. Original JavaScript v0 codes are not compatible with this SDK. Apple App Clip is a visual reference, not an interoperable format.
