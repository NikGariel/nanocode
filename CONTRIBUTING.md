# Contributing

Guides: [English](docs/i18n/README.en.md#contributing-and-translations) · [Русский](docs/i18n/README.ru.md) · [简体中文](docs/i18n/README.zh-CN.md) · [Español](docs/i18n/README.es.md) · [Français](docs/i18n/README.fr.md) · [Deutsch](docs/i18n/README.de.md) · [Português](docs/i18n/README.pt-BR.md) · [日本語](docs/i18n/README.ja.md) · [한국어](docs/i18n/README.ko.md).

Run `npm ci`, `npm run build`, `npm test`, `node scripts/check-site.mjs`, `node scripts/test-package.mjs`, `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`.

The Rust core owns the format and reader. Keep ABI changes additive; preserve v1 decoding. The generator must never truncate a payload or silently accept a palette rejected by the assessment API. Camera tests should model an actual failure, not mirror implementation details. Camera frames shared in issues must contain no personal information.

## Documentation

Edit `site/content/locales.mjs` and `site/content/ui.mjs`, not generated `README.md`, `docs/i18n/README.*.md` or `dist/`. Shared code examples are in `site/content/examples.mjs`; builds generate all nine language variants. Keep API identifiers, commands and numbers unchanged. Native-speaker review is welcome. Check all translations when changing a factual claim.

## Release

See [release procedure](docs/RELEASING.md). A tag creates a draft release only after required jobs succeed. Package registry publication is a separate authenticated maintainer action. Do not publish undocumented or untested capabilities as supported.
