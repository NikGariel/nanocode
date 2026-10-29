# Scanner performance and camera work

The 0.2.1 reader keeps the v1/v2 wire format and C ABI unchanged. It improves acquisition and decoding; it does not claim universal readability or parity with a platform QR scanner.

## Practices reviewed

- [ZXing HybridBinarizer](https://github.com/zxing/zxing/blob/master/core/src/main/java/com/google/zxing/common/HybridBinarizer.java): use local brightness information under shadows instead of a single global black point. NanoCode has two data inks plus background, so ordinary binary thresholding would discard information. Our fallback estimates a local background envelope and preserves the three classes. This is an independent implementation, not a copy of ZXing's binarizer.
- [ZXing-C++ reading pipeline](https://github.com/zxing-cpp/zxing-cpp/blob/master/core/src/ReadBarcode.cpp): bounded attempts at multiple image scales. The web camera uses 640-pixel analysis and a 960-pixel retry every fourth attempt, without changing the preview's aspect ratio.
- [Google ML Kit barcode guidance](https://developers.google.com/ml-kit/vision/barcode-scanning/android): avoid excessive capture resolution, keep only the latest frame, work off the UI thread, and confirm repeated reads. Adequate pixel size and focus remain necessary.
- [Apple TN2445](https://developer.apple.com/library/archive/technotes/tn2445/_index.html): discard late frames and reduce frame rate instead of allowing chronic processing overload. Native AVFoundation hosts should set `alwaysDiscardsLateVideoFrames`; CameraX hosts should use `STRATEGY_KEEP_ONLY_LATEST` and close each ImageProxy.

## Changes

Geometry uses the uniformly spaced profile radii to find the nearest ring in constant time and a smaller spatial sample. Orientation search samples the protected NC header before visiting all payload cells. Bilinear luminance sampling reduces sensitivity to fractional pixel position. All accepted reads still pass complete ECC, length, padding and CRC checks; no header-only or cached-payload acceptance is allowed.

After a normal miss, a separable sliding maximum estimates the light background locally. Dividing by that envelope compensates for gradual exposure variation and some display banding. A second polarity handles a dark quiet zone. The algorithm cannot recover data erased by saturation, missing arcs or severe blur. Each refinement pass has a finite work budget.

The browser worker first retries a padded region around the previous detection, then falls back to the complete image. It returns the transferred grayscale buffer for reuse. There is exactly one in-flight frame, and `requestVideoFrameCallback` (with a currentTime fallback) avoids confirming the same video frame twice. The latest frame is captured only after the previous attempt and a rest interval finish.

The web demo requests 1280×720, ideal 15 fps/max 24 fps; browsers may negotiate another size/rate. Continuous focus/exposure are requested only when exposed by the track. The analysis duty target is at most 30%, then 15% after 15 seconds, with a minimum 90 ms rest. This covers capture/conversion plus worker turnaround, not total CPU/GPU/camera energy. After two matching reads the stream and worker stop, retaining the processed frame and result. After 60 seconds without confirmation the session stops and can be restarted. Hidden pages stop the camera too.

## Reproducible measurements

Run `node scripts/bench-scanner.mjs [path/to/nanocode.wasm]`. The same deterministic fixtures can be tested against the old release binary and the new one. Measurements below are medians of three attempts in Node v21.7.3, macOS ARM64; they exclude camera acquisition and rendering. They are synthetic fixtures, not an iPhone field study.

| Fixture | 0.2.0 | 0.2.1 | Result |
| --- | ---: | ---: | --- |
| Monitor, S light | 103.1 ms | 14.3 ms | both read |
| Monitor, S dark | 676.3 ms | 28.7 ms | both read |
| Monitor, M light | 120.8 ms | 15.6 ms | both read |
| Monitor, M dark | 120.1 ms | 14.4 ms | both read |
| Monitor, L light | 110.4 ms | 15.2 ms | both read |
| Monitor, L dark | 108.3 ms | 14.2 ms | both read |
| Monitor, XL light | 184.8 ms | 12.7 ms | both read |
| Monitor, XL dark | 184.9 ms | 11.6 ms | both read |
| Close-up M | 111.8 ms | 7.7 ms | both read |
| Exposure gradient + blur | 12.6 ms, miss | 19.3 ms | now reads |
| Screen bands + blur | 635.2 ms, miss | 24.4 ms | now reads |
| Synthetic moiré + blur | 102.4 ms | 7.6 ms | both read |
| Washed-out + blur | 131.8 ms | 6.5 ms | both read |
| Blank frame | 3.2 ms | 3.6 ms | both reject |
| Stripes without a code | 7.6 ms | 26.7 ms | both reject |

Fallback correction makes some negative frames slower; the scheduler limits repeated work. The complete 15-fixture suite went from 13/15 to 15/15 expected outcomes. This small constructed set does not estimate real-world accuracy.

An additional manual check used an iPhone selected as the camera in Safari on Mac, reading ORBIT:EVT:042 from a monitor: 640×360, 28 ms for the confirming frame, two matching reads, followed by automatic camera shutdown. Processing occurred on the Mac, not on the phone.

## Device validation still required

For Safari on iPhone, compare identical payload sizes and displayed diameters with QR using the same distance, monitor brightness, focus, angle and lighting. Record time to two matching reads (including misses), decode latency distribution, false accepts and energy/thermal behavior during a timed no-code scan. Measure the old and new versions on the same device. Do not infer temperature or QR parity from desktop latency.

Use **Save frame** before confirmation to retain an actual failing camera image. Keep private surroundings out of reports; real images are not uploaded by the scanner. Synthetic regressions, worker transport tests and scheduler tests run in `npm test`. Native mobile apps must implement their own capture lifecycle; a synchronous C/Swift/Dart call does not automatically limit camera work.

## 0.2.2: centre-logo regression

After deployment, the user confirmed fast reading without a logo but reported failure with the landing-page N. An additional fixture reproduced a miss in 0.2.1 when that exact polygon was combined with display banding and blur. Corrected-image passes now exclude the safe centre from their first geometry/palette fit, instead of allowing logo pixels to consume the region-search budget. The envelope radius was reduced to just exceed half the maximum arc stroke plus blur; the previous radius also caused a failure at 512 pixels.

The extended 21-fixture benchmark passes all expected outcomes locally. The N/banding fixture changes from a miss (100.4 ms median) to a validated read (76.1 ms); plain N reads in 10.8 ms and N inside the monitor scene in 14.2 ms. Ordinary monitor scenes remain approximately 12–29 ms. Rust regressions use the real landing payload ORBIT:EVT:042 and N polygon at 256, 384 and 512 pixels, with and without blur/banding. The user’s actual failing camera frame and a phone retest are still needed to establish that the observed field case is resolved.


## 0.2.3: captured monitor frame with N

The subsequent user capture still failed in 0.2.2 despite the synthetic improvements. Version 0.2.3 estimates the illumination envelope inside edge-derived code regions as well as the whole frame. It compares the bounding-box and hull-ellipse seeds instead of unconditionally trusting the hull. The radial proposal gate allows blurred ink to occupy a wider footprint; acceptance still requires the complete ECC/CRC checks. Region correction has a shared cap of 12,000 geometry evaluations per raw-image pass.

The unchanged private 720×1280 capture now decodes ORBIT:EVT:042 (M) with no corrected bits, as does its 540×960 bilinear downsample. The 360×640 downsample still fails. The web scheduler therefore keeps the next frame at 960 pixels after a first detection, instead of reverting immediately to 640. This also avoids losing the first detection from the temporal confirmation window. Existing rest intervals and camera shutdown are preserved.

In Node v21.7.3 on macOS ARM64, one replay through the actual browser worker took 49.8 ms for the 540×960 frame and 16.0 ms for tracked confirmation. A following blank image returned no detection. These are individual desktop measurements, not phone timing or an energy benchmark. All 21 synthetic benchmark outcomes and the existing Rust/JS tests pass. A live iPhone Safari retest is still required.

Private captures are intentionally absent from this repository. To replay a local image converted to headerless row-major 8-bit grayscale:

```sh
node scripts/test-camera-frame.mjs /private/path/frame.gray 540 960 ORBIT:EVT:042
```

The script verifies full-frame detection, tracked confirmation, and rejection after the code disappears. It does not upload or copy the image into the repository.
