# NanoCode for JavaScript

Branded circular codes, powered by the same Rust engine as the native libraries.

Until a registry release is published, install the `.tgz` from a GitHub Release:

```sh
npm install ./nikgariel-nanocode-0.3.0.tgz
```

```js
import {loadNanoCode} from '@nikgariel/nanocode';
const code = await loadNanoCode();
document.querySelector('#code').innerHTML = code.svg('LOOP:RIDE:024');
const options = code.secondaryColors('#00a6a1', '#ffffff');
// Empty: change the primary colour or background first.
if (options.length) {
  const palette = {one:'#00a6a1', zero:options[0].color, background:'#ffffff'};
  const advice = code.assess(palette); // status, allowed, recommended, issues
  document.querySelector('#code').innerHTML = code.svg('LOOP:RIDE:024', palette);
}
const result = code.scan(grayscaleBytes, width, height); // null or verified payload
```

Bundlers can pass their resolved WASM URL to `loadNanoCode(url)`. Node.js can pass
`await readFile(new URL('./nanocode.wasm', import.meta.resolve('@nikgariel/nanocode')))`.
The synchronous scanner should run in a Worker for camera use. Camera permission,
frame capture and UI remain host-application responsibilities.

`secondaryColors(one, background = '#ffffff')` returns up to eight `{color, score}` suggestions, best brightness separation first. All score at least 55/255. `assess()` returns `incompatible` below 35, `caution` at 35–54 and `recommended` at 55+. `svg()` rejects incompatible palettes; caution is allowed but should be shown to the user. Scores are heuristics, not camera/print guarantees.

Optional `logoDataUrl` accepts PNG/JPEG/WebP (base64, up to 1 MB) in the protected centre.
No network requests, analytics or payload execution are performed by the core.
Full docs: https://github.com/NikGariel/nanocode

## Documentation languages

[en](https://nikgariel.github.io/nanocode/en/docs.html) · [ru](https://nikgariel.github.io/nanocode/ru/docs.html) · [zh-CN](https://nikgariel.github.io/nanocode/zh-CN/docs.html) · [es](https://nikgariel.github.io/nanocode/es/docs.html) · [fr](https://nikgariel.github.io/nanocode/fr/docs.html) · [de](https://nikgariel.github.io/nanocode/de/docs.html) · [pt-BR](https://nikgariel.github.io/nanocode/pt-BR/docs.html) · [ja](https://nikgariel.github.io/nanocode/ja/docs.html) · [ko](https://nikgariel.github.io/nanocode/ko/docs.html)
