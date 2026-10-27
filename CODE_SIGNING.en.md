# Code signing policy

> Spanish version: [CODE_SIGNING.md](CODE_SIGNING.md). The Spanish version is authoritative. In case of conflict, the Spanish version prevails.

## Current status

Versions are built by GitHub Actions. Version `0.1.0` was published without a digital signature as a step before adopting SignPath Foundation. Signing will start with version `0.1.1`.

Release `v0.1.0`: <https://github.com/DaniloGarrido/cardumen-deck/releases/tag/v0.1.0>.

## Statement

Signed versions will be published with free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

The publisher users see on signed versions will be **SignPath Foundation**.

## Scope

- Only binaries built from this repository's source code are signed.
- The portable executable and the NSIS installer are signed.
- Third-party artifacts are not signed.
- The installer may include unsigned third-party components, as allowed by SignPath Foundation policy.
- Certificates, tokens, and private keys are not uploaded to the repository.
- Each version includes `SHA256SUMS.txt` to verify the integrity of the published files.

## Roles

- **Author:** [@DaniloGarrido](https://github.com/DaniloGarrido), repository owner. Can modify code without additional review.
- **Reviewer:** [@DaniloGarrido](https://github.com/DaniloGarrido), repository owner. Reviews changes proposed by non-maintainers.
- **Approver:** [@DaniloGarrido](https://github.com/DaniloGarrido), repository owner. Manually approves each signing request.

Each signing request requires manual approval. There is no automatic release signing.

## Privacy

The privacy policy is in [PRIVACY.en.md](PRIVACY.en.md). Cardumen Deck has no telemetry, analytics, remote crash reporting, or proprietary backend.

## Build requirements

- The build runs on GitHub Actions using GitHub-hosted runners (`windows-latest`).
- The version is built from a `v*` tag that matches the application version.
- The main executable is signed before the NSIS installer is generated.
- The installer is signed after it is generated.
- Unsigned artifacts are uploaded as GitHub Actions artifacts before being submitted to SignPath.
- The build and signing flow are recorded in the GitHub Actions history.

## Binary metadata

Signed binaries must have consistent metadata:

- Product name: `Cardumen Deck`.
- Product version: the same as the release tag.
- File version: the same as the product version.

The SignPath artifact configuration must enforce these values before signing.

## Verification

```powershell
signtool verify /pa /v CardumenDeck-v0.1.1-windows-x64-portable.exe
signtool verify /pa /v CardumenDeck-v0.1.1-windows-x64-setup.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.1-windows-x64-portable.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.1-windows-x64-setup.exe
```

On a signed version, `Get-AuthenticodeSignature` must show `Valid` and the publisher must include **SignPath Foundation**.

## SignPath checklist

- [x] OSI-approved license (MIT).
- [x] Public repository.
- [x] README and build documentation.
- [x] Signing policy created (this file).
- [x] Versions built by GitHub Actions (portable + installer).
- [ ] Application submitted and approved by SignPath Foundation.
- [ ] Multi-factor authentication (MFA) enabled for maintainers on GitHub and SignPath.
- [ ] First signed version.
