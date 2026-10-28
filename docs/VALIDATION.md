# Validation record — 0.2.0

2026-10-28, macOS ARM64. This is evidence, not a production scan-rate claim. Localized capability/limitation guidance is in the [nine-language documentation](../README.md).

- **Rust:** 16 release tests pass. Exhaustive single/double-bit Hamming checks, all format boundaries, binary payloads and zero padding, palette rejection, white/dark/intermediate backgrounds, rotations, perspective, noise, modest blur/occlusion, codes surrounded by monitor UI and central high-contrast brand marks.
- **WASM:** actual ABI tests pass for generation, scanning, UTF-8, all four profile IDs, metadata, too-small output buffers, unsafe palettes, automatic format selection and background recommendations. One synthetic 960×640 monitor frame decoded in approximately 124 ms on this Mac after keeping the clean-frame fast path separate from logo-aware candidate refinement. This is neither p95 nor phone performance.
- **JavaScript packaging:** packed ESM/WASM consumer, exported APIs and unsafe logo URL rejection pass; TypeScript declarations checked with a strict consumer fixture.
- **Safari:** generated SVG rasterized by the browser and read back as `ORBIT:EVT:042`, format M; corresponding demo experience displayed. Static landing, scenario tabs, QR link and locale navigation checked. Camera reliability still needs a representative field dataset.
- **Apple:** XCFramework built with iOS ARM64, iOS simulator ARM64/x86_64 and macOS ARM64/x86_64 slices. SwiftPM wrapper tests passed on macOS ARM64 (generation, input rejection, background suggestions, empty-frame no-result).
- **Documentation:** nine complete localized guides/READMEs; 36 static language pages. Build check validates language fields, canonical/hreflang links, JSON-LD, Markdown sections, sitemap and internal file links.
- **Android / Flutter / Windows / Linux:** adapters and CI build jobs provided. Native Android AAR/NDK and Flutter device builds were not run on this Mac; actual release CI results and device tests must be checked before claiming support. Java/JNI was compiled and executed on this Mac: SVG, empty payload, successful grayscale scan with payload/metadata, and invalid input all passed. This host smoke test is not an Android device test.

The user previously reported improved reading of a monitor using an iPhone selected as a connected Mac camera. This is qualitative feedback, not measured success rate.

Not established: minimum print size, robust arbitrary scenery, severe glare/motion, p95 latency across devices, physical-print durability, false-positive rates on a real-world image corpus, or equivalence to Apple App Clip. CRC is not authentication. Legacy JavaScript v0 is unsupported.
