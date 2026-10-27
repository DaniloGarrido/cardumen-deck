# Code signing policy

> Spanish version: [CODE_SIGNING.md](CODE_SIGNING.md). The Spanish version is authoritative. In case of conflict, the Spanish version prevails.

## Current status

Versions are built by GitHub Actions and may currently be published without a digital signature. The initial `0.1.0` version may be unsigned.

## Goal

The project aims to be accepted by **SignPath Foundation**, which provides free code signing for open-source projects.

If approved:

- The publisher users see will be **SignPath Foundation**.
- Signing will happen inside the GitHub Actions workflow.
- Private keys will never be stored in the repository.

## Rules

- Only binaries built from commits in this repository are signed.
- Third-party artifacts are not signed.
- Certificates, tokens, and private keys are not uploaded.
- Each version includes `SHA256SUMS.txt` for the portable version and the NSIS installer.
- The build must be reproducible from a Git tag.

## Verification

```powershell
signtool verify /pa /v CardumenDeck-v0.1.0-windows-x64-portable.exe
signtool verify /pa /v CardumenDeck-v0.1.0-windows-x64-setup.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.0-windows-x64-portable.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.0-windows-x64-setup.exe
```

## Checksums

Each version includes `SHA256SUMS.txt` to verify the integrity of the portable version and the installer.

## SignPath checklist

- [x] OSI-approved license (MIT).
- [x] Public repository.
- [x] README and build documentation.
- [x] Signing policy created (this file).
- [ ] Versions built by GitHub Actions (portable + installer).
- [ ] Application submitted and approved by SignPath Foundation.
- [ ] Multi-factor authentication (MFA) enabled for maintainers.
- [ ] First signed version.
