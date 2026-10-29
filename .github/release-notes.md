Includes the 0.2.1 speed/power improvements plus a fix for logos under screen banding: corrected-image passes exclude the centre before geometry fitting, and the local illumination envelope follows arc thickness. The exact landing-page N is regression-tested at three sizes with blur and bands.

Scanner performance update. Encoded S/M/L/XL and legacy v1 compatibility are unchanged.

- Faster geometry fitting and protected-header-first orientation search, plus bilinear sampling.
- Local brightness correction for monitor gradients and refresh bands.
- Web demo: fresh-frame processing, tracked regions, adaptive rest intervals, 720p/15 fps camera request, automatic camera shutdown after confirmation and idle pause after 60 seconds.
- Regression tests and a reproducible synthetic benchmark: [measurements and limits](https://github.com/NikGariel/nanocode/blob/main/docs/SCANNER-PERFORMANCE.md).

Assets include native C/Rust libraries, Web/WASM npm archive, Swift/XCFramework, Android AAR/Maven archive and native Flutter bundle. Installation guides are available in nine languages. SHA256SUMS.txt covers all binary assets.

Experimental release. Desktop synthetic measurements are not an iPhone temperature or QR-parity guarantee. Native integrations must apply their own camera backpressure and lifecycle policy. Registry publication remains separate.
