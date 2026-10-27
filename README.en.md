<p align="center">
  <img src="docs/assets/octopus.webp" alt="Cardumen Deck" width="112" />
</p>

<h1 align="center">Cardumen Deck</h1>

<p align="center">
  <strong>Many services, one current.</strong>
</p>

<p align="center">
  <a href="https://github.com/DaniloGarrido/cardumen-deck/actions/workflows/ci.yml"><img src="https://github.com/DaniloGarrido/cardumen-deck/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/DaniloGarrido/cardumen-deck/releases"><img src="https://img.shields.io/github/v/release/DaniloGarrido/cardumen-deck?display_name=tag&sort=semver" alt="Latest release" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/DaniloGarrido/cardumen-deck" alt="MIT license" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4" alt="Windows 10/11" />
  <img src="https://img.shields.io/badge/telemetry-none-success" alt="No telemetry" />
</p>

<p align="center">
  <a href="README.md">Español</a> ·
  <a href="#download">Download</a> ·
  <a href="PRIVACY.en.md">Privacy</a> ·
  <a href="SECURITY.en.md">Security</a> ·
  <a href="AI.en.md">AI</a> ·
  <a href="LEGAL.en.md">Legal</a> ·
  <a href="CHANGELOG.md">Changelog</a>
</p>

A local Windows dashboard to start, monitor, and operate development services.

> **Status:** `0.1.x`, evolving and usable for daily development. Windows-only for now.

> **Intended use:** local development tool. It is not a managed service or a production component. You may use it in your real development workflow; you are responsible for commands, credentials, data, and backups.

![Cardumen Deck dashboard](docs/screenshots/dashboard.webp)

## Features

- Workspaces for grouping services.
- Start, stop, and restart local processes.
- Detect running services and reconcile orphaned processes.
- CPU, RAM, and process metrics.
- HTTP, TCP, and process health checks.
- Per-run logs and an integrated log viewer.
- Git integration: status, branches, checkout, pull, push, merge, and conflicts.
- Dependency and environment detection.
- Guided service creation.
- Light/dark themes and multiple palettes.
- Internal error and activity log.

![Cardumen Deck settings](docs/screenshots/settings.webp)

## Privacy and security

- **No telemetry:** no analytics, remote crash reporting, user identifiers, or backend of its own.
- **Limited network use:** only HTTP/TCP health checks, Git operations, and configured URL opening.
- **Local data:** configuration, state, and logs are stored on the device. There is no cloud sync.
- **No encryption at rest:** configuration and logs are not encrypted and may contain secrets.
- **No sandbox:** commands run with your Windows user privileges.

See [PRIVACY.en.md](PRIVACY.en.md) and [SECURITY.en.md](SECURITY.en.md) for details.

## Download

Use the latest published version:

