# Releasing NanoCode

User-facing installation/release instructions exist in all nine [guides](../README.md). This file is the maintainer runbook; the protocol reference and legal license remain canonical English sources.

1. Update the workspace npm version, `packages/js/package.json`, `crates/nanocode/Cargo.toml`, Cargo.lock, Android Maven version, Flutter pubspec/podspec versions, shared examples and all locale status paragraphs. `node scripts/check-version.mjs` checks packaged version agreement.
2. Run the checks in CONTRIBUTING.md. Build Apple with the five targets in `scripts/build-apple.sh`; run `swift test --package-path packages/apple`. Android needs JDK 17, Gradle 8.11.1, SDK 35, CMake 3.22.1 and NDK 28.2.13676358. Flutter source checkouts need assembled binary artifacts before device builds.
3. Push a `vX.Y.Z` tag matching the manifest version. The release workflow builds native macOS ARM64/x86_64, Linux ARM64/x86_64 and Windows x86_64; Apple iOS device and ARM64/x86_64 simulator XCFramework; Android arm64-v8a/armeabi-v7a/x86_64/x86 AAR and file Maven repository; npm/WASM; a native Flutter archive. Flutter Windows/Linux artifacts are x86_64; do not describe the archive as ARM64 desktop support.
4. Review the **draft** release, CI logs, SHA256SUMS.txt, ABI exports and real-device camera behaviour. Publish only the assets whose checks actually succeeded. A green build is not a scan-reliability certification.
5. Optional registry publishing uses maintainer-owned names and credentials. The JS scoped package is `@nikgariel/nanocode`; verify scope ownership. Reserve the crate/pub package names before replacing path installation with registry commands. Maven Central requires namespace verification, signing and publication metadata. Never substitute an unrelated package with a similar name.
6. Apple archives contain a local Swift package and XCFramework. They can be added in Xcode immediately after extraction. A remote SwiftPM binary distribution needs a dedicated repository manifest with the exact release URL and `swift package compute-checksum` of that version's XCFramework ZIP. No fabricated checksum or nonfunctional remote package URL is shipped here.

### Recovering a failed build

For a transient download or runner failure, use **Re-run failed jobs** on the existing Actions run; successful jobs and their artifacts are retained. If the workflow itself needs a fix, commit the fix to `main`, then run **Release artifacts → Run workflow** from `main` with the existing version tag in the `tag` field. The workflow checks out that exact tag for every SDK and verifies its manifest version. Do not move a published tag to repair build infrastructure. Source or API changes require a new version and tag. A release is created only after all required jobs succeed.

## GitHub Pages

Target repository: `NikGariel/nanocode`, independent of the legacy `nanocodegenerator`. In repository Settings → Pages, choose GitHub Actions. The Pages workflow deploys `dist/`, with `SITE_URL` derived from configure-pages. Enable Pages only after the new repository exists. The expected project address is `https://nikgariel.github.io/nanocode/`; it is not live merely because the build succeeds.

All page routes are physical HTML files. README and documentation use the default project address; customize `SITE_URL` when moving to another host. The root pages are English aliases with canonical links to `/en/`; no forced language redirects.

`/nanocode/robots.txt` is not an origin-root robots policy. On the owner's site or custom domain, deploy a root `/robots.txt` allowing crawling and pointing to `/nanocode/sitemap.xml` (or the corresponding custom-domain URL). Submit the sitemap in verified search-console accounts if desired. llms.txt is an optional documentation index, not a ranking mechanism.

Sources checked for this implementation: [GitHub Pages workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages), [Google multilingual sites](https://developers.google.com/search/docs/specialty/international/managing-multi-regional-sites), [Google AI features](https://developers.google.com/search/docs/appearance/ai-features), [OpenAI crawler roles](https://developers.openai.com/api/docs/bots). Search crawling and model-training crawlers are distinct; no inclusion or recommendation is guaranteed.
