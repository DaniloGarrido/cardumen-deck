# Privacy Policy

> Spanish version: [PRIVACY.md](PRIVACY.md). The Spanish version is authoritative. In case of conflict, the Spanish version prevails. [LICENSE](LICENSE) prevails over this document.

## Scope

This policy describes what data Cardumen Deck uses, where it is stored, and what network connections it may make. It applies to the desktop application and its local configuration.

## Summary

- Cardumen Deck **does not include telemetry, analytics, remote crash reporting, or a backend of its own**.
- There are no accounts, sign-in, cloud sync, or advertising.
- Application data is stored locally.
- The application only uses the network for specific actions: health checks, Git operations, and opening URLs.
- Windows, WebView2, Git, and the services you run may have their own policies and connections.

## What the application does not do

Cardumen Deck does not:

- send telemetry or usage metrics;
- use analytics or advertising SDKs;
- send crash reports to a server of its own;
- create user identifiers;
- sync data to the cloud;
- download remote configuration;
- check for updates automatically;
- sell or share data with third parties.

## Local data

| Data | Location | Purpose | Encrypted |
|---|---|---|---|
| Configuration | `%APPDATA%\CardumenDeck\config.json` | Services, workspaces, settings, and environment variables | No |
| Runtime state | `%APPDATA%\CardumenDeck\runtime.json` | PIDs, statuses, log paths, and timestamps | No |
| Service logs | `%LOCALAPPDATA%\CardumenDeck\logs\` | Command, arguments, working directory, output, and errors | No |
| WebView2 profile | `%LOCALAPPDATA%\CardumenDeck\` | Theme, view, sidebar, and local preferences | Per WebView2/Windows |
| Clipboard | System memory | Read or write text when the user requests it | Not applicable |

Environment variables defined for a service are stored in `config.json` **without encryption**. Do not store production secrets in that file.

Logs may contain sensitive information: commands, paths, service names, process output, and errors. They may also contain secrets if a service prints them. Review and protect logs before sharing them.

## Network connections

| Action | Destination | Data sent | When it happens |
|---|---|---|---|
| HTTP health check | Service's configured URL | `GET` request with `User-Agent: CardumenDeck/0.1` | While health checking is active |
| TCP health check | Configured host and port | TCP connection | While health checking is active |
| Git operations | Repository's configured remote | Standard Git data | When the user runs `fetch`, `pull`, or `push` |
| Open URL | Default browser | The requested URL | When the user opens a URL |
| WebView2/Windows | Operating system services | Data according to Windows/WebView2 settings | Outside Cardumen Deck's control |

The application does not add credentials to HTTP health checks. If a URL contains tokens or credentials in the path or query string, that data may be sent to the configured endpoint.

Git may use Git Credential Manager or another system credential helper. Cardumen Deck does not store those credentials.

## Third parties

- **WebView2 and Windows** may collect diagnostics or telemetry according to their own settings.
- **Git and configured remotes** have their own policies.
- **Health check endpoints** receive the requests the user configures.
- **GitHub** hosts the repository, workflows, and releases; it does not receive application runtime data.

## Retention and deletion

- Logs are retained according to Cardumen Deck's retention settings.
- `config.json` and `runtime.json` remain until the user deletes them.
- To delete local data, close the application and remove:
  - `%APPDATA%\CardumenDeck`
  - `%LOCALAPPDATA%\CardumenDeck`

There is no remote backup or recovery by the maintainers.

## Local data security

Cardumen Deck does not encrypt its configuration or logs. Protection depends on the operating system, user permissions, and device encryption.

Recommendations:

- use a Windows account with least privilege;
- keep device encryption enabled, such as BitLocker;
- do not store production secrets in `config.json`;
- review logs before sharing them;
- keep configuration backups if they are important to your workflow.

## Real development use

Cardumen Deck is a local development tool. It is not a managed service or a production component. You may use it in your real development workflow, but you are responsible for the commands you run, the credentials you use, the data you store, and your backups.

If you use it on a company device or with company code, that company's internal policies may apply.

## Changes to this policy

This policy may be updated when application behavior changes. Relevant changes will be recorded in [CHANGELOG.md](CHANGELOG.md).

## Contact

For privacy questions, use the repository discussions. For vulnerabilities, use GitHub Security Advisories. Do not post secrets or personal data in public issues.

This document is not legal advice. See [LEGAL.md](LEGAL.md).