- [Download the latest version](https://github.com/DaniloGarrido/cardumen-deck/releases)
- **Portable:** `CardumenDeck-v*-windows-x64-portable.exe`. No installation required.
- **Installer:** `CardumenDeck-v*-windows-x64-setup.exe`. NSIS installer.
- Every release includes `SHA256SUMS.txt`.

> The `0.1.x` release may be unsigned. Windows SmartScreen may show a warning. Verify the SHA256 before running and do not disable your antivirus.

### Verify SHA256

```powershell
Get-FileHash .\CardumenDeck-v0.1.0-windows-x64-portable.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

## Requirements

### To use the application

- Windows 10 or 11.
- WebView2 Runtime. Modern Windows 10/11 already includes it.
- Git for Windows if you plan to use Git features.
- Git Credential Manager or configured credentials for remote `push`/`pull`.

You do not need Node.js or Rust to use the portable or installer.

### To develop

- Node.js 24+ and npm 10+.
- Rust stable 1.85+.
- Visual Studio Build Tools with C++.
- WebView2 Runtime.
- Git for Windows.

## Getting started

1. Open Cardumen Deck.
2. Create a workspace.
3. Add a service with a name, working directory, and command.
4. Start the service from its card.
5. Configure an HTTP, TCP, or process health check if you want status monitoring.

## Data and privacy

- Configuration: `%APPDATA%\CardumenDeck\config.json`
- Runtime: `%APPDATA%\CardumenDeck\runtime.json`
- Logs: `%LOCALAPPDATA%\CardumenDeck\logs\`

Cardumen Deck has no remote backend, analytics, or telemetry. Health checks only contact the URLs or ports you configure. Git operations contact the remote only when you run them.

Environment variables are stored in `config.json` without encryption. Logs may contain commands, paths, process output, and secrets printed by a service. Review and protect those files before sharing them.

Windows, WebView2, Git, and configured endpoints may have their own policies and connections. See [PRIVACY.en.md](PRIVACY.en.md) for the full data and network table.

If you are upgrading from an older version, configuration, runtime state, and dashboard logs are migrated automatically on first launch.

## Uninstall

- **Portable:** delete the `.exe`.
- **Installer:** use “Add or remove programs” or the NSIS uninstaller.
- To remove your data as well, delete:
  - `%APPDATA%\CardumenDeck`
  - `%LOCALAPPDATA%\CardumenDeck`

After migration, the old `ServiceDashboard` folders can be deleted if you no longer need them.

## Known limitations

- Windows only.
- No auto-update; download a new version manually.
- The `0.1.x` version may be unsigned.
- Git features require Git and configured credentials for remote operations.
- The NSIS installer and portable build share the same local configuration.
- No sandbox: commands run with user privileges.
- Configuration and logs are not encrypted.
- No independent audit, certification, SLA, or bug bounty.
- No central management or telemetry for managing multiple installations.

## Troubleshooting

### SmartScreen blocks the executable

This is expected for unsigned binaries. Verify the SHA256 and use “More info” → “Run anyway” only if you trust the file.

### The application does not open or is blank

Check that WebView2 Runtime is installed and up to date. Also check the logs in `%LOCALAPPDATA%\CardumenDeck\logs\`.

### Git is not detected

Install Git for Windows and make sure `git` is on `PATH`. Restart Cardumen Deck afterwards.

### A service does not start

Check the working directory, command, arguments, and run log from the service card. If the command works in a terminal but not in the application, check `PATH` and environment differences.

### Port 1420 is busy in development

Stop the process using the port or restart the development session.

## Stack

- **Frontend:** Vue 3, TypeScript, Vite, Lucide.
- **Backend:** Rust, Tauri 2, Tokio.
- **UI:** WebView2.
- **Data:** local JSON files, no remote backend.

## Development

```powershell
git clone https://github.com/DaniloGarrido/cardumen-deck.git
cd cardumen-deck
npm install
npm run dev
```

## Build

From the repository root:

```powershell
# Portable executable
npm run build -- --no-bundle

# Portable + NSIS installer
npm run build
```

Output paths:

- Portable: `Backend/target/release/cardumen-deck.exe`
- Installer: `Backend/target/release/bundle/nsis/`

## Verification

```powershell
npm run lint
npm run check
npm run build
cargo fmt --manifest-path Backend/Cargo.toml -- --check
cargo clippy --manifest-path Backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path Backend/Cargo.toml --lib
```

CI is configured to run frontend lint, type-check, frontend build, Rust formatting, Clippy, and backend tests on Windows.

## Versions and signing

The release workflow is configured to build the portable version, installer, and `SHA256SUMS.txt` from a `v*` tag.

Code signing through **SignPath Foundation** is pending approval. Until then, the publisher may be unknown and SmartScreen may warn. See [CODE_SIGNING.en.md](CODE_SIGNING.en.md).

## Security

Cardumen Deck uses a restrictive CSP, scoped capabilities, and no remote backend or telemetry. There is no sandbox, encryption at rest, or independent audit. To report vulnerabilities, use GitHub Security Advisories. Do not open a public issue with exploitable details. See [SECURITY.en.md](SECURITY.en.md).

## AI in development

Part of the code and documentation was developed with artificial intelligence assistance. The application does not use artificial intelligence at runtime and does not send prompts to artificial intelligence APIs. Maintainers review and test the code, but artificial intelligence may introduce errors. See [AI.en.md](AI.en.md).

## Contributing

Read [CONTRIBUTING.en.md](CONTRIBUTING.en.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Artificial intelligence-assisted contributions must follow the policy in [AI.en.md](AI.en.md).

## Support

Support is community-based and best effort. There is no SLA, commercial support, or bug bounty. See [SUPPORT.en.md](SUPPORT.en.md).

## Legal notice

Cardumen Deck is distributed under the MIT license. The software is provided “as is”, without warranty, and use is at your own risk. There is no affiliation with Microsoft, Tauri, Vue, Rust, Lucide, or GitHub. See [LEGAL.en.md](LEGAL.en.md).

## Structure

```text
.
├── Backend/            # Tauri + Rust
├── FrontEnd/           # Vue 3 + TypeScript + Vite
├── docs/               # Architecture, screenshots, and assets
├── .github/            # CI, releases, and templates
├── package.json
├── README.md
└── README.en.md
```

## Technical documentation

- [Architecture](docs/ARCHITECTURE.en.md)
- [Privacy](PRIVACY.en.md)
- [Security](SECURITY.en.md)
- [AI in development](AI.en.md)
- [Legal notice](LEGAL.en.md)
- [Third-party notices](THIRD_PARTY_NOTICES.en.md)
- [Support](SUPPORT.en.md)
- [Signing policy](CODE_SIGNING.en.md)
- [Changelog](CHANGELOG.md)

## Credits

- Mascot: Cardumen Deck octopus.
- Built with [Tauri](https://tauri.app/), [Vue](https://vuejs.org/), and [Rust](https://www.rust-lang.org/).

## License

MIT. See [LICENSE](LICENSE).
