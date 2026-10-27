# Security

> Spanish version: [SECURITY.md](SECURITY.md). The Spanish version is authoritative. In case of conflict, the Spanish version prevails. [LICENSE](LICENSE) prevails over this document.

## Supported versions

The main branch and the latest `0.1.x` version receive security fixes. There is no service-level agreement (SLA) or commercial support.

## Reporting a vulnerability

Do not open a public issue with exploitable details.

Use **GitHub Security Advisories** for the repository:

1. Go to the **Security** tab.
2. Open **Advisories**.
3. Create a private report.

Include:

- affected version;
- steps to reproduce;
- expected impact;
- environment (Windows, WebView2, application version);
- relevant logs without personal data or secrets.

Disclosure is coordinated and performed on a reasonable-effort basis. Response times are not guaranteed.

## Scope

In scope:

- application code;
- Tauri configuration;
- Content Security Policy;
- capabilities;
- CI and release workflows;
- dependency and release configuration.

Out of scope:

- Windows, WebView2, and other system components;
- Git and configured remotes;
- services, commands, and endpoints configured by the user;
- third-party dependencies, unless the issue is caused by Cardumen Deck's use of them;
- compromised devices or compromised user accounts.

## Security posture

Cardumen Deck implements the following measures:

- Restrictive Content Security Policy in the Tauri window.
- Scoped capabilities for the features used by the interface.
- No remote backend or telemetry.
- Locally stored data.
- Lockfiles for Rust and npm dependencies.
- Dependabot, security alerts, and security updates configured.
- Secret scanning and push protection configured on GitHub.
- CodeQL configured for JavaScript/TypeScript.
- CI configured for lint, type-check, build, formatting, Clippy, and tests.
- SHA256 checksums in releases.
- Code signing pending approval by SignPath Foundation.

## What is not claimed

Cardumen Deck does not claim or guarantee:

- a third-party security audit;
- certification under a security standard;
- freedom from vulnerabilities;
- a security sandbox;
- encryption of configuration or logs at rest;
- protection against malicious commands executed by the user;
- protection against compromised dependencies;
- protection against a compromised Windows, WebView2, or Git;
- an SLA, commercial support, or a bug bounty;
- indemnification of the user or third parties.

## Trust model

- The application runs with the Windows user's privileges.
- Service commands run with those same privileges.
- Commands can access files, credentials, and networks available to the user.
- The Job Object is used for process management, not as a security sandbox.
- Git operations can modify the repository and the remote.
- Health checks can reach configured internal or external endpoints.
- Logs and configuration may contain sensitive data.

## Data and secrets

- `config.json` stores environment variables without encryption.
- Logs may contain commands, paths, process output, and secrets printed by a service.
- Production secrets should not be used in the configuration.
- Git Credential Manager or another system helper may manage Git credentials; Cardumen Deck does not store them.
- Device encryption, a least-privilege account, and backups are recommended.

See [PRIVACY.en.md](PRIVACY.en.md) for data and network details.

## Network

The application only uses the network for:

- configured HTTP/TCP health checks;
- Git operations against configured remotes;
- opening URLs in the default browser.

Windows, WebView2, Git, and configured endpoints may have their own policies and connections.

## Release integrity

- Each release is configured to publish the portable build, NSIS installer, and `SHA256SUMS.txt`.
- Code signing through SignPath Foundation is pending.
- While unsigned, Windows SmartScreen may show warnings.
- Verify the SHA256 before running a downloaded binary.

See [CODE_SIGNING.md](CODE_SIGNING.md).

## Supply chain

- Lockfiles are used for dependencies.
- Dependabot is configured for npm, Cargo, and GitHub Actions.
- Secret scanning and push protection are enabled.
- CodeQL is configured for JavaScript/TypeScript.
- The application is distributed as a Windows binary and does not include telemetry.

These measures reduce risk but do not guarantee that a dependency or workflow is secure.

## Real development use

Cardumen Deck is a local development tool. It is not a managed service or a production component. You may use it in your real development workflow, but you are responsible for commands, credentials, data, and backups.

If you use it on a company device or with company code, that company's internal policies may apply.

## Support

Support is community-based and best effort. There is no SLA or commercial support. See [SUPPORT.en.md](SUPPORT.en.md).

This document is not legal advice. See [LEGAL.en.md](LEGAL.en.md).
