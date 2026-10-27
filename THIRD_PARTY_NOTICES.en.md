# Third-party notices

> Spanish version: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). The Spanish version is authoritative. In case of conflict, the Spanish version prevails. [LICENSE](LICENSE) prevails over this document.

## Scope

Cardumen Deck uses third-party components. Each component retains its own license and terms. This document is a summary and does not replace the original licenses.

## Main components

| Component | Use | License or terms | Link |
|---|---|---|---|
| Tauri | Desktop framework | MIT or Apache-2.0 | https://github.com/tauri-apps/tauri |
| Vue | Interface | MIT | https://github.com/vuejs/core |
| Rust | Language and toolchain | MIT or Apache-2.0 | https://www.rust-lang.org/ |
| Lucide | Icons | ISC | https://github.com/lucide-icons/lucide |
| WebView2 | Windows interface engine | Microsoft terms | https://developer.microsoft.com/microsoft-edge/webview2/ |
| NSIS | Windows installer | NSIS license | https://nsis.sourceforge.io/ |

## Transitive dependencies

Exact dependencies and versions are recorded in:

- `Backend/Cargo.lock` for Rust.
- `package-lock.json` for npm.
- `Backend/Cargo.toml` and `package.json` for direct dependencies.

Exact licenses must be verified in the upstream projects and in the license files included by each dependency.

## Binary distribution

If you distribute Cardumen Deck or a modified version, you must retain the copyright notices and licenses required by third-party components. This document does not replace those notices.

## Trademarks

Trademarks, trade names, and logos belong to their respective owners. Cardumen Deck is not affiliated with or endorsed by those projects.

## Not legal advice

This document is not legal advice. For a license or distribution review, consult a qualified professional.
