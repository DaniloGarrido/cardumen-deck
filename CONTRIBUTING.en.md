# Contributing

> Spanish version: [CONTRIBUTING.md](CONTRIBUTING.md). The Spanish version is authoritative. In case of conflict, the Spanish version prevails.

Thank you for contributing to Cardumen Deck.

## Before you start

- Search existing issues before opening a new one.
- For large changes, open an issue first to agree on the approach.
- Do not include secrets, tokens, certificates, or personal paths.
- Do not publish customer data, confidential code, or internal information.

## Environment

Requirements:

- Windows 10/11.
- Node.js 24+ and npm 10+.
- Rust stable 1.85+.
- Visual Studio Build Tools with C++.
- WebView2 Runtime.
- Git for Windows.

## Development

```powershell
npm install
npm run dev
```

## Verification

Before opening a pull request:

```powershell
npm run lint
npm --workspace FrontEnd run check
npm --workspace FrontEnd run build
cargo fmt --manifest-path Backend/Cargo.toml -- --check
cargo clippy --manifest-path Backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path Backend/Cargo.toml --lib
```

## Artificial intelligence-assisted contributions

If you use artificial intelligence tools:

- review the result before submitting it;
- verify that licenses and provenance are compatible;
- do not include proprietary, confidential, or unauthorized code;
- add or update tests where appropriate;
- describe the change and its impact;
- do not paste confidential code into unapproved tools.

Maintainers may reject contributions that do not meet these requirements. See [AI.en.md](AI.en.md).

## Security

- Do not include secrets in issues, logs, screenshots, or pull requests.
- For vulnerabilities, use GitHub Security Advisories. Do not open a public issue with exploitable details.
- If the change affects permissions, network, command execution, Git, or file handling, describe the security impact.
- See [SECURITY.en.md](SECURITY.en.md).

## Pull requests

- Keep the change focused.
- Explain what problem it solves.
- Include steps to test it.
- Attach a screenshot or video if the interface changes.
- Update documentation if behavior changes.
- Indicate whether the change was assisted by artificial intelligence.

## Commits

Use clear imperative messages. Examples:

- `feat: add service health filters`
- `fix: keep service actions aligned on narrow cards`
- `docs: document signing policy`

## Licenses and provenance

- By contributing, you agree that your contribution will be distributed under the project's MIT license.
- Do not submit code with incompatible licenses.
- Retain third-party copyright notices and licenses where applicable.
- See [THIRD_PARTY_NOTICES.en.md](THIRD_PARTY_NOTICES.en.md).
