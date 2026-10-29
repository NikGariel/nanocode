# NanoCode for Android

[en](https://nikgariel.github.io/nanocode/en/docs.html) · [ru](https://nikgariel.github.io/nanocode/ru/docs.html) · [zh-CN](https://nikgariel.github.io/nanocode/zh-CN/docs.html) · [es](https://nikgariel.github.io/nanocode/es/docs.html) · [fr](https://nikgariel.github.io/nanocode/fr/docs.html) · [de](https://nikgariel.github.io/nanocode/de/docs.html) · [pt-BR](https://nikgariel.github.io/nanocode/pt-BR/docs.html) · [ja](https://nikgariel.github.io/nanocode/ja/docs.html) · [ko](https://nikgariel.github.io/nanocode/ko/docs.html)

Set `ANDROID_NDK_HOME` to NDK 28.2.13676358, install the four Rust Android targets, run `sh scripts/build-android.sh`, then `gradle -p packages/android :nanocode:assembleRelease :nanocode:publish`. Java API is callable from Kotlin. Release AAR and file Maven repository include four ABIs. JDK 17, Gradle 8.11.1, SDK 35; minSdk 23. CameraX is the host app’s responsibility.

Experimental version 0.2.1. See the localized guide for API examples, formats, palettes, permissions and security. Registry publication and real-device testing are separate from source availability. MIT.
